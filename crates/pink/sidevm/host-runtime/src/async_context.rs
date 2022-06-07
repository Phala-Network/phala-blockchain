use std::cell::{Cell, RefCell};
use std::future::Future;
use std::pin::Pin;
use std::ptr::NonNull;
use std::sync::{Arc, Weak};
use std::task::{self, RawWaker, RawWakerVTable, Waker};

use crate::env::TaskSet;

thread_local! {
    static TLS_TASK_ENV: RefCell<Option<TaskEnv>> = RefCell::new(None);
}

#[derive(Clone)]
struct TaskEnv {
    tasks: Weak<TaskSet>,
    this_task: i32,
}

struct WakerData {
    env: TaskEnv,
    parent: Waker,
    guest_waker_id: i32,
}

impl Drop for WakerData {
    fn drop(&mut self) {
        if let Some(tasks) = self.env.tasks.upgrade() {
            // negative means drop it
            tasks
                .awake_wakers
                .lock()
                .unwrap()
                .push_back(-1 - self.guest_waker_id);
        }
    }
}

impl WakerData {
    fn wake_by_ref(&self) {
        if let Some(tasks) = self.env.tasks.upgrade() {
            tasks.push_task(self.env.this_task);
            tasks
                .awake_wakers
                .lock()
                .unwrap()
                .push_back(self.guest_waker_id);
        }
        self.parent.wake_by_ref();
    }
}

fn relay_waker(env: &TaskEnv, parent: &Waker, guest_waker_id: i32) -> Waker {
    fn raw_waker_from_data(data: Arc<WakerData>) -> RawWaker {
        let vtable = &RawWakerVTable::new(clone_waker, wake_waker, wake_by_ref_waker, drop_waker);
        RawWaker::new(Arc::into_raw(data) as *const (), vtable)
    }

    fn clone_waker(data: *const ()) -> RawWaker {
        let data = unsafe { Arc::from_raw(data as *const WakerData) };
        let cloned = data.clone();
        let _ = Arc::into_raw(data);
        raw_waker_from_data(cloned)
    }

    fn wake_waker(data: *const ()) {
        let data = unsafe { Arc::from_raw(data as *const WakerData) };
        data.wake_by_ref();
        drop(data);
    }

    fn wake_by_ref_waker(data: *const ()) {
        let data = unsafe { Arc::from_raw(data as *const WakerData) };
        data.wake_by_ref();
        let _ = Arc::into_raw(data);
    }

    fn drop_waker(data: *const ()) {
        unsafe {
            drop(Arc::from_raw(data as *const WakerData));
        }
    }
    let data = Arc::new(WakerData {
        env: env.clone(),
        parent: parent.clone(),
        guest_waker_id,
    });

    unsafe { Waker::from_raw(raw_waker_from_data(data)) }
}

struct ClearEnvOnDrop;
impl Drop for ClearEnvOnDrop {
    fn drop(&mut self) {
        TLS_TASK_ENV.with(|tls_task_env| {
            tls_task_env.borrow_mut().take();
        });
    }
}

/// Sets the thread-local task context used by async/await futures.
pub(crate) fn set_task_env<F, R>(awake_tasks: Arc<TaskSet>, task_id: i32, f: F) -> R
where
    F: FnOnce() -> R,
{
    TLS_TASK_ENV.with(|tls_task_env| {
        *tls_task_env.borrow_mut() = Some(TaskEnv {
            tasks: Arc::downgrade(&awake_tasks),
            this_task: task_id,
        });
    });
    let _clear_env = ClearEnvOnDrop;
    f()
}

// It is impossible to pass the task context into the VM. So we store it in a thead local storage.
// Just as the initial version of Rust's async/await did. The following codes are taken from
// https://github.com/rust-lang/rust/pull/51580/files#diff-2437bade3937fa15310072df88db95aa1d2cd047069275cdddaccf2e4b1dc431R53-R116

thread_local! {
    static TLS_CX: Cell<Option<NonNull<task::Context<'static>>>> = Cell::new(None);
}

struct SetOnDrop(Option<NonNull<task::Context<'static>>>);

impl Drop for SetOnDrop {
    fn drop(&mut self) {
        TLS_CX.with(|tls_cx| {
            tls_cx.set(self.0.take());
        });
    }
}

/// Sets the thread-local task context used by async/await futures.
pub(crate) fn set_task_cx<F, R>(cx: &mut task::Context, f: F) -> R
where
    F: FnOnce() -> R,
{
    let old_cx = TLS_CX.with(|tls_cx| {
        tls_cx.replace(NonNull::new(
            cx as *mut task::Context as *mut () as *mut task::Context<'static>,
        ))
    });
    let _reset_cx = SetOnDrop(old_cx);
    f()
}

/// Retrieves the thread-local task context used by async/await futures.
///
/// This function acquires exclusive access to the task context.
///
/// Panics if no task has been set or if the task context has already been
/// retrived by a surrounding call to get_task_cx.
pub(crate) fn get_task_cx<F, R>(guest_waker_id: i32, f: F) -> R
where
    F: FnOnce(&mut task::Context) -> R,
{
    let cx_ptr = TLS_CX.with(|tls_cx| {
        // Clear the entry so that nested `get_task_cx` calls
        // will fail or set their own value.
        tls_cx.replace(None)
    });
    let _reset_cx = SetOnDrop(cx_ptr);

    let mut cx_ptr = cx_ptr.expect("TLS task::Context not set. This is a bug.");

    TLS_TASK_ENV.with(move |tls_task_env| unsafe {
        let borrow = tls_task_env.borrow();
        let env = borrow
            .as_ref()
            .expect("TLS TaskEnv not set. This is a bug.");
        let parent = cx_ptr.as_mut().waker();
        let waker = relay_waker(env, parent, guest_waker_id);
        let mut cx = task::Context::from_waker(&waker);
        f(&mut cx)
    })
}

/// Polls a future in the current thread-local task context.
pub(crate) fn poll_in_task_cx<F: ?Sized>(
    guest_waker_id: i32,
    f: Pin<&mut F>,
) -> task::Poll<F::Output>
where
    F: Future,
{
    get_task_cx(guest_waker_id, |cx| f.poll(cx))
}

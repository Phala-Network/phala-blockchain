use std::cell::Cell;
use std::future::Future;
use std::pin::Pin;
use std::ptr::NonNull;
use std::task;

// It is impossible to pass the task context into the VM. So we store it in a thead local storage.
// Just as the initial version of Rust's async/await did. Codes are taken from
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
pub fn set_task_cx<F, R>(cx: &mut task::Context, f: F) -> R
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
pub fn get_task_cx<F, R>(task_id: i32, f: F) -> R
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
    unsafe { f(cx_ptr.as_mut()) }
}

/// Polls a future in the current thread-local task context.
pub fn poll_in_task_cx<F>(f: Pin<&mut F>, task_id: i32) -> task::Poll<F::Output>
where
    F: Future,
{
    get_task_cx(task_id, |cx| f.poll(cx))
}

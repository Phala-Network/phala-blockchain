use core::panic;

use super::*;

extern "Rust" {
    fn sidevm_main_future() -> Pin<Box<dyn Future<Output = ()>>>;
}

type TaskFuture = Pin<Box<dyn Future<Output = ()>>>;
type Tasks = Vec<Option<TaskFuture>>;

thread_local! {
    static CURRENT_TASK: std::cell::Cell<i32>  = Default::default();
    static TASKS: RefCell<Tasks> = RefCell::new(vec![Some(unsafe { sidevm_main_future() })]);
    static SPAWNING_TASKS: RefCell<Vec<TaskFuture>> = RefCell::new(vec![]);
}

// TODO.kevin: Support task joining
pub struct TaskHandle;

pub fn spawn(fut: impl Future<Output = ()> + 'static) -> TaskHandle {
    SPAWNING_TASKS.with(move |tasks| (*tasks).borrow_mut().push(Box::pin(fut)));
    TaskHandle
}

fn start_task(tasks: &mut Tasks, task: TaskFuture) {
    const MAX_N_TASKS: usize = (i32::MAX / 2) as _;

    for (task_id, task_ref) in tasks.iter_mut().enumerate().skip(1) {
        if task_ref.is_none() {
            *task_ref = Some(task);
            ocall::mark_task_ready(task_id as _).expect("Mark task ready failed");
            return;
        }
    }

    if tasks.len() < MAX_N_TASKS {
        let task_id = tasks.len();
        tasks.push(Some(task));
        ocall::mark_task_ready(task_id as _).expect("Mark task ready failed");
        return;
    }

    panic!("Spawn task failed, Max number of tasks reached");
}

fn start_spawned_tasks(tasks: &mut Tasks) {
    SPAWNING_TASKS.with(|spowned_tasks| {
        for task in spowned_tasks.borrow_mut().drain(..) {
            start_task(tasks, task);
        }
    })
}

pub(crate) fn current_task() -> i32 {
    CURRENT_TASK.with(|id| id.get())
}

fn set_current_task(task_id: i32) {
    CURRENT_TASK.with(|id| id.set(task_id))
}

fn poll_with_dummy_context<F>(f: Pin<&mut F>) -> task::Poll<F::Output>
where
    F: Future + ?Sized,
{
    fn raw_waker() -> task::RawWaker {
        task::RawWaker::new(
            &(),
            &task::RawWakerVTable::new(
                |_| raw_waker(),
                // Let's forbid to use the Context in wasm.
                |_| panic!("Dummy waker should never be called"),
                |_| panic!("Dummy waker should never be called"),
                |_| (),
            ),
        )
    }
    let waker = unsafe { task::Waker::from_raw(raw_waker()) };
    let mut context = task::Context::from_waker(&waker);
    f.poll(&mut context)
}

#[no_mangle]
extern "C" fn sidevm_poll() -> i32 {
    use task::Poll::*;

    fn poll() -> task::Poll<()> {
        loop {
            let task_id = match ocall::next_ready_task() {
                Ok(id) => id as usize,
                Err(OcallError::NotFound) => return task::Poll::Pending,
                Err(err) => panic!("Error occured: {:?}", err),
            };
            let exited = TASKS.with(|tasks| -> Option<bool> {
                let exited = {
                    let mut tasks = tasks.borrow_mut();
                    let task = tasks.get_mut(task_id)?.as_mut()?;
                    set_current_task(task_id as _);
                    match poll_with_dummy_context(task.as_mut()) {
                        Pending => (),
                        Ready(()) => {
                            tasks[task_id] = None;
                        }
                    }
                    tasks[0].is_none()
                };
                if !exited {
                    start_spawned_tasks(&mut *tasks.borrow_mut());
                }
                Some(exited)
            });
            if let Some(true) = exited {
                return task::Poll::Ready(());
            }
        }
    }
    match poll() {
        Ready(()) => 1,
        Pending => 0,
    }
}

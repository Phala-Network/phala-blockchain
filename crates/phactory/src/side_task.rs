use std::any::Any;

use crate::storage::Storage;
use ::chain::BlockNumber;

/// A side task is designed to do some async works in the BACKGROUND.
///
/// In the old days, we do all the chain block processing logic in the `dispatch_block` RPC call, synchronously.
/// But there are some cases, such as making an HTTP request or doing some CPU heavy computing, it is not acceptable
/// do it in the RPC call synchronously. So we need to move these works to another thread and report it's result in
/// the RPC call. That's what the `SideTask` is for.
///
/// When some async works are needed, we can create a `SideTask` and spawn some sort of background task to do the works.
/// When the background task is done, it sets the result into the corresponding `SideTask` and wait for the chain block
/// processing to poll it to process the result. We can then make some side-input (mq egress) in the result processing.
pub trait SideTask {
    /// The scheduler will call this function at any time, typically once each block, until it returns PollState::Complete.
    fn poll(self, block: &PollContext) -> PollState<Self>
    where
        Self: Sized;
}

pub struct PollContext<'a> {
    pub block_number: BlockNumber,
    pub storage: &'a Storage,
}

pub enum PollState<T> {
    /// The task is in progress.
    Running(T),
    /// The task is done. The task will be removed from the queue.
    Complete,
}

struct TaskWrapper {
    poll_fn: fn(
        Box<dyn Any + Send + 'static>,
        context: &PollContext,
    ) -> PollState<Box<dyn Any + Send + 'static>>,
    task: Box<dyn Any + Send + 'static>,
}

impl TaskWrapper {
    fn poll(self, context: &PollContext) -> PollState<Self> {
        match (self.poll_fn)(self.task, context) {
            PollState::Running(task) => PollState::Running(Self {
                poll_fn: self.poll_fn,
                task,
            }),
            PollState::Complete => PollState::Complete,
        }
    }

    fn wrap<T: SideTask + Send + 'static>(task: T) -> Self {
        fn poll_fn<T: SideTask + Send + 'static>(
            task: Box<dyn Any + Send + 'static>,
            context: &PollContext,
        ) -> PollState<Box<dyn Any + Send + 'static>> {
            let task: Box<T> = task.downcast().expect("Should nerver fail");
            match task.poll(context) {
                PollState::Running(task) => PollState::Running(Box::new(task)),
                PollState::Complete => PollState::Complete,
            }
        }
        Self {
            poll_fn: poll_fn::<T>,
            task: Box::new(task),
        }
    }
}

#[derive(Default)]
pub struct SideTaskManager {
    tasks: Vec<TaskWrapper>,
}

impl SideTaskManager {
    pub fn poll(&mut self, context: &PollContext) {
        let mut remain = vec![];
        for task in self.tasks.drain(..) {
            match task.poll(&context) {
                PollState::Running(task) => remain.push(task),
                PollState::Complete => (),
            }
        }
        self.tasks = remain;
    }

    pub fn add_task<T: SideTask + Send + 'static>(&mut self, task: T) {
        self.tasks.push(TaskWrapper::wrap(task));
    }

    pub fn tasks_count(&self) -> usize {
        self.tasks.len()
    }
}

pub mod async_side_task {
    use async_executor::Task;
    use chain::BlockNumber;
    use futures::Future;
    use std::sync::{Arc, Mutex};

    use crate::side_task::{PollContext, PollState, SideTask};

    #[must_use = "SideTask will loss it's work without adding it to the task manager"]
    pub struct AsyncSideTask<Tsk, Rlt, Proc> {
        report_at: BlockNumber,
        result: Arc<Mutex<Option<Rlt>>>,
        result_process: Proc,
        _async_task: Tsk,
    }

    impl<Tsk, Rlt, Proc> SideTask for AsyncSideTask<Tsk, Rlt, Proc>
    where
        Tsk: Send,
        Rlt: Send,
        Proc: Send + FnOnce(Option<Rlt>, &PollContext),
    {
        fn poll(self, context: &PollContext) -> PollState<Self> {
            if context.block_number >= self.report_at {
                let result = self.result.lock().unwrap().take();
                (self.result_process)(result, context);
                PollState::Complete
            } else {
                PollState::Running(self)
            }
        }
    }

    impl<Rlt, Proc> AsyncSideTask<Task<()>, Rlt, Proc>
    where
        Rlt: Send + 'static,
        Proc: Send + FnOnce(Option<Rlt>, &PollContext),
    {
        /// Create a new `AsyncSideTask`.
        ///
        /// The task will finish at `block_number` + `duration` blocks.
        ///
        /// The task_future is a future to do an async task (e.g. a http request) to fetch some async resoures.
        ///
        /// And process the result in result_process (e.g. report a mq message).
        ///
        /// # Examples
        ///
        /// ```ignore
        /// let mut task_man = SideTaskManager::default();
        /// let cur_block = 100;
        /// let duration = 3;
        /// let task = AsyncSideTask::spawn(cur_block, 3, async {
        ///         surf::get("https://ifconfig.me").await.unwrap().body_string().await.unwrap()
        ///     },
        ///     |result, _context| {
        ///         // process the result here
        ///     },
        /// );
        /// task_man.add_task(task);
        /// ```
        pub fn spawn(
            block_number: BlockNumber,
            duration: BlockNumber,
            task_future: impl Future<Output = Rlt> + Send + 'static,
            result_process: Proc,
        ) -> Self {
            let result = Arc::new(Mutex::new(None));
            let set_result = result.clone();

            let task = phala_async_executor::spawn(async move {
                let result = task_future.await;
                *set_result.lock().unwrap() = Some(result);
            });

            AsyncSideTask {
                report_at: block_number + duration,
                result,
                result_process,
                _async_task: task,
            }
        }
    }
}

use crate::storage::Storage;
use ::chain::BlockNumber;
use phala_mq::MessageSendQueue;

type SigningMessage = phala_mq::SigningMessage<sp_core::sr25519::Pair>;

pub struct PollContext<'a> {
    pub block_number: BlockNumber,
    pub send_mq: &'a MessageSendQueue,
    pub storage: &'a Storage,
}

struct TaskWrapper {
    on_finish: Box<dyn FnOnce(&PollContext) -> Option<Vec<SigningMessage>> + Send + 'static>,
    default_messages: Vec<SigningMessage>,
    end_block: BlockNumber,
}

impl TaskWrapper {
    fn finish(self, context: &PollContext) {
        let messages = (self.on_finish)(context).unwrap_or(self.default_messages);
        for msg in messages {
            context
                .send_mq
                .enqueue_message(msg.message.sender.clone(), |seq| msg.sign(seq));
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
            if task.end_block > context.block_number {
                remain.push(task);
                continue;
            }
            if task.end_block == context.block_number {
                task.finish(context);
                continue;
            }
            error!(
                "BUG: side task end at past block, end_block={} current_block={}",
                task.end_block, context.block_number
            );
        }
        self.tasks = remain;
    }

    pub fn add_task<
        F: FnOnce(&PollContext) -> Option<[SigningMessage; N]> + Send + 'static,
        const N: usize,
    >(
        &mut self,
        current_block: BlockNumber,
        duration: BlockNumber,
        default_messages: [SigningMessage; N],
        finish: F,
    ) {
        let task = TaskWrapper {
            on_finish: Box::new(move |context| finish(context).map(|arr| arr.to_vec())),
            default_messages: default_messages.to_vec(),
            end_block: current_block + duration,
        };
        self.tasks.push(task);
    }

    pub fn tasks_count(&self) -> usize {
        self.tasks.len()
    }
}

pub mod async_side_task {
    use anyhow::Result;
    use async_executor::Task;
    use futures::Future;
    use std::sync::{Arc, Mutex};

    use crate::side_task::PollContext;

    use super::{BlockNumber, SigningMessage};

    #[must_use = "SideTask will loss it's work without adding it to the task manager"]
    pub struct AsyncSideTask<Tsk, const N: usize> {
        result: Arc<Mutex<Option<[SigningMessage; N]>>>,
        _async_task: Tsk,
    }

    impl<Tsk, const N: usize> AsyncSideTask<Tsk, N>
    where
        Tsk: Send,
    {
        fn finish(self, _context: &PollContext) -> Option<[SigningMessage; N]> {
            self.result.lock().unwrap().take()
        }
    }

    impl<const N: usize> AsyncSideTask<Task<()>, N> {
        /// Create a new `AsyncSideTask`.
        ///
        /// The task will finish at `block_number` + `duration` blocks.
        ///
        /// The `future` is a future to do an async task (e.g. a http request) to fetch some async resoures.
        ///
        /// And process the result in result_process (e.g. report a mq message).
        ///
        /// # Examples
        ///
        /// ```ignore
        /// let mut task_man = SideTaskManager::default();
        /// let cur_block = 100;
        /// let duration = 3;
        /// let task = AsyncSideTask::spawn(async {
        ///         let ip = surf::get("https://ifconfig.me").await.unwrap().body_string().await.unwrap()
        ///         [msg_ch.prepare_message(mk_msg_from_ip(ip))]
        ///     },
        /// );
        /// task_man.add_task_finish_at(cur_block + duration, [mk_default_msg()], |c| task.finish(c));
        /// ```
        pub fn spawn(
            future: impl Future<Output = Result<[SigningMessage; N]>> + Send + 'static,
        ) -> Self {
            let result = Arc::new(Mutex::new(None));
            let set_result = result.clone();

            let task = phala_async_executor::spawn(async move {
                let result = future.await;
                if let Err(err) = &result {
                    log::error!("Async side task returns error: {:?}", err);
                }
                *set_result.lock().unwrap() = result.ok();
            });

            AsyncSideTask {
                result,
                _async_task: task,
            }
        }
    }

    impl super::SideTaskManager {
        pub fn add_async_task<
            F: Future<Output = Result<[SigningMessage; N]>> + Send + 'static,
            const N: usize,
        >(
            &mut self,
            current_block: BlockNumber,
            duration: BlockNumber,
            default_messages: [SigningMessage; N],
            future: F,
        ) {
            let task = AsyncSideTask::spawn(future);
            self.add_task(current_block, duration, default_messages, |context| task.finish(context));
        }
    }
}

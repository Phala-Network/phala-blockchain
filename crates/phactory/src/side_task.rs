use crate::storage::Storage;
use ::chain::BlockNumber;

pub trait SideTask {
    // The scheduler will call this function at any time, typically once each block, until it returns PollState::Complete.
    fn poll(&self, block: &Context) -> PollState;
}

pub struct Context<'a> {
    pub block_number: BlockNumber,
    pub storage: &'a Storage,
}

pub enum PollState {
    // The task is in progress and expects to be polled again at the next_poll_block.
    Running { next_poll_block: BlockNumber },
    // The task is done. The task will be removed from the queue.
    Complete,
}

impl PollState {
    pub fn is_running(&self) -> bool {
        matches!(self, PollState::Running { .. })
    }
}

#[derive(Default)]
pub struct SideTaskManager {
    tasks: Vec<Box<dyn SideTask + Send>>,
}

impl SideTaskManager {
    pub fn poll(&mut self, context: &Context) {
        self.tasks.retain(|task| {
            task.poll(&context).is_running()
        });
    }

    pub fn add_task<T: SideTask + Send + 'static>(&mut self, task: T) {
        self.tasks.push(Box::new(task));
    }
}

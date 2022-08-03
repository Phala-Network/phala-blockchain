use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::{Arc, Mutex, Weak};
use std::time::Instant;

use rbtree::RBTree;
use std::task;

pub type VirtualTime = u128;

pub trait TaskIdType: Clone + Send + Eq + Hash + Debug + 'static {}
impl<T: Clone + Send + Eq + Hash + Debug + 'static> TaskIdType for T {}

type WeakScheduler<TaskId> = Weak<Mutex<SchedulerInner<TaskId>>>;

#[derive(Clone)]
pub struct TaskScheduler<TaskId: TaskIdType> {
    inner: Arc<Mutex<SchedulerInner<TaskId>>>,
}

impl<TaskId: TaskIdType> TaskScheduler<TaskId> {
    pub fn new(virtual_cores: u32) -> Self {
        Self {
            inner: Arc::new_cyclic(|weak_inner| {
                Mutex::new(SchedulerInner::new(virtual_cores, weak_inner.clone()))
            }),
        }
    }

    pub fn poll_resume(
        &self,
        cx: &task::Context<'_>,
        task_id: &TaskId,
        weight: u32,
    ) -> task::Poll<RunningGuard<TaskId>> {
        self.inner.lock().unwrap().poll_resume(cx, task_id, weight)
    }

    pub fn exit(&self, task_id: &TaskId) {
        self.inner.lock().unwrap().exit(task_id)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum TaskState {
    Idle,
    Ready,
    ToRun,
    Running,
}

struct Task {
    state: TaskState,
    virtual_runtime: VirtualTime,
}

struct ReadyTask<TaskId> {
    id: TaskId,
    waker: task::Waker,
}

pub struct RunningGuard<TaskId: TaskIdType> {
    queue: WeakScheduler<TaskId>,
    task_id: TaskId,
    start_time: Instant,
    actual_cost: Option<VirtualTime>,
    weight: u32,
}

impl<TaskId: TaskIdType> RunningGuard<TaskId> {
    pub fn set_cost(&mut self, cost: VirtualTime) {
        self.actual_cost = Some(cost);
    }
}

struct SchedulerInner<TaskId: TaskIdType> {
    weak_self: WeakScheduler<TaskId>,
    tasks: HashMap<TaskId, Task>,
    ready_tasks: RBTree<VirtualTime, ReadyTask<TaskId>>,
    virtual_clock: VirtualTime,
    virtual_cores: u32,
    running_tasks: u32,
}

unsafe impl<T: TaskIdType> Send for SchedulerInner<T> {}

impl<TaskId: TaskIdType> SchedulerInner<TaskId> {
    fn new(virtual_cores: u32, weak_self: WeakScheduler<TaskId>) -> Self {
        Self {
            weak_self,
            tasks: Default::default(),
            ready_tasks: RBTree::new(),
            virtual_cores,
            virtual_clock: 0,
            running_tasks: 0,
        }
    }

    fn exit(&mut self, id: &TaskId) {
        let task = match self.tasks.remove(id) {
            Some(task) => task,
            None => return,
        };
        if matches!(task.state, TaskState::ToRun | TaskState::Running) {
            self.running_tasks -= 1;
            self.schedule();
        }
    }

    fn poll_resume(
        &mut self,
        cx: &task::Context<'_>,
        id: &TaskId,
        weight: u32,
    ) -> task::Poll<RunningGuard<TaskId>> {
        self.maybe_reset_clock();

        let task = self.tasks.entry(id.clone()).or_insert_with(|| Task {
            state: TaskState::Idle,
            virtual_runtime: self.virtual_clock,
        });

        match task.state {
            TaskState::Idle => {
                let ready = ReadyTask {
                    id: id.clone(),
                    waker: cx.waker().clone(),
                };
                task.virtual_runtime = task.virtual_runtime.max(self.virtual_clock);
                task.state = TaskState::Ready;
                self.ready_tasks.insert(task.virtual_runtime, ready);
                self.schedule();
                task::Poll::Pending
            }
            TaskState::Ready => task::Poll::Pending,
            TaskState::ToRun => {
                let guard = RunningGuard {
                    queue: self.weak_self.clone(),
                    task_id: id.clone(),
                    start_time: Instant::now(),
                    actual_cost: None,
                    weight,
                };
                task.state = TaskState::Running;
                task::Poll::Ready(guard)
            }
            TaskState::Running => panic!("BUG: resuming a running task"),
        }
    }

    fn park(&mut self, task_id: &TaskId, actual_cost: VirtualTime) {
        let task = match self.tasks.get_mut(task_id) {
            Some(task) => task,
            None => return,
        };

        assert_eq!(
            task.state,
            TaskState::Running,
            "BUG: parking a non-running task"
        );

        task.virtual_runtime += actual_cost.max(1);
        task.state = TaskState::Idle;
        self.running_tasks -= 1;
        self.schedule();
    }

    fn schedule(&mut self) {
        while self.running_tasks < self.virtual_cores {
            let (vruntime, ready_task) = match self.ready_tasks.pop_first() {
                Some(v) => v,
                None => break,
            };
            let task = match self.tasks.get_mut(&ready_task.id) {
                Some(task) => task,
                // The task has already been dropped.
                None => continue,
            };
            self.running_tasks += 1;
            self.virtual_clock = vruntime;

            task.state = TaskState::ToRun;
            ready_task.waker.wake();
        }
    }

    fn maybe_reset_clock(&mut self) {
        if self.virtual_clock > VirtualTime::MAX / 2 {
            for task in self.tasks.values_mut() {
                task.virtual_runtime = task.virtual_runtime.saturating_sub(self.virtual_clock);
            }
            self.virtual_clock = 0;
        }
    }
}

impl<TaskId: TaskIdType> Drop for RunningGuard<TaskId> {
    fn drop(&mut self) {
        if let Some(inner) = self.queue.upgrade() {
            let actual_cost = self
                .actual_cost
                .unwrap_or_else(|| self.start_time.elapsed().as_nanos() as VirtualTime);
            let actual_cost = actual_cost / self.weight.max(1) as VirtualTime;
            inner.lock().unwrap().park(&self.task_id, actual_cost);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::future::Future;
    use std::pin::Pin;
    use std::time::Duration;

    struct TestTask {
        scheduler: TaskScheduler<u32>,
        id: u32,
        weight: u32,
        cost: VirtualTime,
        realtime: VirtualTime,
    }

    impl Future for TestTask {
        type Output = ();
        fn poll(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
            let _guard = match self.scheduler.poll_resume(cx, &self.id, self.weight) {
                task::Poll::Ready(guard) => guard,
                task::Poll::Pending => return task::Poll::Pending,
            };
            std::thread::sleep(Duration::from_millis(self.cost as _));
            self.realtime += self.cost;
            println!(
                "Task [{}] w={}, t={}, r={}",
                self.id,
                self.weight,
                self.realtime,
                self.realtime / self.weight as VirtualTime,
            );
            task::Poll::Ready(())
        }
    }

    #[tokio::test]
    #[ignore]
    async fn it_works() {
        let scheduler = TaskScheduler::new(3);
        let mut tasks = vec![];
        for id in 1..=9_u32 {
            let scheduler = scheduler.clone();
            let handle = tokio::spawn(async move {
                let task = TestTask {
                    scheduler,
                    id,
                    weight: id,
                    cost: 100,
                    realtime: 0,
                };
                tokio::pin!(task);
                loop {
                    (&mut task).await;
                }
            });
            tasks.push(handle);
        }
        for task in tasks {
            let _ = task.await;
        }
    }
}

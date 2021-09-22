use async_executor::{Executor, Task};
use std::future::Future;

static EXECUTOR: Executor<'static> = Executor::new();

pub fn spawn<T: Send + 'static>(future: impl Future<Output = T> + Send + 'static) -> Task<T> {
    #[cfg(not(feature = "no-thread"))]
    {
        use std::sync::Once;

        static START_EXECUTOR: Once = Once::new();

        START_EXECUTOR.call_once(|| {
            std::thread::spawn(|| run_executor());
        });
    }
    EXECUTOR.spawn(async {
        // Delay one second to avoid net traffic storm when pRuntime replaying blocks.
        let _ = async_std::task::sleep(std::time::Duration::from_secs(1)).await;
        future.await
    })
}

pub fn run_executor() {
    async_io::block_on(EXECUTOR.run(futures::future::pending::<()>()));
}

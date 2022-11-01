use async_executor::{Executor, Task};
use std::future::Future;

static EXECUTOR: Executor<'static> = Executor::new();

pub fn spawn<T: Send + 'static>(future: impl Future<Output = T> + Send + 'static) -> Task<T> {
    #[cfg(not(feature = "no-thread"))]
    {
        // For testing codes runing outside of SGX environment.
        use std::sync::Once;

        static START_EXECUTOR: Once = Once::new();

        START_EXECUTOR.call_once(|| {
            std::thread::spawn(run_executor);
        });
    }
    EXECUTOR.spawn(async {
        // TODO: find out better ways to bypass the async when syncing.
        // Delay one second to avoid net traffic storm when pRuntime replaying blocks.
        async_std::task::sleep(std::time::Duration::from_secs(1)).await;
        future.await
    })
}

pub fn run_executor() {
    futures::executor::block_on(EXECUTOR.run(futures::future::pending::<()>()));
}

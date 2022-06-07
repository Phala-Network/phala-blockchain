//! Executors to bridge frameworks to sidevm.

/// A hyper executor that executes futures with sidevm::spawn.
#[cfg(feature = "hyper")]
#[derive(Clone, Copy, Debug)]
pub struct HyperExecutor;

#[cfg(feature = "hyper")]
impl<F> hyper::rt::Executor<F> for HyperExecutor
where
    F: std::future::Future + 'static,
{
    fn execute(&self, fut: F) {
        crate::spawn(fut);
    }
}

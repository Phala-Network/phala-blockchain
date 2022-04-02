use std::future::Future;
use std::pin::Pin;
use std::sync::Mutex;
use std::task::{self, Context, Poll, Waker};
use std::time::Duration;

use pink_sidevm_env::{OcallError as Errno, OcallFuncsImplement as Ocall};

use once_cell::sync::Lazy;

struct Sleep {
    // Resouce ID. Think of it as a FD.
    id: i32,
}

impl Drop for Sleep {
    fn drop(&mut self) {
        let _ = Ocall.close(self.id);
    }
}

fn sleep(duration: Duration) -> Sleep {
    let id = Ocall.create_timer(duration.as_millis() as i32);
    if id == -1 {
        panic!("failed to create timer");
    }
    Sleep { id }
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let rv = Ocall.poll(self.id, 0);
        if rv == Errno::Pending as i32 {
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

async fn main() {
    sleep(Duration::from_secs(3)).await
}

// entry point
#[no_mangle]
extern "C" fn sidevm_poll() -> i32 {
    static MAIN_FUTURE: Lazy<Mutex<Pin<Box<dyn Future<Output = ()> + Sync + Send>>>> =
        Lazy::new(|| Mutex::new(Box::pin(main())));

    match poll_with_dummy_context(MAIN_FUTURE.lock().unwrap().as_mut()) {
        Poll::Ready(()) => 1,
        Poll::Pending => 0,
    }
}

pub fn poll_with_dummy_context<F>(f: Pin<&mut F>) -> Poll<F::Output>
where
    F: Future + ?Sized,
{
    fn raw_waker() -> task::RawWaker {
        task::RawWaker::new(
            &mut (),
            &task::RawWakerVTable::new(
                |_| raw_waker(),
                // We never really use the Context
                |_| panic!("Dummy waker should never be called"),
                |_| panic!("Dummy waker should never be called"),
                |_| (),
            ),
        )
    }
    let waker = unsafe { Waker::from_raw(raw_waker()) };
    let mut context = Context::from_waker(&waker);
    f.poll(&mut context)
}

use std::future::Future;
use std::pin::Pin;
use std::sync::Mutex;
use std::task::{self, Context, Poll, Waker};
use std::time::Duration;

use pink_sidevm_env::{self as env, ocall_funcs_guest as ocall};

use once_cell::sync::Lazy;

/// Resource ID. Think of it as a FD.
pub struct ResourceId(pub i32);

impl Drop for ResourceId {
    fn drop(&mut self) {
        let _ = ocall::close(self.0);
    }
}

struct Sleep {
    id: ResourceId,
}

fn sleep(duration: Duration) -> Sleep {
    let id = ocall::create_timer(duration.as_millis() as i32).expect("failed to create timer");
    Sleep { id: ResourceId(id) }
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let rv = ocall::poll(self.id.0).expect("Poll timer failed");
        match rv {
            env::Poll::Ready(_) => Poll::Ready(()),
            env::Poll::Pending => Poll::Pending,
        }
    }
}

async fn main() {
    ocall::enable_ocall_trace(true).unwrap();
    assert_eq!(ocall::echo(vec![4, 2]).unwrap(), vec![4, 2]);
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

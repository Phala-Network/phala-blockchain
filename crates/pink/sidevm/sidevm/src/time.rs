//! Provides functionalities in tokio::time.

use super::*;

use core::pin::Pin;
use derive_more::{Display, Error};
use std::future::Future;
use std::task::{Context, Poll};
use std::time::Duration;

use crate::ResourceId;

/// The future to sleep for a given duration.
pub struct Sleep {
    id: ResourceId,
}

/// Sleep for the specified duration.
///
/// # Example
/// ```ignore
/// use pink_sidevm::time;
/// time::sleep(Duration::from_millis(100)).await;
/// ```
pub fn sleep(duration: Duration) -> Sleep {
    let id = ocall::create_timer(duration.as_millis() as i32).expect("failed to create timer");
    Sleep { id: ResourceId(id) }
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        use env::OcallError;
        let waker_id = env::tasks::intern_waker(cx.waker().clone());
        let rv = ocall::poll_read(waker_id, self.id.0, &mut []);
        match rv {
            Ok(_) => Poll::Ready(()),
            Err(OcallError::Pending) => Poll::Pending,
            Err(err) => panic!("unexpected error: {:?}", err),
        }
    }
}

/// Indicates that a timeout has elapsed for `timeout(future)`.
#[derive(Display, Error, Debug)]
pub struct TimedOut;

/// Timeout the provided future for the specified duration.
pub async fn timeout<T: Future<Output = O>, O>(
    duration: Duration,
    future: T,
) -> Result<O, TimedOut> {
    use futures::FutureExt;
    futures::select! {
        v = future.fuse() => Ok(v),
        _ = sleep(duration).fuse() => Err(TimedOut),
    }
}

/// The future returned by `take_a_rest_if_needed`.
pub struct Rest {
    resting: bool
}

impl Future for Rest {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.resting {
            // Return `Pending` and become Ready immediately.
            self.resting = false;
            cx.waker().wake_by_ref();
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

/// Take a rest if it it about to stifled
///
/// If the remaining gas of current slot is less than 30%, the returned future will be `Pending` on
/// first poll and become `Ready` again immediately.
/// If the remaining gas is equal or more than 30%, the returned future will be `Ready` immediately.
pub fn take_a_rest_if_needed() -> Rest {
    let remaining = ocall::gas_remaining().expect("failed to get gas remaining");
    // Yield if there is less than 30% of gas remaining.
    Rest {
        resting: remaining < 30,
    }
}

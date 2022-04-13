//! Provides functionalities in tokio::time.

use super::*;

use core::pin::Pin;
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

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let rv = ocall::poll_read(self.id.0, &mut []).expect("Poll timer failed");
        match rv {
            env::Poll::Ready(_) => Poll::Ready(()),
            env::Poll::Pending => Poll::Pending,
        }
    }
}

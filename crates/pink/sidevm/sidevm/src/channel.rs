//! Multi-producer, single-consumer channel implementation.
use super::{ocall, ResourceId};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

/// Receiver end of a channel.
pub struct Receiver {
    res_id: ResourceId,
}

/// The future to get the next message from the channel.
pub struct RxNext<'a> {
    ch: &'a Receiver,
}

impl Receiver {
    /// Create a new `Receiver` from a `ResourceId`.
    pub const fn new(res_id: ResourceId) -> Self {
        Self { res_id }
    }

    /// Get the next message from the channel.
    pub fn next(&self) -> RxNext {
        RxNext { ch: self }
    }
}

impl Future for RxNext<'_> {
    type Output = Option<Vec<u8>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        use crate::env::OcallError;
        let waker_id = crate::env::tasks::intern_waker(cx.waker().clone());
        match ocall::poll(waker_id, self.ch.res_id.0) {
            Ok(msg) => Poll::Ready(Some(msg)),
            Err(OcallError::EndOfFile) => Poll::Ready(None), // tx dropped
            Err(OcallError::Pending) => Poll::Pending,
            Err(err) => panic!("unexpected error: {:?}", err),
        }
    }
}

/// The Pink standard input messages channel. Think of it as a stdin of a normal process.
///
/// When the sidevm instance is being killed, the tx in the runtime is droped while the instance is
/// running. At this time the rx-end might receive a None which indicate the tx-end has been closed.
pub fn input_messages() -> &'static Receiver {
    static MSG_RX: Receiver = Receiver::new(ResourceId(0));
    &MSG_RX
}

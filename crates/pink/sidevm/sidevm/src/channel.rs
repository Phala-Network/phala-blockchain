//! Multi-producer, single-consumer channel implementation.
use pink_sidevm_env::{
    query::{AccountId, QueryRequest},
    OcallError,
};

use super::{ocall, ResourceId};
use scale::Decode;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

/// A query from an external RPC request.
pub struct Query {
    /// The account sending the query.
    pub origin: AccountId,
    /// The query payload.
    pub payload: Vec<u8>,
    /// The reply channel. Invoke `send` on this channel to send the reply.
    pub reply_tx: OneshotSender,
}

/// A message from ink! to the side VM.
pub type Message = Vec<u8>;

/// Sender end of a oneshot channel connected to host-side.
pub struct OneshotSender {
    res_id: ResourceId,
}

impl OneshotSender {
    const fn new(res_id: ResourceId) -> Self {
        Self { res_id }
    }

    /// Send a message to the host-side.
    pub fn send(self, data: &[u8]) -> Result<(), OcallError> {
        ocall::oneshot_send(self.res_id.0, data)
    }
}

/// Receiver end of a channel connected to host-side.
pub struct Receiver<M> {
    res_id: ResourceId,
    _marker: std::marker::PhantomData<M>,
}

/// The future to get the next message from the channel.
pub struct Next<'a, M> {
    ch: &'a Receiver<M>,
}

impl<T> Receiver<T> {
    /// Create a new `Receiver` from a `ResourceId`.
    pub const fn new(res_id: ResourceId) -> Self {
        Self {
            res_id,
            _marker: std::marker::PhantomData,
        }
    }

    /// Get the next message from the channel.
    pub fn next(&self) -> Next<T> {
        Next { ch: self }
    }
}

impl Future for Next<'_, Message> {
    type Output = Option<Message>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let waker_id = crate::env::tasks::intern_waker(cx.waker().clone());
        match ocall::poll(waker_id, self.ch.res_id.0) {
            Ok(msg) => Poll::Ready(Some(msg)),
            Err(OcallError::EndOfFile) => Poll::Ready(None), // tx dropped
            Err(OcallError::Pending) => Poll::Pending,
            Err(err) => panic!("unexpected error: {:?}", err),
        }
    }
}

impl Future for Next<'_, Query> {
    type Output = Option<Query>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let waker_id = crate::env::tasks::intern_waker(cx.waker().clone());
        match ocall::poll(waker_id, self.ch.res_id.0) {
            Ok(msg) => {
                let request =
                    QueryRequest::decode(&mut &msg[..]).expect("Failed to decode QueryRequest");
                let reply_tx = OneshotSender::new(ResourceId(request.reply_tx));
                Poll::Ready(Some(Query {
                    origin: request.origin,
                    payload: request.payload,
                    reply_tx,
                }))
            }
            Err(OcallError::EndOfFile) => Poll::Ready(None), // The tx dropped
            Err(OcallError::Pending) => Poll::Pending,
            Err(err) => panic!("unexpected error: {:?}", err),
        }
    }
}

/// The Pink standard input messages channel. Think of it as a stdin of a normal process.
///
/// When the sidevm instance is being killed, the tx in the runtime is droped while the instance is
/// running. At this time the rx-end might receive a None which indicate the tx-end has been closed.
pub fn input_messages() -> &'static Receiver<Message> {
    static MSG_RX: Receiver<Message> = Receiver::new(ResourceId(0));
    &MSG_RX
}

/// Queries from RPC channel.
pub fn incoming_queries() -> &'static Receiver<Query> {
    static QUERY_RX: Receiver<Query> = Receiver::new(ResourceId(1));
    &QUERY_RX
}

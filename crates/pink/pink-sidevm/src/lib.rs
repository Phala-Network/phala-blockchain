//! ocall/ecall
//! async model
//! resource model

use std::future::Future;
use std::pin::Pin;
use std::task::{Poll, Context};

pub struct WasmInstance {

}

pub enum State {
    Running,
    Stopped,
}

impl Future for WasmInstance {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Pending
    }
}

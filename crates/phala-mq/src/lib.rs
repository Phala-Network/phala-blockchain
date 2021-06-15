#![no_std]

#[cfg(feature = "sgx")]
pub extern crate serde_sgx as serde;

extern crate alloc;

mod types;
#[cfg(feature = "dispatcher")]
mod dispatcher;
#[cfg(feature = "queue")]
mod send_queue;
#[cfg(any(feature = "queue", feature = "dispatcher"))]
mod simple_mpsc;

#[cfg(feature = "dispatcher")]
pub use dispatcher::MessageDispatcher;
#[cfg(feature = "queue")]
pub use send_queue::{MessageSendHandle, MessageSendQueue, Signer};
pub use types::*;

// TODO.kevin: use std::sync::Mutex instead.
// See:
//    https://matklad.github.io/2020/01/02/spinlocks-considered-harmful.html
//    https://matklad.github.io/2020/01/04/mutexes-are-faster-than-spinlocks.html
#[cfg(any(feature = "queue", feature = "dispatcher"))]
use spin::mutex::Mutex;

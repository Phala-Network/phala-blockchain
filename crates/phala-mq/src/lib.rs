#![no_std]

extern crate alloc;

mod types;
mod simple_mpsc;
mod dispatcher;
mod send_queue;

pub use types::*;
pub use send_queue::{MessageSendQueue, MessageSendHandle, Signer};
pub use dispatcher::MessageDispatcher;

// TODO.kevin: use std::sync::Mutex instead.
// See:
//    https://matklad.github.io/2020/01/02/spinlocks-considered-harmful.html
//    https://matklad.github.io/2020/01/04/mutexes-are-faster-than-spinlocks.html
use spin::mutex::Mutex;

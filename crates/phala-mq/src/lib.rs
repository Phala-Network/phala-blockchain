#![no_std]

#[cfg(feature = "sgx")]
pub extern crate serde_sgx as serde;

extern crate alloc;

mod signer;
pub mod types;

#[cfg(feature = "dispatcher")]
mod dispatcher;
#[cfg(feature = "queue")]
mod send_queue;
#[cfg(any(feature = "queue", feature = "dispatcher"))]
mod simple_mpsc;

#[cfg(feature = "dispatcher")]
pub use dispatcher::{MessageDispatcher, TypedReceiveError, TypedReceiver};
#[cfg(feature = "queue")]
pub use send_queue::{MessageChannel, MessageSendQueue};
#[cfg(any(feature = "queue", feature = "dispatcher"))]
pub use simple_mpsc::{ReceiveError, Receiver};

pub use signer::MessageSigner;

pub use types::*;

// TODO.kevin: use std::sync::Mutex instead.
// See:
//    https://matklad.github.io/2020/01/02/spinlocks-considered-harmful.html
//    https://matklad.github.io/2020/01/04/mutexes-are-faster-than-spinlocks.html
#[cfg(any(feature = "queue", feature = "dispatcher"))]
use spin::mutex::Mutex;

#[cfg(all(feature = "queue", feature = "signers"))]
pub use alias::*;

#[cfg(all(feature = "queue", feature = "signers"))]
mod alias {
    use super::*;
    use sp_core::sr25519;
    pub type Sr25519MessageChannel = MessageChannel<sr25519::Pair>;
}

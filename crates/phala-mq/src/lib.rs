#![no_std]

#[cfg(feature = "sgx")]
pub extern crate serde_sgx as serde;

extern crate alloc;

mod types;
mod signer;

#[cfg(feature = "dispatcher")]
mod dispatcher;
#[cfg(feature = "queue")]
mod send_queue;
#[cfg(any(feature = "queue", feature = "dispatcher"))]
mod simple_mpsc;

#[cfg(feature = "dispatcher")]
pub use dispatcher::MessageDispatcher;
#[cfg(feature = "queue")]
pub use send_queue::{MessageSendQueue, MessageChannel, TypedMessageChannel};

pub use signer::MessageSigner;

pub use types::*;

// TODO.kevin: use std::sync::Mutex instead.
// See:
//    https://matklad.github.io/2020/01/02/spinlocks-considered-harmful.html
//    https://matklad.github.io/2020/01/04/mutexes-are-faster-than-spinlocks.html
#[cfg(any(feature = "queue", feature = "dispatcher"))]
use spin::mutex::Mutex;


#[cfg(all(feature = "queue", feature="signers"))]
pub use alias::*;

#[cfg(all(feature = "queue", feature="signers"))]
mod alias {
    use super::*;
    use sp_core::ecdsa;
    pub type EcdsaTypedMessageChannel<T> = TypedMessageChannel<ecdsa::Pair, T>;
}
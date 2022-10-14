#![cfg_attr(not(feature = "std"), no_std)]

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

#[cfg(feature = "checkpoint")]
pub mod checkpoint_helper;

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
    pub use crate::signer::signers::Sr25519Signer;
    pub type SignedMessageChannel = crate::MessageChannel<Sr25519Signer>;
}

pub mod traits {
    use parity_scale_codec::Encode;

    use crate::{BindTopic, Path, SigningMessage};

    /// A MessageChannel is used to push messages into the egress queue, then the messages
    /// are ready to be synchronized to the chain by pherry or prb.
    pub trait MessageChannel {
        type Signer;
        /// Push given binary data as message payload into the egress queue.
        fn push_data(&self, data: alloc::vec::Vec<u8>, topic: impl Into<Path>);
        /// Same as push_data, except that it a SCALE encodable typed message which will be encoded into binary data.
        fn push_message_to(&self, message: &impl Encode, topic: impl Into<Path>) {
            self.push_data(message.encode(), topic)
        }
        /// Same as push_message_to, except that the type of message is bound on a topic.
        fn push_message<M: Encode + BindTopic>(&self, message: &M) {
            self.push_message_to(message, M::topic())
        }
        fn set_dummy(&self, _dummy: bool) {}
        /// Set signer for the channel.
        fn set_signer(&mut self, _signer: Self::Signer) {}
    }

    /// A MessagePrepareChannel is used prepare messages which later can be pushed into the message queue.
    ///
    /// The purpose of this extra step is that sometimes(side-task e.g.) we need store pre-generated messages
    /// somewhere and push it later. But the final `SignedMessage` contains the sequence which must be signed can
    /// not be known in advance until it about to be pushed out. So we need to pack all stuffs which are required
    /// to make a `SignedMessage` except the sequence into a so-called `SigningMessage` struct and store it for
    /// later pushing.
    pub trait MessagePrepareChannel {
        type Signer;

        /// Like push_data but returns the SigningMessage rather than pushes it into the egress queue.
        fn prepare_with_data(&self, data: alloc::vec::Vec<u8>, to: impl Into<Path>) -> SigningMessage<Self::Signer>;
        /// Like push_message_to but returns the SigningMessage rather than pushes it into the egress queue.
        fn prepare_message_to(&self, message: &impl Encode, to: impl Into<Path>) -> SigningMessage<Self::Signer> {
            self.prepare_with_data(message.encode(), to)
        }
        /// Like push_message but returns the SigningMessage rather than pushes it into the egress queue.
        fn prepare_message<M: Encode + BindTopic>(&self, message: &M) -> SigningMessage<Self::Signer> {
            self.prepare_message_to(message, M::topic())
        }
    }
}

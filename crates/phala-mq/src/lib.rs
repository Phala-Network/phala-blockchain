#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "sgx")]
pub extern crate serde_sgx as serde;

extern crate alloc;

#[cfg(feature = "signers")]
pub use phala_mq_derive::MessageHashing;

#[cfg(not(feature = "signers"))]
pub use phala_mq_derive::DummyMessageHashing as MessageHashing;

pub use traits::MessageHashing;

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

    use crate::{BindTopic, MqHash, Path};

    #[cfg(feature = "signers")]
    use crate::SigningMessage;

    pub trait MessageHashing {
        fn hash(&self) -> MqHash;
    }

    pub trait MessageChannelBase {
        fn last_hash(&self) -> MqHash;
    }

    /// A MessageChannel is used to push messages into the egress queue, then the messages
    /// are ready to be synchronized to the chain by pherry or prb.
    pub trait MessageChannel: MessageChannelBase {
        /// Push given binary data as message payload into the egress queue.
        fn push_data(&self, data: alloc::vec::Vec<u8>, topic: impl Into<Path>, hash: MqHash);
        /// Same as push_data, except that it a SCALE encodable typed message which will be encoded into binary data.
        fn push_message_to(&self, message: &impl Encode, topic: impl Into<Path>, hash: MqHash) {
            self.push_data(message.encode(), topic, hash)
        }
        /// Same as push_message_to, except that the type of message is bound on a topic.
        fn push_message<M: Encode + BindTopic + MessageHashing>(&self, message: &M) {
            let hash = message.hash();
            self.push_message_to(message, M::topic(), hash)
        }
        fn set_dummy(&self, _dummy: bool) {}
    }

    /// A MessagePrepareChannel is used prepare messages which later can be pushed into the message queue.
    ///
    /// The purpose of this extra step is that sometimes(side-task e.g.) we need store pre-generated messages
    /// somewhere and push it later. But the final `SignedMessage` contains the sequence which must be signed can
    /// not be known in advance until it about to be pushed out. So we need to pack all stuffs which are required
    /// to make a `SignedMessage` except the sequence into a so-called `SigningMessage` struct and store it for
    /// later pushing.
    #[cfg(feature = "signers")]
    pub trait MessagePrepareChannel: MessageChannelBase {
        type Signer;

        /// Like push_data but returns the SigningMessage rather than pushes it into the egress queue.
        fn prepare_with_data(
            &self,
            data: alloc::vec::Vec<u8>,
            to: impl Into<Path>,
        ) -> SigningMessage<Self::Signer>;
        /// Like push_message_to but returns the SigningMessage rather than pushes it into the egress queue.
        fn prepare_message_to(
            &self,
            message: &impl Encode,
            to: impl Into<Path>,
        ) -> SigningMessage<Self::Signer> {
            self.prepare_with_data(message.encode(), to)
        }
        /// Like push_message but returns the SigningMessage rather than pushes it into the egress queue.
        fn prepare_message<M: Encode + BindTopic + MessageHashing>(
            &self,
            message: &M,
        ) -> SigningMessage<Self::Signer> {
            self.prepare_message_to(message, M::topic())
        }
    }
}

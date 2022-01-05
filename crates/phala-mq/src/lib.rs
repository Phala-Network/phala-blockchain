#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "sgx")]
pub extern crate serde_sgx as serde;

extern crate alloc;

#[cfg(feature = "queue")]
pub use phala_mq_derive::MessageHashing;

#[cfg(not(feature = "queue"))]
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
pub use send_queue::{MessageChannel, MessageSendQueue, SequenceInfo};
#[cfg(any(feature = "queue", feature = "dispatcher"))]
pub use simple_mpsc::{ReceiveError, Receiver};

#[cfg(feature = "queue")]
pub use signer::signers::MessageSigner;

pub use types::*;

// TODO.kevin: use std::sync::Mutex instead.
// See:
//    https://matklad.github.io/2020/01/02/spinlocks-considered-harmful.html
//    https://matklad.github.io/2020/01/04/mutexes-are-faster-than-spinlocks.html
#[cfg(feature = "queue")]
use spin::mutex::Mutex;

#[cfg(feature = "queue")]
pub use alias::*;

#[cfg(feature = "queue")]
mod alias {
    pub use crate::signer::signers::Sr25519Signer;
    pub use crate::MessageChannel as SignedMessageChannel;
}

pub mod traits {
    use parity_scale_codec::Encode;

    use crate::{BindTopic, MqHash, Path};

    #[cfg(feature = "queue")]
    use crate::Message;

    pub trait MessageHashing {
        fn hash(&self) -> MqHash;
    }

    /// A MessageChannel is used to push messages into the egress queue, then the messages
    /// are ready to be synchronized to the chain by pherry or prb.
    pub trait MessageChannel {
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
        /// Make an appointment for the next message.
        ///
        /// Return the sequence of the message.
        fn make_appointment(&self) -> Option<u64> {
            None
        }
    }

    /// A MessagePreparing is used to prepare messages which later can be pushed into the message queue.
    ///
    #[cfg(feature = "queue")]
    pub trait MessagePreparing {
        /// Like push_data but returns the Message rather than pushes it into the egress queue.
        fn prepare_with_data(&self, data: alloc::vec::Vec<u8>, to: impl Into<Path>) -> Message;

        /// Like push_message_to but returns the Message rather than pushes it into the egress queue.
        fn prepare_message_to(&self, message: &impl Encode, to: impl Into<Path>) -> Message {
            self.prepare_with_data(message.encode(), to)
        }

        /// Like push_message but returns the Message rather than pushes it into the egress queue.
        fn prepare_message<M: Encode + BindTopic>(&self, message: &M) -> Message {
            self.prepare_message_to(message, M::topic())
        }
    }
}

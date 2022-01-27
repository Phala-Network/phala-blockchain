#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use ink_env::{emit_event, topics::state::HasRemainingTopics, Environment, Topics};

use scale::{Decode, Encode};

#[cfg(feature = "runtime_utils")]
use ::{ink_env::test::EmittedEvent, std::convert::TryInto};

pub use pink_extension_macro::contract;

pub mod chain_extension;

const PINK_EVENT_TOPIC: &[u8] = b"phala.pink.event";

pub type EcdhPublicKey = [u8; 32];
pub type Hash = [u8; 32];

#[derive(Encode, Decode, Debug)]
pub struct Message {
    pub payload: Vec<u8>,
    pub topic: Vec<u8>,
}

#[derive(Encode, Decode, Debug)]
pub struct OspMessage {
    pub message: Message,
    pub remote_pubkey: Option<EcdhPublicKey>,
}

#[derive(Encode, Decode, Debug)]
pub enum PinkEvent {
    /// Contract pushed a raw message
    Message(Message),
    /// Contract pushed an osp message
    OspMessage(OspMessage),
    /// Contract has an on_block_end ink message and will emit this event on instantiation.
    OnBlockEndSelector(u32),
}

impl Topics for PinkEvent {
    type RemainingTopics = [HasRemainingTopics; 1];

    fn topics<E, B>(
        &self,
        builder: ink_env::topics::TopicsBuilder<ink_env::topics::state::Uninit, E, B>,
    ) -> <B as ink_env::topics::TopicsBuilderBackend<E>>::Output
    where
        E: Environment,
        B: ink_env::topics::TopicsBuilderBackend<E>,
    {
        builder
            .build::<Self>()
            .push_topic(&PINK_EVENT_TOPIC)
            .finish()
    }
}

#[cfg(feature = "runtime_utils")]
impl PinkEvent {
    pub fn event_topic() -> Hash {
        topics_for(Self::OnBlockEndSelector(0))[0]
    }
}

/// Push a raw message to a topic accepting only vanilla messages
///
/// Most phala system topics accept vanilla messages
pub fn push_message(payload: Vec<u8>, topic: Vec<u8>) {
    emit_event::<PinkEnvironment, _>(PinkEvent::Message(Message { payload, topic }))
}

/// Push a message to a topic accepting optinal secret messages
///
/// Contract commands topic accept osp messages
pub fn push_osp_message(payload: Vec<u8>, topic: Vec<u8>, remote_pubkey: Option<EcdhPublicKey>) {
    emit_event::<PinkEnvironment, _>(PinkEvent::OspMessage(OspMessage {
        message: Message { payload, topic },
        remote_pubkey,
    }))
}

/// Turn on on_block_end feature and set it's selector
///
pub fn set_on_block_end_selector(selector: u32) {
    emit_event::<PinkEnvironment, _>(PinkEvent::OnBlockEndSelector(selector))
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum PinkEnvironment {}

impl Environment for PinkEnvironment {
    const MAX_EVENT_TOPICS: usize = <ink_env::DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

    type AccountId = <ink_env::DefaultEnvironment as Environment>::AccountId;
    type Balance = <ink_env::DefaultEnvironment as Environment>::Balance;
    type Hash = <ink_env::DefaultEnvironment as Environment>::Hash;
    type BlockNumber = <ink_env::DefaultEnvironment as Environment>::BlockNumber;
    type Timestamp = <ink_env::DefaultEnvironment as Environment>::Timestamp;

    type ChainExtension = chain_extension::PinkExt;
}

#[cfg(feature = "runtime_utils")]
fn topics_for(event: impl Topics + Encode) -> Vec<Hash> {
    EmittedEvent::new::<PinkEnvironment, _>(event)
        .topics
        .into_iter()
        .map(|hash| {
            hash.encoded_bytes()
                .expect("Never fail")
                .try_into()
                .expect("Never fail")
        })
        .collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_event_topics() {
        insta::assert_debug_snapshot!(super::PinkEvent::event_topic());
    }
}

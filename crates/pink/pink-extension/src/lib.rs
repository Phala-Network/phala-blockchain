#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use std::convert::TryInto;

use alloc::vec::Vec;
use ink_env::{
    emit_event, test::EmittedEvent, topics::state::HasRemainingTopics, Environment, Topics,
};
use scale::{Decode, Encode};

const PHALA_MESSAGE_TOPIC: &[u8] = b"phala.mq.message";
const PHALA_OSP_MESSAGE_TOPIC: &[u8] = b"phala.mq.osp_message";

pub type EcdhPublicKey = [u8; 32];
pub type Hash = [u8; 32];

#[derive(Encode, Decode, Default, Debug)]
pub struct Message {
    pub payload: Vec<u8>,
    pub topic: Vec<u8>,
}

#[derive(Encode, Decode, Default, Debug)]
pub struct OspMessage {
    pub message: Message,
    pub remote_pubkey: Option<EcdhPublicKey>,
}

impl Topics for Message {
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
            .push_topic(&PHALA_MESSAGE_TOPIC)
            .finish()
    }
}

impl Message {
    pub fn event_topic() -> Hash {
        topics_for(Self::default())[0]
    }
}

impl Topics for OspMessage {
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
            .push_topic(&PHALA_OSP_MESSAGE_TOPIC)
            .finish()
    }
}

impl OspMessage {
    pub fn event_topic() -> Hash {
        topics_for(Self::default())[0]
    }
}

/// Push a raw message to a topic accepting only vanilla messages
///
/// Most phala system topics accept vanilla messages
pub fn push_message(payload: Vec<u8>, topic: Vec<u8>) {
    emit_event::<PinkEnvironment, _>(Message { payload, topic })
}

/// Push a message to a topic accepting optinal secret messages
///
/// Contract commands topic accept osp messages
pub fn push_osp_message(payload: Vec<u8>, topic: Vec<u8>, remote_pubkey: Option<EcdhPublicKey>) {
    emit_event::<PinkEnvironment, _>(OspMessage {
        message: Message { payload, topic },
        remote_pubkey,
    })
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
    type RentFraction = <ink_env::DefaultEnvironment as Environment>::RentFraction;

    type ChainExtension = <ink_env::DefaultEnvironment as Environment>::ChainExtension;
}

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
        insta::assert_debug_snapshot!(super::Message::event_topic());
        insta::assert_debug_snapshot!(super::OspMessage::event_topic());
    }
}

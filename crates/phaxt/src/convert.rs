mod gen {
    pub use crate::khala::runtime_types::phala_mq::types::*;
    pub use crate::khala::runtime_types::sp_core::sr25519::*;
}
use phala_types::messaging::*;

impl From<MessageOrigin> for gen::MessageOrigin {
    fn from(other: MessageOrigin) -> Self {
        match other {
            MessageOrigin::Pallet(v) => Self::Pallet(v),
            MessageOrigin::Contract(v) => Self::Contract(v),
            MessageOrigin::Worker(v) => Self::Worker(gen::Public(v.0)),
            MessageOrigin::AccountId(v) => Self::AccountId(v),
            MessageOrigin::MultiLocation(v) => Self::MultiLocation(v),
            MessageOrigin::Gatekeeper => Self::Gatekeeper,
        }
    }
}

impl From<Message> for gen::Message {
    fn from(other: Message) -> Self {
        Self {
            sender: other.sender.into(),
            destination: gen::Topic(other.destination.into()),
            payload: other.payload.into(),
        }
    }
}

impl From<SignedMessage> for gen::SignedMessage {
    fn from(other: SignedMessage) -> Self {
        Self {
            message: other.message.into(),
            sequence: other.sequence,
            signature: other.signature,
        }
    }
}

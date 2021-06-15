use alloc::vec::Vec;

#[cfg(feature = "scale-codec")]
use parity_scale_codec::{Decode, Encode};

#[cfg(any(feature = "serde", feature = "serde_sgx"))]
use serde::{Deserialize, Serialize};

pub type Path = Vec<u8>;
pub type SenderId = Vec<u8>;

/// The origin of a Phala message
// TODO: should we use XCM MultiLocation directly?
// [Reference](https://github.com/paritytech/xcm-format#multilocation-universal-destination-identifiers)
#[cfg_attr(any(feature = "serde", feature = "serde_sgx"), derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde_sgx", serde(crate="serde"))]
#[cfg_attr(feature = "scale-codec", derive(Encode, Decode))]
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Origin {
    /// Runtime pallets (identified by pallet name)
    Pallet(Vec<u8>),
    /// A confidential contract
    Contract(H256),
    /// A pRuntime worker
    Worker(Vec<u8>),
    /// A user
    AccountId(H256),
    /// A remote location (parachain, etc.)
    Multilocaiton(Vec<u8>),
}

impl Origin {
    /// Builds a new native confidential contract `MessageOrigin`
    #[cfg(feature = "scale-codec")]
    pub fn native_contract(id: u32) -> Self {
        Self::Contract(id.encode())
    }

    /// Returns if the origin is located off-chain
    pub fn is_offchain(&self) -> bool {
        match self {
            Self::Contract(_) | Self::Worker(_) => true,
            _ => false,
        }
    }
}

/// The topic in the message queue, indicating a group of destination message receivers
#[cfg_attr(any(feature = "serde", feature = "serde_sgx"), derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde_sgx", serde(crate="serde"))]
#[cfg_attr(feature = "scale-codec", derive(Encode, Decode))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Topic {
    /// The topic targets a cetrain receiver identified by `MessageOrigin`
    Targeted(MessageOrigin),
    /// A general topic that can be subscribed by anyone
    Named(Vec<u8>),
}

impl Topic {
    pub fn is_offchain(&self) -> bool {
        if let Topic::Targeted(origin) = self {
            origin.is_offchain()
        } else {
            false
        }
    }
}

#[cfg_attr(any(feature = "serde", feature = "serde_sgx"), derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde_sgx", serde(crate="serde"))]
#[cfg_attr(feature = "scale-codec", derive(Encode, Decode))]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Message {
    pub sender: SenderId,
    pub destination: Path,
    pub payload: Vec<u8>,
}

impl Message {
    pub fn new(
        sender: impl Into<SenderId>,
        destination: impl Into<Path>,
        payload: Vec<u8>,
    ) -> Self {
        Message {
            sender: sender.into(),
            destination: destination.into(),
            payload,
        }
    }

    #[cfg(feature = "scale-codec")]
    pub fn sender(&self) -> Option<Origin> {
        Decode::decode(&mut &self.sender[..]).ok()
    }

    #[cfg(feature = "scale-codec")]
    pub fn decode_payload<T: Decode>(&self) -> Option<T> {
        Decode::decode(&mut &self.payload[..]).ok()
    }
}

#[cfg_attr(any(feature = "serde", feature = "serde_sgx"), derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde_sgx", serde(crate="serde"))]
#[cfg_attr(feature = "scale-codec", derive(Encode, Decode))]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SignedMessage {
    pub message: Message,
    pub sequence: u64,
    pub signature: Vec<u8>,
}

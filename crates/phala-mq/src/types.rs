use alloc::string::String;
use alloc::vec::Vec;
use core::hash::{Hash, Hasher};

use derive_more::Display;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_core::crypto::{AccountId32, UncheckedFrom};

pub type Path = Vec<u8>;
pub type SenderId = MessageOrigin;
pub use sp_core::H256 as ContractId;
pub use sp_core::H256 as AccountId;
pub use sp_core::H256 as ContractClusterId;

use crate::MessageSigner;
use phala_serde_more as more;
use serde::{Deserialize, Serialize};

/// The origin of a Phala message
// TODO: should we use XCM MultiLocation directly?
// [Reference](https://github.com/paritytech/xcm-format#multilocation-universal-destination-identifiers)
#[derive(Encode, Decode, TypeInfo, Debug, Clone, Eq, PartialOrd, Ord, Display)]
#[derive(Serialize, Deserialize)]
pub enum MessageOrigin {
    /// Runtime pallets (identified by pallet name)
    #[display(fmt = "Pallet(\"{}\")", "String::from_utf8_lossy(_0)")]
    #[serde(with = "more::scale_bytes")]
    Pallet(Vec<u8>),
    /// A contract cluster
    #[display(fmt = "Cluster({})", "hex::encode(_0)")]
    #[serde(with = "more::scale_bytes")]
    Cluster(ContractClusterId),
    /// A confidential contract
    #[display(fmt = "Contract({})", "hex::encode(_0)")]
    #[serde(with = "more::scale_bytes")]
    Contract(ContractId),
    /// A pRuntime worker
    #[display(fmt = "Worker({})", "hex::encode(_0)")]
    #[serde(with = "more::scale_bytes")]
    Worker(sp_core::sr25519::Public),
    /// A user
    #[display(fmt = "AccountId({})", "hex::encode(_0)")]
    #[serde(with = "more::scale_bytes")]
    AccountId(AccountId),
    /// A remote location (parachain, etc.)
    #[display(fmt = "MultiLocation({})", "hex::encode(_0)")]
    #[serde(with = "more::scale_bytes")]
    MultiLocation(Vec<u8>),
    /// All gatekeepers share the same origin
    Gatekeeper,
}

impl Hash for MessageOrigin {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let encoded = Encode::encode(self);
        encoded.hash(state);
    }
}

// PartialEq must agree with Hash.
// See: https://rust-lang.github.io/rust-clippy/master/index.html#derive_hash_xor_eq
impl PartialEq for MessageOrigin {
    fn eq(&self, other: &Self) -> bool {
        let encoded_self = Encode::encode(self);
        let encoded_other = Encode::encode(other);
        encoded_self == encoded_other
    }
}

impl MessageOrigin {
    /// Returns if the origin is located off-chain
    pub fn is_offchain(&self) -> bool {
        matches!(
            self,
            Self::Cluster(_) | Self::Contract(_) | Self::Worker(_) | Self::Gatekeeper
        )
    }

    /// Returns if the origin is from a Pallet
    pub fn is_pallet(&self) -> bool {
        matches!(self, Self::Pallet(_))
    }

    /// Returns if the origin is from a Gatekeeper
    pub fn is_gatekeeper(&self) -> bool {
        matches!(self, Self::Gatekeeper)
    }

    /// Returns the account id if the origin is from a user, or `Err(BadOrigin)` otherwise
    pub fn account(&self) -> Result<AccountId32, BadOrigin> {
        match self {
            Self::AccountId(account_id) => Ok(AccountId32::unchecked_from(account_id.clone())),
            _ => Err(BadOrigin),
        }
    }
}

pub struct BadOrigin;

/// The topic in the message queue, indicating a group of destination message receivers.
///
/// A topic can be any non-empty binary string except there are some reserved value for the first byte.
///
/// # The reserved values for the first byte:
///
/// ~!@#$%&*_+-=|<>?,./;:'
///
/// # Indicator byte
///  Meaning of some special values appearing at the first byte:
///
///  - b'^': The topic's subscribers are on-chain only.
///
/// # Example:
/// ```rust
///    use phala_mq::Topic;
///
///    // An on-chain only topic. Messages sent to this topic will not be dispatched
///    // to off-chain components.
///    let an_onchain_topic = Topic::new(*b"^topic path");
///    assert!(!an_onchain_topic.is_offchain());
///
///    // An normal topic. Messages sent to this topic will be dispatched to off-chain subscribers
///    // as well as on-chain ones.
///    let a_normal_topic = Topic::new(*b"topic path");
///    assert!(a_normal_topic.is_offchain());
/// ```
///
#[derive(Encode, Decode, TypeInfo, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Topic(#[serde(with = "more::scale_bytes")] Path);

impl core::fmt::Debug for Topic {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let repr = alloc::string::String::from_utf8_lossy(&self.0[..]);
        f.write_str(repr.as_ref())
    }
}

impl Topic {
    const RESERVED_BYTES: &'static [u8] = b"~!@#$%&*_+-=|<>?,./;:'";

    pub fn new(path: impl Into<Path>) -> Self {
        Self(path.into())
    }

    pub fn path(&self) -> &Path {
        &self.0
    }

    pub fn is_offchain(&self) -> bool {
        if !self.is_valid() {
            return false;
        }
        self.0[0] != b'^'
    }

    pub fn is_valid(&self) -> bool {
        if self.0.is_empty() {
            return false;
        }
        !Self::RESERVED_BYTES.contains(&self.0[0])
    }
}

impl From<Path> for Topic {
    fn from(path: Path) -> Self {
        Self::new(path)
    }
}

impl From<Topic> for Path {
    fn from(topic: Topic) -> Self {
        topic.0
    }
}

/// Messages implementing BindTopic can be sent without giving the destination.
pub trait BindTopic {
    fn topic() -> Path;
}

impl BindTopic for () {
    fn topic() -> Path {
        Vec::new()
    }
}

/// Indicates the type is a contract command
pub trait ContractCommand {
    fn contract_id() -> ContractId;
}

#[macro_export]
macro_rules! bind_topic {
    ($t: ident, $path: expr) => {
        impl $crate::types::BindTopic for $t {
            fn topic() -> Vec<u8> {
                $path.to_vec()
            }
        }
    };
    ($t: ident<$($gt: ident),+>, $path: expr) => {
        impl<$($gt),+> $crate::types::BindTopic for $t<$($gt),+> {
            fn topic() -> Vec<u8> {
                $path.to_vec()
            }
        }
    }
}

#[derive(Encode, Decode, TypeInfo, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub sender: SenderId,
    pub destination: Topic,
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
            destination: Topic::new(destination),
            payload,
        }
    }

    pub fn decode_payload<T: Decode>(&self) -> Option<T> {
        Decode::decode(&mut &self.payload[..]).ok()
    }

    pub fn decode<T: Decode>(&self) -> Option<DecodedMessage<T>> {
        let payload = Decode::decode(&mut &self.payload[..]).ok()?;
        Some(DecodedMessage {
            sender: self.sender.clone(),
            destination: self.destination.clone(),
            payload,
        })
    }
}

pub struct DecodedMessage<T> {
    pub sender: SenderId,
    pub destination: Topic,
    pub payload: T,
}

#[derive(Encode, Decode, TypeInfo, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SignedMessage {
    pub message: Message,
    pub sequence: u64,
    pub signature: Vec<u8>,
}

impl SignedMessage {
    pub fn data_be_signed(&self) -> Vec<u8> {
        MessageToBeSigned {
            message: &self.message,
            sequence: self.sequence,
        }
        .raw_data()
    }
}

#[derive(Encode)]
pub(crate) struct MessageToBeSigned<'a> {
    pub(crate) message: &'a Message,
    pub(crate) sequence: u64,
}

impl<'a> MessageToBeSigned<'a> {
    pub(crate) fn raw_data(&self) -> Vec<u8> {
        self.encode()
    }
}

#[derive(Encode, Decode, Debug, Clone, Serialize, Deserialize)]
pub struct SigningMessage<Signer> {
    pub message: Message,
    pub signer: Signer,
}

impl<Signer: MessageSigner> SigningMessage<Signer> {
    pub fn sign(self, sequence: u64) -> SignedMessage {
        let data = MessageToBeSigned {
            message: &self.message,
            sequence,
        };
        let signature = self.signer.sign(&data.raw_data());
        SignedMessage {
            message: self.message,
            sequence,
            signature,
        }
    }
}

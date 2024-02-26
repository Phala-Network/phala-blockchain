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
#[derive(
    Encode, Decode, TypeInfo, Debug, Clone, Eq, PartialOrd, Ord, Display, Serialize, Deserialize,
)]
pub enum MessageOrigin {
    /// Runtime pallets (identified by pallet name)
    #[display(fmt = "Pallet(\"{}\")", "String::from_utf8_lossy(_0)")]
    #[serde(with = "more::scale_bytes")]
    Pallet(Vec<u8>),
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
    /// A contract cluster
    #[display(fmt = "Cluster({})", "hex::encode(_0)")]
    #[serde(with = "more::scale_bytes")]
    Cluster(ContractClusterId),
    /// Reserved, we use this prefix to indicate the signed content is not a mq message
    #[codec(index = 255)]
    Reserved,
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

    /// Returns if we can trust the origin to not send us non-well-formed messages
    pub fn always_well_formed(&self) -> bool {
        matches!(self, Self::Pallet(_) | Self::Worker(_) | Self::Gatekeeper)
    }

    /// Returns if the origin is from a Gatekeeper
    pub fn is_gatekeeper(&self) -> bool {
        matches!(self, Self::Gatekeeper)
    }

    /// Returns the account id if the origin is from a user, or `Err(BadOrigin)` otherwise
    pub fn account(&self) -> Result<AccountId32, BadOrigin> {
        match self {
            Self::AccountId(account_id) => Ok(AccountId32::unchecked_from(*account_id)),
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
        Some(DecodedMessage {
            sender: self.sender.clone(),
            destination: self.destination.clone(),
            payload: self.decode_payload()?,
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

#[derive(Encode, Decode, TypeInfo, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

#[cfg(test)]
mod test {
    use core::convert::TryInto;

    use crate::Sr25519Signer;

    use super::*;

    #[test]
    fn test_topic() {
        let topic = Topic::new(*b"topic");
        assert!(topic.is_valid());
        assert!(topic.is_offchain());
        assert_eq!(topic.path(), b"topic");

        let topic = Topic::new(*b"^topic");
        assert!(topic.is_valid());
        assert!(!topic.is_offchain());

        let topic = Topic::new(*b"");
        assert!(!topic.is_valid());
        assert!(!topic.is_offchain());

        println!("topic: {topic:?}");
    }

    #[test]
    fn test_origin() {
        use sp_core::sr25519::Public;

        let origin = MessageOrigin::Pallet(b"pallet".to_vec());
        assert!(origin.is_pallet());
        assert!(!origin.is_offchain());
        assert!(origin.account().is_err());
        assert!(!origin.is_gatekeeper());
        assert!(origin.always_well_formed());

        let origin = MessageOrigin::AccountId(AccountId::default());
        assert!(!origin.is_pallet());
        assert!(!origin.is_offchain());
        assert!(origin.account().is_ok());

        assert!(MessageOrigin::Gatekeeper.always_well_formed());
        assert!(MessageOrigin::Worker(Public(Default::default())).always_well_formed());
        assert!(!MessageOrigin::Contract(Default::default()).always_well_formed());
    }

    #[test]
    fn test_topic_of_tuple() {
        assert_eq!(<() as BindTopic>::topic(), b"");
    }

    #[test]
    fn test_topic_convert() {
        let topic = Topic::new(*b"topic");
        let path: Path = topic.into();
        assert_eq!(path, b"topic");
        let topic: Topic = path.into();
        assert_eq!(topic, Topic::new(*b"topic"));
    }

    #[test]
    fn test_message_signing() {
        use sp_core::sr25519::{Pair, Signature};
        use sp_core::Pair as _;
        let pair = Pair::from_seed(b"12345678901234567890123456789012");
        let pubkey = pair.public();
        let signer = Sr25519Signer::from(pair);
        let message = Message::new(MessageOrigin::Gatekeeper, *b"topic", b"payload".to_vec());
        let signing_message = SigningMessage { message, signer };
        let signed_message = signing_message.sign(0);
        assert_eq!(signed_message.message.payload, b"payload");
        assert_eq!(signed_message.sequence, 0);
        let signed_payload = signed_message.data_be_signed();
        assert!(Pair::verify(
            &Signature(signed_message.signature.try_into().unwrap()),
            signed_payload,
            &pubkey
        ));
    }

    #[test]
    fn test_message_decode() {
        let payload = "payload".encode();
        let message = Message::new(MessageOrigin::Gatekeeper, *b"topic", payload);
        let decoded_message = message.decode::<String>().unwrap();
        assert_eq!(decoded_message.payload, "payload");
    }

    #[test]
    fn test_hash_of_origin() {
        use std::collections::HashMap;
        let origin = MessageOrigin::Pallet(b"pallet".to_vec());
        let mut map = HashMap::<MessageOrigin, ()>::new();
        map.insert(origin.clone(), ());
        assert!(map.contains_key(&origin));
    }

    #[test]
    fn bind_topic_works() {
        bind_topic!(Foo<T>, b"foo");
        struct Foo<T>(Option<T>);
        bind_topic!(Bar, b"bar");
        struct Bar;

        assert_eq!(Foo::<u32>::topic(), b"foo");
        assert_eq!(Foo::<u64>::topic(), b"foo");
        assert_eq!(Bar::topic(), b"bar");
    }

    #[test]
    fn test_codecs() {
        #[derive(Encode, Decode, TypeInfo, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
        struct TestSuite {
            origin: MessageOrigin,
            signed_message: SignedMessage,
            signing_message: SigningMessage<()>,
            topic: Topic,
        }

        let suite = TestSuite {
            origin: MessageOrigin::Reserved,
            signed_message: SignedMessage {
                message: Message::new(MessageOrigin::Reserved, *b"topic", vec![]),
                sequence: 0,
                signature: vec![],
            },
            signing_message: SigningMessage {
                message: Message::new(MessageOrigin::Reserved, *b"topic", vec![]),
                signer: (),
            },
            topic: Topic::new(*b"topic"),
        };
        let cloned = suite.clone();
        let encoded = Encode::encode(&cloned);
        let decoded = TestSuite::decode(&mut &encoded[..]).unwrap();
        assert_eq!(decoded, suite);
        let serialized = serde_cbor::to_vec(&cloned).unwrap();
        let deserialzed: TestSuite = serde_cbor::from_slice(&serialized[..]).unwrap();
        assert_eq!(deserialzed, suite);

        insta::assert_display_snapshot!(type_info_stringify::type_info_stringify::<TestSuite>())
    }

    #[test]
    fn test_display_message_origin() {
        assert_eq!(
            format!("{}", MessageOrigin::Pallet(b"test".to_vec())),
            "Pallet(\"test\")"
        );
        assert_eq!(format!("{}", MessageOrigin::Gatekeeper), "Gatekeeper");
        assert_eq!(
            format!("{}", MessageOrigin::AccountId([0u8; 32].into())),
            "AccountId(0000000000000000000000000000000000000000000000000000000000000000)"
        );
        assert_eq!(format!("{}", MessageOrigin::Contract([0u8; 32].into())), "Contract(0000000000000000000000000000000000000000000000000000000000000000)");
        assert_eq!(
            format!(
                "{}",
                MessageOrigin::Worker(sp_core::sr25519::Public([0u8; 32]))
            ),
            "Worker(0000000000000000000000000000000000000000000000000000000000000000)"
        );
        assert_eq!(format!("{}", MessageOrigin::MultiLocation(vec![0u8])), "MultiLocation(00)");
        assert_eq!(format!("{}", MessageOrigin::Cluster([0u8; 32].into())), "Cluster(0000000000000000000000000000000000000000000000000000000000000000)");
        assert_eq!(format!("{}", MessageOrigin::Reserved), "Reserved");
    }
}

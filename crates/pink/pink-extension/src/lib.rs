#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(alloc_error_handler))]

extern crate alloc;

use alloc::vec::Vec;
use ink::env::{emit_event, topics::state::HasRemainingTopics, Environment, Topics};

use ink::EnvAccess;
use scale::{Decode, Encode};

pub use pink_extension_macro::contract;

pub mod chain_extension;
pub use chain_extension::pink_extension_instance as ext;
pub mod logger;
pub mod system;

#[cfg(all(not(feature = "std"), feature = "dlmalloc"))]
mod allocator_dlmalloc;

pub use logger::ResultExt;

const PINK_EVENT_TOPIC: &[u8] = b"phala.pink.event";

pub type EcdhPublicKey = [u8; 32];
pub type Hash = [u8; 32];
pub type EcdsaPublicKey = [u8; 33];
pub type EcdsaSignature = [u8; 65];
pub type AccountId = <PinkEnvironment as Environment>::AccountId;
pub type Balance = <PinkEnvironment as Environment>::Balance;
pub type BlockNumber = <PinkEnvironment as Environment>::BlockNumber;

pub trait ConvertTo<To> {
    fn convert_to(&self) -> To;
}

impl<F, T> ConvertTo<T> for F
where
    F: AsRef<[u8; 32]>,
    T: From<[u8; 32]>,
{
    fn convert_to(&self) -> T {
        (*self.as_ref()).into()
    }
}

/// A phala-mq message
#[derive(Encode, Decode, Debug)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct Message {
    pub payload: Vec<u8>,
    pub topic: Vec<u8>,
}

/// A phala-mq message with optional encryption key
#[derive(Encode, Decode, Debug)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct OspMessage {
    pub message: Message,
    pub remote_pubkey: Option<EcdhPublicKey>,
}

#[derive(Encode, Decode, Debug)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum HookPoint {
    OnBlockEnd,
}

/// System Event used to communicate between the contract and the runtime.
#[derive(Encode, Decode, Debug)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum PinkEvent {
    /// Contract pushed a raw message
    Message(Message),
    /// Contract pushed an osp message
    OspMessage(OspMessage),
    /// Set contract hook
    SetHook {
        /// The event to hook
        hook: HookPoint,
        /// The target contract address
        contract: AccountId,
        /// The selector to invoke on hooked event fired.
        selector: u32,
        /// The gas limit when calling the selector
        gas_limit: u64,
    },
    /// Deploy a sidevm instance to given contract instance
    DeploySidevmTo {
        /// The target contract address
        contract: AccountId,
        /// The hash of the sidevm code.
        code_hash: Hash,
    },
    /// Push a message to the associated sidevm instance.
    SidevmMessage(Vec<u8>),
    /// CacheOperation
    CacheOp(CacheOp),
    /// Stop the side VM instance if it is running.
    StopSidevm,
    /// Force stop the side VM instance if it is running.
    ForceStopSidevm {
        /// The target contract address
        contract: AccountId,
    },
    /// Set the log handler contract for current cluster.
    SetLogHandler(AccountId),
    /// Set the weight of contract used to schedule queries and sidevm vruntime
    SetContractWeight { contract: AccountId, weight: u32 },
    /// Upgrade the system contract to latest version.
    UpgradeSystemContract { storage_payer: AccountId },
}

impl PinkEvent {
    pub fn allowed_in_query(&self) -> bool {
        match self {
            PinkEvent::Message(_) => false,
            PinkEvent::OspMessage(_) => false,
            PinkEvent::SetHook { .. } => false,
            PinkEvent::DeploySidevmTo { .. } => true,
            PinkEvent::SidevmMessage(_) => true,
            PinkEvent::CacheOp(_) => true,
            PinkEvent::StopSidevm => true,
            PinkEvent::ForceStopSidevm { .. } => true,
            PinkEvent::SetLogHandler(_) => false,
            PinkEvent::SetContractWeight { .. } => false,
            PinkEvent::UpgradeSystemContract { .. } => false,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            PinkEvent::Message(_) => "Message",
            PinkEvent::OspMessage(_) => "OspMessage",
            PinkEvent::SetHook { .. } => "SetHook",
            PinkEvent::DeploySidevmTo { .. } => "DeploySidevmTo",
            PinkEvent::SidevmMessage(_) => "SidevmMessage",
            PinkEvent::CacheOp(_) => "CacheOp",
            PinkEvent::StopSidevm => "StopSidevm",
            PinkEvent::ForceStopSidevm { .. } => "ForceStopSidevm",
            PinkEvent::SetLogHandler(_) => "SetLogHandler",
            PinkEvent::SetContractWeight { .. } => "SetContractWeight",
            PinkEvent::UpgradeSystemContract { .. } => "UpgradeSystemContract",
        }
    }
}

#[derive(Encode, Decode, Debug)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum CacheOp {
    Set { key: Vec<u8>, value: Vec<u8> },
    SetExpiration { key: Vec<u8>, expiration: u64 },
    Remove { key: Vec<u8> },
}

impl Topics for PinkEvent {
    type RemainingTopics = [HasRemainingTopics; 1];

    fn topics<E, B>(
        &self,
        builder: ink::env::topics::TopicsBuilder<ink::env::topics::state::Uninit, E, B>,
    ) -> <B as ink::env::topics::TopicsBuilderBackend<E>>::Output
    where
        E: Environment,
        B: ink::env::topics::TopicsBuilderBackend<E>,
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
        use std::convert::TryFrom;
        let topics = topic::topics_for(Self::StopSidevm);
        let topic: &[u8] = topics[0].as_ref();
        Hash::try_from(topic).expect("Should not failed")
    }
}

/// Push a raw message to a topic accepting only vanilla messages
///
/// Most phala system topics accept vanilla messages
pub fn push_message(payload: Vec<u8>, topic: Vec<u8>) {
    emit_event::<PinkEnvironment, _>(PinkEvent::Message(Message { payload, topic }))
}

/// Push a message to a topic accepting optional secret messages
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
pub fn set_hook(hook: HookPoint, contract: AccountId, selector: u32, gas_limit: u64) {
    emit_event::<PinkEnvironment, _>(PinkEvent::SetHook {
        hook,
        contract,
        selector,
        gas_limit,
    })
}

/// Start a SideVM instance
pub fn start_sidevm(code_hash: Hash) -> Result<(), system::DriverError> {
    let driver =
        crate::system::SidevmOperationRef::instance().ok_or(system::Error::DriverNotFound)?;
    driver.deploy(code_hash)
}

/// Deploy a SideVM instance to a given contract.
/// The caller must be the system contract.
pub fn deploy_sidevm_to(contract: AccountId, code_hash: Hash) {
    emit_event::<PinkEnvironment, _>(PinkEvent::DeploySidevmTo {
        contract,
        code_hash,
    });
}

/// Stop a SideVM instance running at given contract address.
/// The caller must be the system contract.
pub fn stop_sidevm_at(contract: AccountId) {
    emit_event::<PinkEnvironment, _>(PinkEvent::ForceStopSidevm { contract });
}

/// Force stop the side VM instance if it is running
///
/// You should avoid to call this function. Instead, prefer let the side program exit gracefully
/// by itself.
pub fn force_stop_sidevm() {
    emit_event::<PinkEnvironment, _>(PinkEvent::StopSidevm)
}

/// Push a message to the associated sidevm instance.
pub fn push_sidevm_message(message: Vec<u8>) {
    emit_event::<PinkEnvironment, _>(PinkEvent::SidevmMessage(message))
}

/// Set the log handler contract of current cluster
pub fn set_log_handler(contract: AccountId) {
    emit_event::<PinkEnvironment, _>(PinkEvent::SetLogHandler(contract))
}

/// Set the weight of contract used to schedule queries and sidevm vruntime
pub fn set_contract_weight(contract: AccountId, weight: u32) {
    emit_event::<PinkEnvironment, _>(PinkEvent::SetContractWeight { contract, weight });
}

/// Upgrade the system contract to latest version
pub fn upgrade_system_contract(storage_payer: AccountId) {
    emit_event::<PinkEnvironment, _>(PinkEvent::UpgradeSystemContract { storage_payer });
}

/// Pink defined environment. Used this environment to access the fat contract runtime features.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum PinkEnvironment {}

impl Environment for PinkEnvironment {
    const MAX_EVENT_TOPICS: usize = <ink::env::DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

    type AccountId = <ink::env::DefaultEnvironment as Environment>::AccountId;
    type Balance = <ink::env::DefaultEnvironment as Environment>::Balance;
    type Hash = <ink::env::DefaultEnvironment as Environment>::Hash;
    type BlockNumber = <ink::env::DefaultEnvironment as Environment>::BlockNumber;
    type Timestamp = <ink::env::DefaultEnvironment as Environment>::Timestamp;

    type ChainExtension = chain_extension::PinkExt;
}

pub fn env() -> EnvAccess<'static, PinkEnvironment> {
    Default::default()
}

#[cfg(feature = "runtime_utils")]
mod topic;

#[cfg(test)]
mod tests {
    #[test]
    fn test_event_topics() {
        insta::assert_debug_snapshot!(super::PinkEvent::event_topic());
    }
}

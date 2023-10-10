#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(alloc_error_handler))]
#![doc = include_str!("../README.md")]

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

/// Hook points defined in the runtime.
#[derive(Encode, Decode, Debug, Clone)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum HookPoint {
    /// When all events in a block are processed.
    OnBlockEnd,
}

/// System Event used to communicate between the contract and the runtime.
#[derive(Encode, Decode, Debug, Clone)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum PinkEvent {
    /// Set contract hook
    ///
    /// Please do not use this event directly, use [`set_hook()`] instead.
    ///
    /// # Availability
    /// System contract
    #[codec(index = 2)]
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
    ///
    /// Please do not use this event directly, use [`deploy_sidevm_to()`] instead.
    ///
    /// # Availability
    /// System contract
    #[codec(index = 3)]
    DeploySidevmTo {
        /// The target contract address
        contract: AccountId,
        /// The hash of the sidevm code.
        code_hash: Hash,
    },
    /// Push a message to the associated sidevm instance.
    ///
    /// Please do not use this event directly, use [`push_sidevm_message()`] instead.
    ///
    /// # Availability
    /// Any contract
    #[codec(index = 4)]
    SidevmMessage(Vec<u8>),
    /// Instructions to manipulate the cache. Including set, remove and set expiration.
    ///
    /// # Availability
    /// Any contract
    #[codec(index = 5)]
    CacheOp(CacheOp),
    /// Stop the side VM instance associated with the caller contract if it is running.
    ///
    /// Please do not use this event directly, use [`force_stop_sidevm()`] instead.
    ///
    /// # Availability
    /// Any contract
    #[codec(index = 6)]
    StopSidevm,
    /// Force stop the side VM instance associated with the given contract if it is running.
    ///
    /// Please do not use this event directly, use [`stop_sidevm_at()`] instead.
    ///
    /// # Availability
    /// System contract
    #[codec(index = 7)]
    ForceStopSidevm {
        /// The target contract address
        contract: AccountId,
    },
    /// Set the log handler contract for current cluster.
    ///
    /// Please do not use this event directly, use [`set_log_handler()`] instead.
    ///
    /// # Availability
    /// System contract
    #[codec(index = 8)]
    SetLogHandler(AccountId),
    /// Set the weight of contract used to schedule queries and sidevm virtual runtime
    ///
    /// Please do not use this event directly, use [`set_contract_weight()`] instead.
    ///
    /// # Availability
    /// System contract
    #[codec(index = 9)]
    SetContractWeight { contract: AccountId, weight: u32 },
    /// Upgrade the runtime to given version
    ///
    /// Please do not use this event directly, use [`upgrade_runtime()`] instead.
    ///
    /// # Availability
    /// System contract
    #[codec(index = 10)]
    UpgradeRuntimeTo { version: (u32, u32) },
}

impl PinkEvent {
    pub fn allowed_in_query(&self) -> bool {
        match self {
            PinkEvent::SetHook { .. } => false,
            PinkEvent::DeploySidevmTo { .. } => true,
            PinkEvent::SidevmMessage(_) => true,
            PinkEvent::CacheOp(_) => true,
            PinkEvent::StopSidevm => true,
            PinkEvent::ForceStopSidevm { .. } => true,
            PinkEvent::SetLogHandler(_) => false,
            PinkEvent::SetContractWeight { .. } => false,
            PinkEvent::UpgradeRuntimeTo { .. } => false,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            PinkEvent::SetHook { .. } => "SetHook",
            PinkEvent::DeploySidevmTo { .. } => "DeploySidevmTo",
            PinkEvent::SidevmMessage(_) => "SidevmMessage",
            PinkEvent::CacheOp(_) => "CacheOp",
            PinkEvent::StopSidevm => "StopSidevm",
            PinkEvent::ForceStopSidevm { .. } => "ForceStopSidevm",
            PinkEvent::SetLogHandler(_) => "SetLogHandler",
            PinkEvent::SetContractWeight { .. } => "SetContractWeight",
            PinkEvent::UpgradeRuntimeTo { .. } => "UpgradeRuntimeTo",
        }
    }

    pub fn is_private(&self) -> bool {
        match self {
            PinkEvent::SetHook { .. } => false,
            PinkEvent::DeploySidevmTo { .. } => false,
            PinkEvent::SidevmMessage(_) => true,
            PinkEvent::CacheOp(_) => true,
            PinkEvent::StopSidevm => false,
            PinkEvent::ForceStopSidevm { .. } => false,
            PinkEvent::SetLogHandler(_) => false,
            PinkEvent::SetContractWeight { .. } => false,
            PinkEvent::UpgradeRuntimeTo { .. } => false,
        }
    }
}

/// Instructions to manipulate the cache.
#[derive(Encode, Decode, Debug, Clone)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum CacheOp {
    /// Set a key-value pair in the cache.
    Set { key: Vec<u8>, value: Vec<u8> },
    /// Set the expiration of a key-value pair in the cache.
    SetExpiration { key: Vec<u8>, expiration: u64 },
    /// Remove a key-value pair from the cache.
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

/// Sets a hook receiver for a given hook point.
///
/// A hook is a mechanism that allows certain actions to be triggered when a specific situation arises.
/// When the situation corresponding to the hook point occurs, the runtime will call the receiver contract
/// using the specified selector.
///
/// # Supported Hook Points
///  - `OnBlockEnd`: The receiver contract will be invoked once all events in a Phala chain block have been processed.
///
/// # Arguments
///
/// * `hook`: The hook point for which the receiver is set.
/// * `contract`: The AccountId of the contract to be called when the hook is triggered.
/// * `selector`: The function selector to be used when calling the receiver contract.
/// * `gas_limit`: The maximum amount of gas that can be used when calling the receiver contract.
///
/// Note: The cost of the execution would be charged to the contract itself.
/// 
/// This api is only available for the system contract. User contracts should use `System::set_hook` instead.
pub fn set_hook(hook: HookPoint, contract: AccountId, selector: u32, gas_limit: u64) {
    emit_event::<PinkEnvironment, _>(PinkEvent::SetHook {
        hook,
        contract,
        selector,
        gas_limit,
    })
}

/// Starts a SideVM instance with the provided code hash.
///
/// The calling contract must be authorized by the `SidevmOperation` driver contract.
///
/// If the code corresponding to the provided hash hasn't been uploaded to the cluster storage yet,
/// it will create an empty SideVM instance. This instance will wait for the code to be uploaded
/// via `prpc::UploadSidevmCode`.
///
///# Arguments
///
///* `code_hash`: The hash of the code to be used for starting the SideVM instance.
///
///# Returns
///
/// A `Result` indicating success or failure, specifically a `system::DriverError` in case of failure.
pub fn start_sidevm(code_hash: Hash) -> Result<(), system::DriverError> {
    let driver =
        crate::system::SidevmOperationRef::instance().ok_or(system::Error::DriverNotFound)?;
    driver.deploy(code_hash)
}

/// Deploy a SideVM instance to a given contract. (system only)
pub fn deploy_sidevm_to(contract: AccountId, code_hash: Hash) {
    emit_event::<PinkEnvironment, _>(PinkEvent::DeploySidevmTo {
        contract,
        code_hash,
    });
}

/// Stop a SideVM instance running at given contract address. (system only)
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

/// Pushes a message to the associated SideVM instance.
///
/// Note: There is no guarantee that the message will be received by the SideVM instance.
/// The message may be dropped due to several reasons:
///
/// - The SideVM instance is not currently running.
/// - The SideVM instance is running, but the message queue is full. This may occur when the SideVM
///   instance is busy processing other messages.
///
///# Arguments
///
///* `message`: The message to be pushed to the SideVM instance.
pub fn push_sidevm_message(message: Vec<u8>) {
    emit_event::<PinkEnvironment, _>(PinkEvent::SidevmMessage(message))
}

/// Set the log handler contract of current cluster. (system only)
pub fn set_log_handler(contract: AccountId) {
    emit_event::<PinkEnvironment, _>(PinkEvent::SetLogHandler(contract))
}

/// Set the weight of contract used to schedule queries and sidevm virtual runtime. (system only)
pub fn set_contract_weight(contract: AccountId, weight: u32) {
    emit_event::<PinkEnvironment, _>(PinkEvent::SetContractWeight { contract, weight });
}

/// Upgrade the pink runtime to given version. (system only)
///
/// Note: pRuntime would exit if the version is not supported.
pub fn upgrade_runtime(version: (u32, u32)) {
    emit_event::<PinkEnvironment, _>(PinkEvent::UpgradeRuntimeTo { version });
}

/// Generate a slice of verifiable random bytes.
///
/// When called in a contract with the same salt, the same random bytes will be generated.
/// Different contracts with the same salt will generate different random bytes.
///
/// # Availability
/// any contract | query | transaction
pub fn vrf(salt: &[u8]) -> Vec<u8> {
    let mut key_salt = b"vrf:".to_vec();
    key_salt.extend_from_slice(salt);
    ext().derive_sr25519_key(key_salt.into())
}

/// Pink defined environment. This environment is used to access the phat contract extended runtime features.
///
/// # Example
/// ```
/// #[ink::contract(env = PinkEnvironment)]
/// mod my_contract {
///     use pink_extension::PinkEnvironment;
///     #[ink(storage)]
///     pub struct MyContract {}
///     impl MyContract {
///         #[ink(constructor)]
///         pub fn new() -> Self {
///             Self {}
///         }
///         #[ink(message)]
///         pub fn my_message(&self) {
///             // Access the pink environment.
///             let _pink_version = self.env().extension().runtime_version();
///         }
///     }
/// }
/// ```
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

/// Returns the PinkEnvironment.
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

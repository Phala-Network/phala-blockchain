#![allow(clippy::let_unit_value)]

use pink_extension_macro as pink;

use alloc::string::String;
use alloc::vec::Vec;
use scale::{Decode, Encode};

use crate::{AccountId, Balance, Hash};

/// Errors that can occur upon calling the system contract.
#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
    PermisionDenied,
    DriverNotFound,
    CodeNotFound,
    ConditionNotMet,
}

/// The code type for existance check.
#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum CodeType {
    Ink,
    Sidevm,
}

impl CodeType {
    pub fn is_ink(&self) -> bool {
        matches!(self, CodeType::Ink)
    }
    pub fn is_sidevm(&self) -> bool {
        matches!(self, CodeType::Sidevm)
    }
}

/// Result type for the system contract messages
pub type Result<T, E = Error> = core::result::Result<T, E>;
pub use this_crate::VersionTuple;

/// The pink system contract interface.
///
/// A system contract would be instantiated whenever a cluster is created.
#[pink::system]
#[ink::trait_definition(namespace = "pink_system")]
pub trait System {
    /// The version of the system. Can be used to determine the api ability.
    #[ink(message, selector = 0x87c98a8d)]
    fn version(&self) -> VersionTuple;
    /// Grant an address the administrator role.
    ///
    /// The caller must be the owner of the cluster.
    #[ink(message)]
    fn grant_admin(&mut self, contract_id: AccountId) -> Result<()>;

    /// Check if an address is an administrator
    #[ink(message)]
    fn is_admin(&self, contract_id: AccountId) -> bool;

    /// Set a contract as a driver for `name`.
    ///
    /// The caller must be the owner of the cluster or an administrator.
    #[ink(message)]
    fn set_driver(&mut self, name: String, contract_id: AccountId) -> Result<()>;

    /// Get driver contract id for `name`.
    #[ink(message)]
    fn get_driver(&self, name: String) -> Option<AccountId>;

    /// Get driver contract id for `name` and the set block number.
    #[ink(message)]
    fn get_driver2(&self, name: String) -> Option<(crate::BlockNumber, AccountId)>;

    /// Deploy a sidevm instance attached to a given contract.
    ///
    /// The caller must be an administrator.
    #[ink(message)]
    fn deploy_sidevm_to(&self, contract_id: AccountId, code_hash: Hash) -> Result<()>;

    /// Stop a sidevm instance attached to a given contract.
    ///
    /// The caller must be an administrator.
    #[ink(message)]
    fn stop_sidevm_at(&self, contract_id: AccountId) -> Result<()>;

    /// Set block hook, such as OnBlockEnd, for given contract
    ///
    /// The caller must be an administrator.
    #[ink(message)]
    fn set_hook(
        &mut self,
        hook: crate::HookPoint,
        contract_id: AccountId,
        selector: u32,
        gas_limit: u64,
    ) -> Result<()>;

    /// Set weight of the contract for query requests and sidevm scheduling.
    ///
    /// Higher weight would let the contract to get more resource.
    #[ink(message)]
    fn set_contract_weight(&self, contract_id: AccountId, weight: u32) -> Result<()>;

    /// Return the total balance of given account
    #[ink(message)]
    fn total_balance_of(&self, account: AccountId) -> Balance;

    /// Return the free balance of given account
    #[ink(message)]
    fn free_balance_of(&self, account: AccountId) -> Balance;

    /// Upgrade the system contract to the latest version.
    #[ink(message)]
    fn upgrade_system_contract(&mut self) -> Result<()>;

    /// Do the upgrade condition checks and state migration if necessary.
    ///
    /// This function is called by the system contract itself on the new version
    /// of code in the upgrading process.
    #[ink(message)]
    fn do_upgrade(&self, from_version: VersionTuple) -> Result<()>;

    /// Upgrade the contract runtime
    #[ink(message)]
    fn upgrade_runtime(&mut self, version: (u32, u32)) -> Result<()>;

    /// Check if the code is already uploaded to the cluster with given code hash.
    #[ink(message)]
    fn code_exists(&self, code_hash: Hash, code_type: CodeType) -> bool;

    /// Get the current code hash of given contract.
    #[ink(message)]
    fn code_hash(&self, account: AccountId) -> Option<ink::primitives::Hash>;

    /// Get the history of given driver.
    #[ink(message)]
    fn driver_history(&self, name: String) -> Option<Vec<(crate::BlockNumber, AccountId)>>;
}

/// Errors that can occur upon calling a driver contract.
#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum DriverError {
    Other(String),
    SystemError(Error),
    BadOrigin,
}

impl From<Error> for DriverError {
    fn from(value: Error) -> Self {
        Self::SystemError(value)
    }
}

/// Driver to manage sidevm deployments.
#[pink::driver]
#[ink::trait_definition]
pub trait SidevmOperation {
    /// Invoked by a contract to deploy a sidevm instance that attached to itself.
    #[ink(message)]
    fn deploy(&self, code_hash: Hash) -> Result<(), DriverError>;

    /// Check if given address has the permission to deploy a sidevm.
    #[ink(message)]
    fn can_deploy(&self, contract_id: AccountId) -> bool;
}

/// Contracts receiving processing deposit events. Can be a driver and the system.
#[pink::driver]
#[ink::trait_definition]
pub trait ContractDeposit {
    /// Change deposit of a contract. A driver should set the contract weight according to the
    /// new deposit.
    #[ink(message, selector = 0xa24bcb44)]
    fn change_deposit(
        &mut self,
        contract_id: AccountId,
        deposit: Balance,
    ) -> Result<(), DriverError>;
}

#![allow(clippy::let_unit_value)]

use pink_extension_macro as pink;

use alloc::string::String;
use alloc::vec::Vec;
use scale::{Decode, Encode};

use crate::{AccountId, Balance, Hash, SidevmConfig, WorkerId};

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
/// The system contract, instantiated with each cluster creation, manages access permissions to
/// the privileged chain extension functions and pink events. Some of these functions or events
/// are exclusive to the system contract. User contracts wishing to call these functions or
/// emit these events must first request the system contract, which then checks the permissions
/// to either execute or reject the request.
#[pink::system]
#[ink::trait_definition(namespace = "pink_system")]
pub trait System {
    /// Returns the system contract version, indicating its API capabilities.
    ///
    /// # Example
    /// ```no_run
    /// use pink_extension::system::SystemRef;
    /// let (major, minor, patch) = SystemRef::instance().version();
    /// ```
    #[ink(message, selector = 0x87c98a8d)]
    fn version(&self) -> VersionTuple;

    /// Grants the administrator role to an address. Administrator contracts can set drivers,
    /// deploy sidevm, etc.
    ///
    /// Must be called by the cluster owner.
    #[ink(message)]
    fn grant_admin(&mut self, contract_id: AccountId) -> Result<()>;

    /// Checks if an address is an administrator.
    #[ink(message)]
    fn is_admin(&self, contract_id: AccountId) -> bool;

    /// Marks a contract as a driver for a given name, retrievable via `get_driver` or `get_driver2`.
    /// The caller must be the cluster owner or an administrator. Any valid string can be a name.
    /// There are predefined names used by the Phat Contract system.
    ///
    /// There are some predefined names that are used by the Phat Contract system:
    /// - `PinkLogger`: The contract that with a sidevm instance that collect the logs and events
    ///  emitted by the ink! contracts in current cluster.
    /// - `ContractDeposit`: The contract that implements the `trait ContractDeposit` which talks
    ///  to the pallet PhatTokenomic on Phala chain.
    #[ink(message)]
    fn set_driver(&mut self, name: String, contract_id: AccountId) -> Result<()>;

    /// Retrieves the driver contract id for a given name.
    #[ink(message)]
    fn get_driver(&self, name: String) -> Option<AccountId>;

    /// Retrieves the driver contract id and the set block number for a given name.
    #[ink(message)]
    fn get_driver2(&self, name: String) -> Option<(crate::BlockNumber, AccountId)>;

    /// Deploys a sidevm instance attached to a contract. Must be called by an administrator.
    #[ink(message)]
    fn deploy_sidevm_to(&self, contract_id: AccountId, code_hash: Hash) -> Result<()>;

    /// Stops a sidevm instance attached to a contract. Must be called by an administrator.
    #[ink(message)]
    fn stop_sidevm_at(&self, contract_id: AccountId) -> Result<()>;

    /// Sets a block hook for a contract. Must be called by an administrator.
    /// Note: This feature is deprecated and will be removed in the future.
    #[ink(message)]
    fn set_hook(
        &mut self,
        hook: crate::HookPoint,
        contract_id: AccountId,
        selector: u32,
        gas_limit: u64,
    ) -> Result<()>;

    /// Sets the contract weight for query requests and sidevm scheduling.
    /// A higher weight allows the contract to access more resources.
    #[ink(message)]
    fn set_contract_weight(&self, contract_id: AccountId, weight: u32) -> Result<()>;

    /// Returns the total balance of a given account.
    #[ink(message)]
    fn total_balance_of(&self, account: AccountId) -> Balance;

    /// Returns the free balance of a given account.
    #[ink(message)]
    fn free_balance_of(&self, account: AccountId) -> Balance;

    /// Upgrades the system contract to the latest version.
    #[ink(message)]
    fn upgrade_system_contract(&mut self) -> Result<()>;

    /// Performs upgrade condition checks and state migration if necessary.
    /// Called by the system contract on the new code version during an upgrade process.
    #[ink(message)]
    fn do_upgrade(&self, from_version: VersionTuple) -> Result<()>;

    /// Upgrades the contract runtime.
    #[ink(message)]
    fn upgrade_runtime(&mut self, version: (u32, u32)) -> Result<()>;

    /// Checks if the code with a given hash is already uploaded to the cluster.
    #[ink(message)]
    fn code_exists(&self, code_hash: Hash, code_type: CodeType) -> bool;

    /// Retrieves the current code hash of a given contract.
    #[ink(message)]
    fn code_hash(&self, account: AccountId) -> Option<ink::primitives::Hash>;

    /// Retrieves the history of a given driver, returning a vector of
    /// (block_number, contract_id) tuples where the block number is the
    /// block number when the driver is set.
    #[ink(message)]
    fn driver_history(&self, name: String) -> Option<Vec<(crate::BlockNumber, AccountId)>>;

    /// Get current event chain head info
    ///
    /// Returns (next event block number, last event block hash)
    #[ink(message)]
    fn current_event_chain_head(&self) -> (u64, Hash);

    /// Deploys a sidevm instance attached to a contract on selected workers. Must be called by an administrator.
    #[ink(message)]
    fn deploy_sidevm_to_workers(
        &self,
        contract_id: AccountId,
        code_hash: Hash,
        workers: Vec<crate::WorkerId>,
        config: SidevmConfig,
    ) -> Result<()>;

    /// Sets a deadline for sidevm instances attached to a contract on selected workers. Must be called by an administrator.
    #[ink(message)]
    fn set_sidevm_deadline(
        &self,
        contract_id: AccountId,
        run_until: crate::BlockNumber,
    ) -> Result<()>;
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

    /// Invoked by a contract to deploy a sidevm instance that attached to itself to selected workers.
    #[ink(message)]
    fn deploy_to_workers(
        &self,
        code_hash: Hash,
        workers: Vec<WorkerId>,
        deadline: u32,
        max_memory_pages: u32,
        code_size: u32,
    ) -> Result<(), DriverError>;
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

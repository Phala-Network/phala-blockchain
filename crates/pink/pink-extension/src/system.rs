#![allow(clippy::let_unit_value)]

use ink_lang as ink;
use pink_extension_macro as pink;

use alloc::string::String;
use scale::{Decode, Encode};

use crate::{AccountId, Balance, Hash};

/// Errors that can occur upon calling the system contract.
#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
    BadOrigin,
    DriverNotFound,
}

/// Result type for the system contract messages
pub type Result<T> = core::result::Result<T, Error>;

/// The pink system contract interface.
///
/// A system contract would be instantiated whenever a cluster is created.
#[pink::system]
#[ink::trait_definition(namespace = "pink_system")]
pub trait System {
    /// The version of the system. Can be used to determine the api ability.
    #[ink(message, selector = 0x87c98a8d)]
    fn version(&self) -> (u16, u16);
    /// Grant an address the administrator role.
    ///
    /// The caller must be the owner of the cluster.
    #[ink(message)]
    fn grant_admin(&mut self, contract_id: AccountId) -> Result<()>;

    /// Set a contract as a driver for `name`.
    ///
    /// The caller must be the owner of the cluster or an administrator.
    #[ink(message)]
    fn set_driver(&mut self, name: String, contract_id: AccountId) -> Result<()>;

    /// Set a contract as a driver for `name`.
    ///
    /// The caller must be the owner of the cluster or an administrator.
    #[ink(message)]
    fn get_driver(&self, name: String) -> Option<AccountId>;

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

    /// Return the total balance of the caller
    #[ink(message)]
    fn total_balance(&self) -> Balance;

    /// Return the free balance of the caller
    #[ink(message)]
    fn free_balance(&self) -> Balance;
}

/// Driver to manage sidevm deployments.
#[pink::driver]
#[ink::trait_definition]
pub trait SidevmOperation {
    /// Invoked by a contract to deploy a sidevm instance that attached to itself.
    #[ink(message)]
    fn deploy(&self, code_hash: Hash) -> Result<()>;
}

/// Contracts receiving processing deposit events. Can be a driver and the system.
#[pink::driver]
#[ink::trait_definition]
pub trait ContractDeposit {
    /// Change deposit of a contract. A driver should set the contract weight according to the
    /// new deposit.
    #[ink(message, selector = 0xa24bcb44)]
    fn change_deposit(&mut self, contract_id: AccountId, deposit: Balance) -> Result<()>;
}

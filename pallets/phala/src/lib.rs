#![cfg_attr(not(feature = "std"), no_std)]

//! Phala Pallets
//!
//! This is the central crate of Phala tightly-coupled pallets.

extern crate alloc;

// Re-export
use utils::{attestation, balance_convert, constants};

pub mod migrations;
pub mod utils;

pub mod compute;
pub mod mq;
pub mod phat;
pub mod puppets;
pub mod registry;
pub mod stake_pool;

use compute::{base_pool, computation, pool_proxy, stake_pool_v2, vault, wrapped_balances};

use frame_support::traits::LockableCurrency;
use frame_system::pallet_prelude::BlockNumberFor;

/// The unified config of the compute pallets
pub trait PhalaConfig: frame_system::Config {
	type Currency: LockableCurrency<Self::AccountId, Moment = BlockNumberFor<Self>>;
}
/// The unified type Balance of pallets from the runtime T.
type BalanceOf<T> = <<T as PhalaConfig>::Currency as frame_support::traits::Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;
/// The unified type ImBalance of pallets from the runtime T.
type NegativeImbalanceOf<T> = <<T as PhalaConfig>::Currency as frame_support::traits::Currency<
	<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

// Alias
pub use compute::base_pool as pallet_base_pool;
pub use compute::computation as pallet_computation;
pub use compute::stake_pool_v2 as pallet_stake_pool_v2;
pub use compute::vault as pallet_vault;
pub use compute::wrapped_balances as pallet_wrapped_balances;
pub use mq as pallet_mq;
pub use phat as pallet_phat;
pub use phat_tokenomic as pallet_phat_tokenomic;
pub use registry as pallet_registry;
pub use stake_pool as pallet_stake_pool;
pub mod phat_tokenomic;

#[cfg(feature = "native")]
use sp_core::hashing;

#[cfg(not(feature = "native"))]
use sp_io::hashing;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod test;

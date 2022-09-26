#![cfg_attr(not(feature = "std"), no_std)]

//! Phala Pallets
//!
//! This is the central crate of Phala tightly-coupled pallets.

#[cfg(target_arch = "wasm32")]
extern crate webpki_wasm as webpki;

#[cfg(not(feature = "std"))]
extern crate alloc;

// Re-export
use utils::{attestation, balance_convert, constants};

pub mod migrations;
pub mod utils;

pub mod compute;
pub mod fat;
pub mod mq;
pub mod puppets;
pub mod registry;

use compute::{basepool, mining, pawnshop, poolproxy, stakepoolv2, vault};

use frame_support::traits::LockableCurrency;
/// The unified config of the compute pallets
pub trait PhalaConfig: frame_system::Config {
	type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
}
/// The unified type Balance of pallets in compute with template T.
type BalanceOf<T> = <<T as PhalaConfig>::Currency as frame_support::traits::Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;
/// The unified type ImBalance of pallets in compute with template T.
type NegativeImbalanceOf<T> = <<T as PhalaConfig>::Currency as frame_support::traits::Currency<
	<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

// Alias
pub use compute::basepool as pallet_basepool;
pub use compute::mining as pallet_mining;
pub use compute::pawnshop as pallet_pawnshop;
pub use compute::stakepoolv2 as pallet_stakepool;
pub use compute::vault as pallet_vault;
pub use fat as pallet_fat;
pub use mq as pallet_mq;
pub use registry as pallet_registry;

#[cfg(feature = "native")]
use sp_core::hashing;

#[cfg(not(feature = "native"))]
use sp_io::hashing;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod test;

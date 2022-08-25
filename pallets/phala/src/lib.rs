#![cfg_attr(not(feature = "std"), no_std)]

//! Phala Pallets
//!
//! This is the central crate of Phala tightly-coupled pallets.

#[cfg(target_arch = "wasm32")]
extern crate webpki_wasm as webpki;

#[cfg(not(feature = "std"))]
extern crate alloc;

// Re-export
use utils::{accumulator, attestation, balance_convert, constants, fixed_point};

pub mod migrations;
pub mod utils;

pub mod basepool;
pub mod fat;
pub mod mining;
pub mod mq;
pub mod ott;
pub mod pawnshop;
pub mod poolproxy;
pub mod puppets;
pub mod registry;
pub mod stakepoolv2;
pub mod vault;

// Alias
pub use basepool as pallet_basepool;
pub use fat as pallet_fat;
pub use mining as pallet_mining;
pub use mq as pallet_mq;
pub use ott as pallet_ott;
pub use pawnshop as pallet_pawnshop;
pub use registry as pallet_registry;
pub use stakepoolv2 as pallet_stakepool;
pub use vault as pallet_vault;

#[cfg(feature = "native")]
use sp_core::hashing;

#[cfg(not(feature = "native"))]
use sp_io::hashing;

#[cfg(test)]
mod mock;

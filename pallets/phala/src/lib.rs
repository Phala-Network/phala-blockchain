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

pub mod fat;
pub mod mining;
pub mod mq;
pub mod ott;
pub mod puppets;
pub mod registry;
pub mod stakepool;

// Alias
pub use fat as pallet_fat;
pub use mining as pallet_mining;
pub use mq as pallet_mq;
pub use ott as pallet_ott;
pub use registry as pallet_registry;
pub use stakepool as pallet_stakepool;

#[cfg(feature = "native")]
use sp_core::hashing;

#[cfg(not(feature = "native"))]
use sp_io::hashing;

#[cfg(test)]
mod mock;

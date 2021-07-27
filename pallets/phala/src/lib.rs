#![cfg_attr(not(feature = "std"), no_std)]

//! Phala Pallets
//!
//! This is the central crate of Phala tightly-coupled pallets.
//!
//! - `phala_legacy`: The legacy `pallet-phala`; will be retired gradually
//! - `mq`: The message queue to connect components in the network
//! - `registry`: Manages the public key of offchain components (i.e. workers and contracts)
//! - `mining`: Manages mining lifecycle, reward and slashes
//! - `stakepool`: Pool for collaboratively mining staking
//!
//! # Status
//!
//! Now `phala-pallet` is still functional, but it will be gradually deconstructed as separate
//! pallets.
//!
//! The `mq` pallet has a basic implementation and it's supposed to work. The `registry` pallet
//! only has very basic API for testing.

pub(crate) mod attestation;
pub(crate) mod balance_convert;
pub mod constants;

pub mod mining;
pub mod mq;
pub mod registry;
pub mod stakepool;

// Alias
pub use mining as pallet_mining;
pub use mq as pallet_mq;
pub use registry as pallet_registry;
pub use stakepool as pallet_stakepool;

#[cfg(feature = "native")]
use sp_core::hashing;

#[cfg(not(feature = "native"))]
use sp_io::hashing;

#[cfg(test)]
mod mock;

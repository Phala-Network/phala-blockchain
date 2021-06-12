#![cfg_attr(not(feature = "std"), no_std)]

pub mod mq;
pub mod phala_legacy;
pub mod registry;

// Alicas
pub use mq as pallet_mq;
pub use phala_legacy as pallet_phala;
pub use registry as pallet_registry;

#![cfg_attr(not(feature = "std"), no_std)]

// pub mod mq;
pub mod phala_legacy;

// pub use mq as pallet_mq;
pub use phala_legacy as pallet_phala;

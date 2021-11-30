#![no_std]
extern crate alloc;

pub mod scale_bytes;

#[cfg(feature = "crypto")]
pub mod key_bytes;

#[cfg(feature = "crypto")]
pub mod option_key_bytes;

#[cfg(feature = "crypto")]
pub mod pubkey_bytes;
#![no_std]

extern crate alloc;

#[cfg(test)]
#[macro_use]
extern crate std;

pub mod ecdh_key;
pub mod identity_key;
pub mod master_key;

pub const SEED_SIZE: usize = 32;
pub type Seed = [u8; SEED_SIZE];

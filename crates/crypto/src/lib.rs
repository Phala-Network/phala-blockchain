#![no_std]

extern crate alloc;

#[cfg(test)]
#[macro_use]
extern crate std;

pub mod ecdh;
pub mod aead;
pub mod secp256k1;

#[derive(Debug)]
pub enum KeyError {
    InvalidSeedLength,
    SecretStringError(sp_core::crypto::SecretStringError),
}

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[cfg(test)]
#[macro_use]
extern crate std;

pub mod aead;
pub mod ecdh;
pub mod sr25519;

#[cfg(feature = "full_crypto")]
pub mod key_share;

#[derive(Debug)]
pub enum CryptoError {
    HkdfExpandError,
    // Ecdh errors
    EcdhInvalidSecretKey,
    EcdhInvalidPublicKey,
    // Aead errors
    AeadInvalidKey,
    AeadEncryptError,
    AeadDecryptError,
    // sr25519
    Sr25519InvalidSecret,
}

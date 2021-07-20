#![no_std]

extern crate alloc;

#[cfg(test)]
#[macro_use]
extern crate std;

pub mod ecdh;
pub mod aead;
pub mod sr25519;

#[derive(Debug)]
pub enum CryptoError {
    // Ecdsa errors
    EcdsaInvalidSeedLength(sp_core::crypto::SecretStringError),
    EcdsaHkdfExpandError,
    // Ecdh errors
    EcdhInvalidSecretKey,
    EcdhInvalidPublicKey,
    // Aead errors
    AeadInvalidKey,
    AeadEncryptError,
    AeadDecryptError,
}

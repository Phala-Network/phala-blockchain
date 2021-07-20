use crate::{ecdh::EcdhKey, CryptoError};

use alloc::{vec, vec::Vec};
use ring::hkdf;
use sp_core::{sr25519, Pair};

pub const SIGNATURE_BYTES: usize = 64;
pub type Signature = sr25519::Signature;

pub const SEED_BYTES: usize = 32;
pub type Seed = [u8; SEED_BYTES];

pub trait Signing {
    fn sign_data(&self, data: &[u8]) -> Signature;

    fn verify_data(&self, sig: &Signature, data: &[u8]) -> bool;
}

impl Signing for sr25519::Pair {
    fn sign_data(&self, data: &[u8]) -> Signature {
        sr25519::Pair::sign(self, data)
    }

    fn verify_data(&self, sig: &Signature, data: &[u8]) -> bool {
        sr25519::Pair::verify(sig, data, &self.public())
    }
}

// Phala Network Poc-4 Genesis Block Hash
const KDF_SALT: [u8; 32] = [
    0x18, 0xe8, 0x76, 0xad, 0xfa, 0x74, 0xcc, 0x74, 0x3b, 0x4b, 0x7d, 0x7f, 0x92, 0xc9, 0x6e, 0x03,
    0xde, 0x55, 0x9a, 0x3c, 0x15, 0x27, 0x05, 0x81, 0xfc, 0xe4, 0x45, 0x96, 0x3d, 0x90, 0xf8, 0x5e,
];

pub trait KDF {
    fn derive_sr25519_pair(&self, info: &[&[u8]]) -> Result<sr25519::Pair, CryptoError>;

    fn derive_ecdh_key(&self) -> Result<EcdhKey, CryptoError>;
}

/// Generic newtype wrapper that lets us implement traits for externally-defined
/// types.
///
/// Refer to https://github.com/briansmith/ring/blob/main/tests/hkdf_tests.rs
#[derive(Debug, PartialEq)]
struct My<T: core::fmt::Debug + PartialEq>(T);

impl hkdf::KeyType for My<usize> {
    fn len(&self) -> usize {
        self.0
    }
}

impl From<hkdf::Okm<'_, My<usize>>> for My<Vec<u8>> {
    fn from(okm: hkdf::Okm<My<usize>>) -> Self {
        let mut r = vec![0_u8; okm.len().0];
        okm.fill(&mut r).unwrap();
        Self(r)
    }
}

impl KDF for sr25519::Pair {
    // TODO.shelven: allow to specify the salt from pruntime (instead of hard code)
    fn derive_sr25519_pair(&self, info: &[&[u8]]) -> Result<sr25519::Pair, CryptoError> {
        let salt = hkdf::Salt::new(hkdf::HKDF_SHA256, &KDF_SALT);
        let prk = salt.extract(&self.as_ref().secret.to_bytes()[..32]);
        let okm = prk
            .expand(info, My(SEED_BYTES))
            .map_err(|_| CryptoError::EcdsaHkdfExpandError)?;

        let mut seed: Seed = [0_u8; SEED_BYTES];
        okm.fill(seed.as_mut())
            .map_err(|_| CryptoError::EcdsaHkdfExpandError)?;

        Ok(sr25519::Pair::from_seed(&seed))
    }

    fn derive_ecdh_key(&self) -> Result<EcdhKey, CryptoError> {
        EcdhKey::from_secret(&self.as_ref().secret.to_bytes())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn generate_key() -> sr25519::Pair {
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        let mut seed: Seed = [0_u8; SEED_BYTES];

        rng.fill_bytes(&mut seed);

        sr25519::Pair::from_seed(&seed)
    }

    #[test]
    fn sign_and_verify() {
        let key = generate_key();
        let data = [233u8; 32];

        let sig = key.sign_data(&data);
        assert!(key.verify_data(&sig, &data))
    }

    #[test]
    fn key_derivation() {
        let secp256k1_key = generate_key();
        // this should not panic
        secp256k1_key
            .derive_sr25519_pair(&[&[255], &[255, 255], &[255, 255, 255]])
            .unwrap();

        let ecdh_key = secp256k1_key.derive_ecdh_key().unwrap();
        // this should not panic
        ecdh_key.public();
    }
}

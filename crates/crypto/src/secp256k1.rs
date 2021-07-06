use crate::{ecdh::EcdhKey, KeyError};
use alloc::{vec, vec::Vec};
use ring::hkdf;
use sp_core::{ecdsa, Pair};

pub type Signature = ecdsa::Signature;

const SEED_SIZE: usize = 32;
type Seed = [u8; SEED_SIZE];

pub trait Signing {
    fn sign_data(&self, data: &[u8]) -> Signature;

    fn verify_data(&self, sig: &Signature, data: &[u8]) -> bool;
}

impl Signing for ecdsa::Pair {
    fn sign_data(&self, data: &[u8]) -> Signature {
        sp_core::Pair::sign(self, data)
    }

    fn verify_data(&self, sig: &Signature, data: &[u8]) -> bool {
        ecdsa::Pair::verify(sig, data, &self.public())
    }
}

// Phala Network Poc-4 Genesis Block Hash
const KDF_SALT: [u8; 32] = [
    0x18, 0xe8, 0x76, 0xad, 0xfa, 0x74, 0xcc, 0x74, 0x3b, 0x4b, 0x7d, 0x7f, 0x92, 0xc9, 0x6e, 0x03,
    0xde, 0x55, 0x9a, 0x3c, 0x15, 0x27, 0x05, 0x81, 0xfc, 0xe4, 0x45, 0x96, 0x3d, 0x90, 0xf8, 0x5e,
];

pub trait KDF {
    fn derive_secp256k1_pair(&self, info: &[&[u8]]) -> Result<ecdsa::Pair, KeyError>;

    fn derive_ecdh_key(&self) -> Result<EcdhKey, KeyError>;
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
        let mut r = vec![0u8; okm.len().0];
        okm.fill(&mut r).unwrap();
        Self(r)
    }
}

impl KDF for ecdsa::Pair {
    fn derive_secp256k1_pair(&self, info: &[&[u8]]) -> Result<ecdsa::Pair, KeyError> {
        let salt = hkdf::Salt::new(hkdf::HKDF_SHA256, &KDF_SALT);
        let prk = salt.extract(&self.seed());
        let okm = prk
            .expand(info, My(SEED_SIZE))
            .expect("failed in hkdf expand");

        let mut seed: Seed = [0_u8; SEED_SIZE];
        okm.fill(seed.as_mut()).expect("failed to fill output buff");

        ecdsa::Pair::from_seed_slice(&seed).map_err(|err| KeyError::SecretStringError(err))
    }

    fn derive_ecdh_key(&self) -> Result<EcdhKey, KeyError> {
        EcdhKey::create(&self.seed())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn generate_key() -> ecdsa::Pair {
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        let mut seed: Seed = [0u8; SEED_SIZE];

        rng.fill_bytes(&mut seed);

        ecdsa::Pair::from_seed(&seed)
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
            .derive_secp256k1_pair(&[&[255], &[255, 255], &[255, 255, 255]])
            .unwrap();

        let ecdh_key = secp256k1_key.derive_ecdh_key().unwrap();
        // this should not panic
        ecdh_key.public();
    }
}

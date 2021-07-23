use crate::{ecdh::EcdhKey, CryptoError};

use alloc::{vec, vec::Vec};
use ring::hkdf;
pub use schnorrkel::{MINI_SECRET_KEY_LENGTH, PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH};
use sp_core::{sr25519, Pair};

pub const SIGNATURE_BYTES: usize = 64;
pub type Signature = sr25519::Signature;

pub type Sr25519SecretKey = [u8; SECRET_KEY_LENGTH]; // 32 privkey, 32 nonce
pub type Sr25519PublicKey = [u8; PUBLIC_KEY_LENGTH]; // 32 compressed pubkey

pub const SEED_BYTES: usize = MINI_SECRET_KEY_LENGTH;
pub type Seed = [u8; SEED_BYTES];

pub trait Signing {
    fn sign_data(&self, data: &[u8]) -> Signature;

    fn verify_data(&self, sig: &Signature, data: &[u8]) -> bool;
}

pub trait KDF {
    fn derive_sr25519_pair(&self, info: &[&[u8]]) -> Result<sr25519::Pair, CryptoError>;

    fn derive_ecdh_key(&self) -> Result<EcdhKey, CryptoError>;
}

pub trait Persistence {
    fn dump_seed(&self) -> Seed;

    fn dump_secret_key(&self) -> Sr25519SecretKey;

    fn restore_from_seed(seed: &Seed) -> sr25519::Pair;

    fn restore_from_secret_key(secret: &Sr25519SecretKey) -> sr25519::Pair;
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
        let prk = salt.extract(&self.as_ref().secret.to_bytes());
        let okm = prk
            .expand(info, My(SEED_BYTES))
            .map_err(|_| CryptoError::HkdfExpandError)?;

        let mut seed: Seed = [0_u8; SEED_BYTES];
        okm.fill(seed.as_mut())
            .map_err(|_| CryptoError::HkdfExpandError)?;

        Ok(sr25519::Pair::from_seed(&seed))
    }

    fn derive_ecdh_key(&self) -> Result<EcdhKey, CryptoError> {
        EcdhKey::from_secret(&self.as_ref().secret.to_bytes())
    }
}

impl Persistence for sr25519::Pair {
    fn dump_seed(&self) -> Seed {
        panic!("No available seed for sr25519 pair");
    }

    fn dump_secret_key(&self) -> Sr25519SecretKey {
        self.as_ref().secret.to_bytes()
    }

    fn restore_from_seed(seed: &Seed) -> sr25519::Pair {
        sr25519::Pair::from_seed(seed)
    }

    fn restore_from_secret_key(secret: &Sr25519SecretKey) -> sr25519::Pair {
        sr25519::Pair::from_seed_slice(secret).expect("should never panic given valid secret; qed.")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn generate_key() -> (sr25519::Pair, Seed) {
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        let mut seed: Seed = [0_u8; SEED_BYTES];

        rng.fill_bytes(&mut seed);

        (sr25519::Pair::from_seed(&seed), seed)
    }

    #[test]
    fn dump_and_restore() {
        let (key, seed) = generate_key();
        let secret = key.dump_secret_key();

        let key1 = sr25519::Pair::restore_from_seed(&seed);
        let key2 = sr25519::Pair::restore_from_secret_key(&secret);

        assert_eq!(key.to_raw_vec(), key1.to_raw_vec());
        assert_eq!(key.to_raw_vec(), key2.to_raw_vec());
    }

    #[test]
    fn sign_and_verify() {
        let (key, _) = generate_key();
        let data = [233u8; 32];

        let sig = key.sign_data(&data);
        assert!(key.verify_data(&sig, &data));
    }

    #[test]
    fn key_derivation() {
        let (sr25519_key, _) = generate_key();
        // this should not panic
        sr25519_key
            .derive_sr25519_pair(&[&[255], &[255, 255], &[255, 255, 255]])
            .unwrap();

        let ecdh_key = sr25519_key.derive_ecdh_key().unwrap();
        // this should not panic
        ecdh_key.public();
    }
}

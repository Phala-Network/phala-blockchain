use crate::{Seed, SEED_SIZE};
use alloc::{vec, vec::Vec};
use ring::hkdf;
use sp_core::crypto::{Pair, SecretStringError};
use sp_core::ecdsa;

// Phala Network Poc-4 Genesis Block Hash
const KDF_SALT: [u8; 32] = [
    0x18, 0xe8, 0x76, 0xad, 0xfa, 0x74, 0xcc, 0x74, 0x3b, 0x4b, 0x7d, 0x7f, 0x92, 0xc9, 0x6e, 0x03,
    0xde, 0x55, 0x9a, 0x3c, 0x15, 0x27, 0x05, 0x81, 0xfc, 0xe4, 0x45, 0x96, 0x3d, 0x90, 0xf8, 0x5e,
];

pub struct MasterKey(ecdsa::Pair);

pub struct ContractKey(ecdsa::Pair);

impl MasterKey {
    pub fn from_seed(seed: &Seed) -> Result<Self, SecretStringError> {
        let pair = ecdsa::Pair::from_seed_slice(seed)?;
        Ok(MasterKey(pair))
    }

    pub fn seed(&self) -> Seed {
        self.0.seed()
    }

    pub fn public(&self) -> ecdsa::Public {
        self.0.public()
    }

    pub fn derive_contract_key(&self, info: &[&[u8]]) -> Result<ContractKey, SecretStringError> {
        let salt = hkdf::Salt::new(hkdf::HKDF_SHA256, &KDF_SALT);
        let prk = salt.extract(&self.seed());
        let okm = prk
            .expand(info, My(SEED_SIZE))
            .expect("failed in hkdf expand");

        let mut seed: Seed = [0_u8; SEED_SIZE];
        okm.fill(seed.as_mut()).expect("failed to fill output buff");

        ContractKey::from_seed(&seed)
    }

    pub fn to_raw_vec(&self) -> Vec<u8> {
        self.0.to_raw_vec()
    }
}

impl ContractKey {
    pub fn from_seed(seed: &Seed) -> Result<Self, SecretStringError> {
        let pair = ecdsa::Pair::from_seed_slice(seed)?;
        Ok(ContractKey(pair))
    }

    pub fn seed(&self) -> Seed {
        self.0.seed()
    }

    pub fn public(&self) -> ecdsa::Public {
        self.0.public()
    }

    pub fn to_raw_vec(&self) -> Vec<u8> {
        self.0.to_raw_vec()
    }
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

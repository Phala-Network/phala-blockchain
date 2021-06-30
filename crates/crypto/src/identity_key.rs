use alloc::vec::Vec;
use sp_core::crypto::{Pair, SecretStringError};
use sp_core::ecdsa;

pub struct IdentityKey(ecdsa::Pair);

pub type Seed = [u8; 32];

impl IdentityKey {
    fn from_seed(seed: &Seed) -> Result<Self, SecretStringError> {
        let pair = ecdsa::Pair::from_seed_slice(seed)?;
        Ok(IdentityKey(pair))
    }

    fn seed(&self) -> Seed {
        self.0.seed()
    }

    fn public(&self) -> ecdsa::Public {
        self.0.public()
    }

    fn sign(&self, message: &[u8]) -> ecdsa::Signature {
        self.0.sign(message)
    }

    fn verify(sig: &ecdsa::Signature, message: &[u8], pubkey: &ecdsa::Public) -> bool {
        ecdsa::Pair::verify(sig, message, pubkey)
    }

    fn to_raw_vec(&self) -> Vec<u8> {
        self.0.to_raw_vec()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn generate_identity_key() -> (IdentityKey, Seed) {
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        let mut seed: Seed = [0u8; 32];

        rng.fill_bytes(&mut seed);

        (IdentityKey::from_seed(&seed).unwrap(), seed)
    }

    #[test]
    fn key_generation() {
        let (key1, seed1) = generate_identity_key();
        let key2 = IdentityKey::from_seed(&seed1).unwrap();

        assert_eq!(key1.seed(), key2.seed());
        assert_eq!(key1.public(), key2.public());
    }

    #[test]
    fn sign_and_verify() {
        let (key, _) = generate_identity_key();
        let message = [233u8; 32];

        let sig = key.sign(&message);
        assert!(IdentityKey::verify(&sig, &message, &key.public()))
    }
}

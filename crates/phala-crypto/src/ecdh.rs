use crate::CryptoError;

use alloc::vec::Vec;
use curve25519_dalek::scalar::Scalar;
use schnorrkel::keys::{ExpansionMode, Keypair, MiniSecretKey, PublicKey, SecretKey};
use schnorrkel::{MINI_SECRET_KEY_LENGTH, PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH};

/// sr25519 key pair
#[derive(Clone)]
pub struct EcdhKey(Keypair);

pub type EcdhSecretKey = [u8; SECRET_KEY_LENGTH]; // 32 privkey, 32 nonce
pub type EcdhPublicKey = [u8; PUBLIC_KEY_LENGTH]; // 32 compressed pubkey

pub type Seed = [u8; MINI_SECRET_KEY_LENGTH]; // 32 seed

impl EcdhKey {
    pub fn create(seed: &Seed) -> Result<EcdhKey, CryptoError> {
        Ok(EcdhKey(
            MiniSecretKey::from_bytes(seed)
                .map_err(|_| CryptoError::EcdhInvalidSecretKey)?
                .expand_to_keypair(ExpansionMode::Ed25519),
        ))
    }

    pub fn from_secret(secret: &EcdhSecretKey) -> Result<EcdhKey, CryptoError> {
        Ok(EcdhKey(
            SecretKey::from_bytes(secret.as_ref())
                .map_err(|_| CryptoError::EcdhInvalidSecretKey)?
                .to_keypair(),
        ))
    }

    pub fn public(&self) -> EcdhPublicKey {
        self.0.public.to_bytes()
    }

    pub fn secret(&self) -> EcdhSecretKey {
        self.0.secret.to_bytes()
    }
}

/// Derives a secret key for symmetric encryption without a KDF
///
/// `pk` must be in compressed version.
pub fn agree(sk: &EcdhKey, pk: &[u8]) -> Result<Vec<u8>, CryptoError> {
    // The first 32 bytes holds the canonical private key
    let mut key = [0u8; 32];
    key.copy_from_slice(&sk.secret()[0..32]);
    let key = Scalar::from_canonical_bytes(key).expect("This should never fail with correct seed");
    let public = PublicKey::from_bytes(pk).or(Err(CryptoError::EcdhInvalidPublicKey))?;
    Ok((key * public.as_point()).compress().0.to_vec())
}

#[cfg(test)]
mod test {
    use super::*;

    fn generate_key() -> EcdhKey {
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        let mut seed: Seed = [0_u8; MINI_SECRET_KEY_LENGTH];

        rng.fill_bytes(&mut seed);

        EcdhKey::create(&seed).unwrap()
    }

    #[test]
    fn ecdh_key_clone() {
        let key1 = generate_key();
        let key2 = key1.clone();
        let key3 = EcdhKey::from_secret(&key1.secret()).unwrap();

        println!(
            "{:?}, {:?}, {:?}",
            key1.secret(),
            key2.secret(),
            key3.secret()
        );

        assert_eq!(key1.secret(), key2.secret());
        assert_eq!(key1.secret(), key3.secret());
    }

    #[test]
    fn ecdh_agree() {
        let key1 = generate_key();
        let key2 = generate_key();

        println!("{:?}", agree(&key1, key2.public().as_ref()));
        println!("{:?}", agree(&key2, key1.public().as_ref()));

        assert_eq!(
            agree(&key1, key2.public().as_ref()).unwrap(),
            agree(&key2, key1.public().as_ref()).unwrap(),
        )
    }
}

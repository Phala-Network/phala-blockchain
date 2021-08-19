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
    Ok((&key * public.as_point()).compress().0.to_vec())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::aead;

    fn generate_key() -> (EcdhKey, Seed) {
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        let mut seed: Seed = [0_u8; MINI_SECRET_KEY_LENGTH];

        rng.fill_bytes(&mut seed);

        (EcdhKey::create(&seed).unwrap(), seed)
    }

    // #[test]
    // fn ecdh_key_clone() {
    //     let key1 = generate_key();
    //     let key2 = key1.clone();
    //     let key3 = EcdhKey::from_secret(&key1.secret()).unwrap();

    //     // println!(
    //     //     "{:?}, {:?}, {:?}",
    //     //     key1.secret(),
    //     //     key2.secret(),
    //     //     key3.secret()
    //     // );

    //     assert_eq!(key1.secret(), key2.secret());
    //     assert_eq!(key1.secret(), key3.secret());
    // }

    #[test]
    fn ecdh_agree() {
        use rand::RngCore;
        let mut rng = rand::thread_rng();

        for n in 1..10 {
            let (key1, seed1) = generate_key();
            let (key2, seed2) = generate_key();

            let agree_key = agree(&key1, key2.public().as_ref()).unwrap();

            let mut iv: aead::IV = [0; 12];
            rng.fill_bytes(&mut iv);

            let mut data = [0_u8; 32].to_vec();
            rng.fill_bytes(&mut data);

            let plaintext = data.clone();

            let encrypted_text = aead::encrypt(&iv, &agree_key, &mut data);

            println!("{{");
            println!(
                "seed1: {}, sk1: {}, pk1: {}",
                hex::encode(&seed1),
                hex::encode(key1.secret()),
                hex::encode(key1.public())
            );
            println!(
                "seed2: {}, sk2: {}, pk2: {}",
                hex::encode(&seed2),
                hex::encode(key2.secret()),
                hex::encode(key2.public())
            );
            println!("agree_key: {}", hex::encode(&agree_key));
            println!(
                "iv: {}, plain_text: {}, encrypted_text: {}",
                hex::encode(&iv),
                hex::encode(&plaintext),
                hex::encode(&data)
            );
            println!("}}");
        }
    }
}

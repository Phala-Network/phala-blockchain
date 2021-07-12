use crate::KeyError;
use alloc::vec::Vec;
use core::convert::TryInto;
use core::mem;
use ring::agreement::{EphemeralPrivateKey, UnparsedPublicKey};
use ring::rand::SystemRandom;

/// secp256r1 key pair
pub struct EcdhKey(EphemeralPrivateKey);

pub type EcdhPrivateKey = [u8; 32];
pub type EcdhPublicKey = ring::agreement::PublicKey;

impl EcdhKey {
    /// Generate an ECDH key pair (secp256r1)
    ///
    /// WARNING: the generated key pair should never be used in real-world scenarios
    pub fn generate() -> EcdhKey {
        // This rng relies on operating system thus is not trusted
        let rng = SystemRandom::new();
        let sk = EphemeralPrivateKey::generate(&ring::agreement::ECDH_P256, &rng)
            .expect("private key generation failed");
        EcdhKey(sk)
    }

    // A hack to create arbitrary private key
    pub fn create(key: &EcdhPrivateKey) -> Result<EcdhKey, KeyError> {
        let len = mem::size_of::<EphemeralPrivateKey>();
        if len != 64 {
            return Err(KeyError::InvalidSeedLength);
        }

        let base_key = EcdhKey::generate();
        unsafe {
            let mut memlayout: [u8; 64] = mem::transmute_copy(&base_key.0);
            let key_slice = &mut memlayout[8..40];
            key_slice.copy_from_slice(key);
            return Ok(EcdhKey(mem::transmute_copy(&memlayout)));
        }
    }

    // A hack to bypass ring's one-time key restriction
    pub fn clone(&self) -> EcdhKey {
        unsafe {
            let new_sk = mem::transmute_copy::<EphemeralPrivateKey, EphemeralPrivateKey>(&self.0);
            EcdhKey(new_sk)
        }
    }

    pub fn public(&self) -> EcdhPublicKey {
        self.0
            .compute_public_key()
            .expect("public key generation failed")
    }

    pub fn secret(&self) -> EcdhPrivateKey {
        let memlayout: [u8; 64] = unsafe { mem::transmute_copy(&self.0) };
        memlayout[8..40]
            .try_into()
            .expect("slice with incorrect length")
    }
}

fn agree_longlived<B: AsRef<[u8]>, F, R, E>(
    my_private_key: &EcdhKey,
    peer_public_key: &UnparsedPublicKey<B>,
    error_value: E,
    kdf: F,
) -> Result<R, E>
where
    F: FnOnce(&[u8]) -> Result<R, E>,
{
    let sk = my_private_key.clone();
    ring::agreement::agree_ephemeral(sk.0, peer_public_key, error_value, kdf)
}

// Derives a secret key for symmetric encryption without a KDF
pub fn agree(sk: &EcdhKey, pk: &[u8]) -> Vec<u8> {
    let unparsed_pk = ring::agreement::UnparsedPublicKey::new(&ring::agreement::ECDH_P256, pk);

    agree_longlived(sk, &unparsed_pk, ring::error::Unspecified, |key_material| {
        Ok(key_material.to_vec())
    })
    .expect("ecdh failed")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ecdh_key_clone() {
        let key1 = EcdhKey::generate();
        let key2 = key1.clone();
        let key3 = EcdhKey::create(&key1.secret()).unwrap();

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
        let key1 = EcdhKey::generate();
        let key2 = EcdhKey::generate();

        println!("{:?}", agree(&key1, key2.public().as_ref()));
        println!("{:?}", agree(&key2, key1.public().as_ref()));

        assert_eq!(
            agree(&key1, key2.public().as_ref()),
            agree(&key2, key1.public().as_ref()),
        )
    }
}

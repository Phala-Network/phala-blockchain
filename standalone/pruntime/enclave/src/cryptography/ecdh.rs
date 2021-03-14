use crate::std::vec::Vec;
use core::convert::TryInto;
use ring::agreement::{EphemeralPrivateKey, UnparsedPublicKey};
use ring::rand::SystemRandom;

// Generates an ECDH key paiir (secp256r1)
pub fn generate_key() -> EphemeralPrivateKey {
    let prng = SystemRandom::new();
    let ecdh_sk =
        EphemeralPrivateKey::generate(&ring::agreement::ECDH_P256, &prng).expect("gen key failed");
    ecdh_sk
}

// A hack to bypass ring's one-time key restriction
pub fn clone_key(key: &EphemeralPrivateKey) -> EphemeralPrivateKey {
    unsafe { std::mem::transmute_copy::<EphemeralPrivateKey, EphemeralPrivateKey>(key) }
}

// A damn hack to create arbitrary private key
pub fn create_key(key: &[u8]) -> Result<EphemeralPrivateKey, ()> {
    let len = std::mem::size_of::<EphemeralPrivateKey>();
    if len != 64 {
        println!("ecdh::create_key unknown EphemeralPrivateKey layout");
        return Err(());
    }
    if key.len() != 32 {
        println!("ecdh::create_key bad key length 32 vs {}", key.len());
        return Err(());
    }

    let base_key = generate_key();
    unsafe {
        let mut memlayout: [u8; 64] = std::mem::transmute_copy(&base_key);
        let key_slice = &mut memlayout[8..40];
        key_slice.copy_from_slice(key);
        return Ok(std::mem::transmute_copy(&memlayout));
    };
}

pub fn dump_key(key: &EphemeralPrivateKey) -> [u8; 32] {
    let memlayout: [u8; 64] = unsafe { std::mem::transmute_copy(key) };
    memlayout[8..40]
        .try_into()
        .expect("slice with incorrect length")
}

fn agree_longlived<B: AsRef<[u8]>, F, R, E>(
    my_private_key: &EphemeralPrivateKey,
    peer_public_key: &UnparsedPublicKey<B>,
    error_value: E,
    kdf: F,
) -> Result<R, E>
where
    F: FnOnce(&[u8]) -> Result<R, E>,
{
    let sk = clone_key(my_private_key);
    ring::agreement::agree_ephemeral(sk, peer_public_key, error_value, kdf)
}

// Derives a secret key for symmetric encryption without a KDF
pub fn agree(sk: &EphemeralPrivateKey, pubkey: &[u8]) -> Vec<u8> {
    let unparsed_pk = ring::agreement::UnparsedPublicKey::new(&ring::agreement::ECDH_P256, pubkey);

    agree_longlived(sk, &unparsed_pk, ring::error::Unspecified, |key_material| {
        Ok(key_material.to_vec())
    })
    .expect("ecdh failed")
}

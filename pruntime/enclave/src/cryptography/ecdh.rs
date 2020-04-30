use crate::std::vec::Vec;
use ring::agreement::{EphemeralPrivateKey, UnparsedPublicKey};
use ring::rand::SystemRandom;

// Generates an ECDH key paiir (secp256r1)
pub fn generate_key() -> EphemeralPrivateKey {
  let prng = SystemRandom::new();
  let ecdh_sk = EphemeralPrivateKey::generate(
    &ring::agreement::ECDH_P256, &prng).expect("gen key failed");
  ecdh_sk
}

// A hack to bypass ring's one-time key restriction
fn clone_key(key: &EphemeralPrivateKey) -> EphemeralPrivateKey {
  unsafe { std::mem::transmute_copy::<EphemeralPrivateKey, EphemeralPrivateKey>(key) }
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
  let unparsed_pk = ring::agreement::UnparsedPublicKey::new(
    &ring::agreement::ECDH_P256, pubkey);

  agree_longlived(
    sk, &unparsed_pk,
    ring::error::Unspecified,
    |key_material| Ok(key_material.to_vec())
  ).expect("ecdh failed")
}

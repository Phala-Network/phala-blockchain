use crate::aead::{self, IV};
use crate::ecdh::{self, EcdhKey, EcdhPublicKey};
use crate::sr25519::{Sr25519SecretKey, KDF};
use crate::CryptoError;

use alloc::vec::Vec;
use sp_core::{sr25519, Pair};

pub fn encrypt_key_to(
    my_key: &sr25519::Pair,
    key_derive_info: &[&[u8]],
    ecdh_pubkey: &EcdhPublicKey,
    secret_key: &Sr25519SecretKey,
    iv: &IV,
) -> Result<(EcdhPublicKey, Vec<u8>), CryptoError> {
    let derived_key = my_key.derive_sr25519_pair(key_derive_info)?;
    let my_ecdh_key = derived_key.derive_ecdh_key()?;
    let secret = ecdh::agree(&my_ecdh_key, ecdh_pubkey)?;
    let mut data = secret_key.to_vec();
    aead::encrypt(&iv, &secret, &mut data)?;

    Ok((my_ecdh_key.public(), data))
}

pub fn decrypt_key_from(
    my_ecdh_key: &EcdhKey,
    ecdh_pubkey: &EcdhPublicKey,
    encrypted_key: &Vec<u8>,
    iv: &IV,
) -> Result<sr25519::Pair, CryptoError> {
    let secret = ecdh::agree(my_ecdh_key, ecdh_pubkey)?;
    let mut key_buff = encrypted_key.clone();
    let secret_key = aead::decrypt(iv, &secret, &mut key_buff[..])?;
    sr25519::Pair::from_seed_slice(secret_key).map_err(|_| CryptoError::Sr25519InvalidSecret)
}

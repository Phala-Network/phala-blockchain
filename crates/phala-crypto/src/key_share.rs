use crate::aead::{self, IV};
use crate::ecdh::{self, EcdhKey, EcdhPublicKey};
use crate::sr25519::{Sr25519SecretKey, KDF};
use crate::CryptoError;

use alloc::borrow::ToOwned;
use alloc::vec::Vec;
use core::convert::TryInto;
use sp_core::sr25519;

pub fn encrypt_secret_to(
    my_key: &sr25519::Pair,
    key_derive_info: &[&[u8]],
    ecdh_pubkey: &EcdhPublicKey,
    secret_key: &Sr25519SecretKey,
    iv: &IV,
) -> Result<(EcdhPublicKey, Vec<u8>), CryptoError> {
    let derived_key = my_key.derive_sr25519_pair(key_derive_info)?;
    let my_ecdh_key = derived_key.derive_ecdh_key();
    let secret = ecdh::agree(&my_ecdh_key, ecdh_pubkey);
    let mut data = secret_key.to_vec();
    aead::encrypt(iv, &secret, &mut data)?;

    Ok((my_ecdh_key.public(), data))
}

pub fn decrypt_secret_from(
    my_ecdh_key: &EcdhKey,
    ecdh_pubkey: &EcdhPublicKey,
    encrypted_key: &[u8],
    iv: &IV,
) -> Result<Sr25519SecretKey, CryptoError> {
    let secret = ecdh::agree(my_ecdh_key, ecdh_pubkey);
    let mut key_buff = encrypted_key.to_owned();
    let secret_key = aead::decrypt(iv, &secret, &mut key_buff[..])?;
    secret_key
        .try_into()
        .or(Err(CryptoError::Sr25519InvalidSecret))
}

#[test]
fn it_works() {
    use sp_core::Pair;

    let key0 = sr25519::Pair::from_seed(b"12345678901234567890123456789012");
    let key1 = sr25519::Pair::from_seed(b"12345678901234567890123456789013");
    let pubkey1 = key1.public().0;

    let iv = aead::generate_iv(b"12345678901234567890123456789012");

    let (pubkey, encrypted_key) =
        encrypt_secret_to(&key0, &[b"test", b"test"], &pubkey1, &[1u8; 64], &iv).unwrap();
    let decrypted_key =
        decrypt_secret_from(&key1.derive_ecdh_key(), &pubkey, &encrypted_key, &iv).unwrap();
    assert_eq!(decrypted_key, [1u8; 64]);
}

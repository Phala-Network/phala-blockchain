use alloc::vec::Vec;

use crate::{EcdsaPublicKey, EcdsaSignature, Hash};

#[derive(scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum SigType {
    Ed25519,
    Sr25519,
    Ecdsa,
}

/// Sign a message with a private key.
///
/// # Examples
/// ```ignore
/// let (privkey, pubkey) = derive_sr25519_pair(b"a spoon of salt");
/// let message = b"hello world";
/// let signature = sign(message, &privkey, SigType::Sr25519);
/// let pass = verify(message, &pubkey, &signature, SigType::Sr25519);
/// assert!(pass);
/// ```
pub fn sign(message: &[u8], key: &[u8], sigtype: SigType) -> Vec<u8> {
    crate::ext().sign(sigtype, key, message)
}

/// Verify a message with a pubkey and signature.
///
/// # Examples
/// ```ignore
/// let (privkey, pubkey) = derive_sr25519_pair(b"a spoon of salt");
/// let message = b"hello world";
/// let signature = sign(message, &privkey, SigType::Sr25519);
/// let pass = verify(message, &pubkey, &signature, SigType::Sr25519);
/// assert!(pass);
/// ```
pub fn verify(message: &[u8], pubkey: &[u8], signature: &[u8], sigtype: SigType) -> bool {
    crate::ext().verify(sigtype, pubkey, message, signature)
}

/// Sign a prehashed message with a ECDSA priviate key
pub fn ecdsa_sign_prehashed(key: &[u8], message_hash: Hash) -> EcdsaSignature {
    crate::ext().ecdsa_sign_prehashed(key, message_hash)
}

/// Verify a prehashed message with a ECDSA pubkey and signature.
pub fn ecdsa_verify_prehashed(
    signature: EcdsaSignature,
    message_hash: Hash,
    pubkey: EcdsaPublicKey,
) -> bool {
    crate::ext().ecdsa_verify_prehashed(signature, message_hash, pubkey)
}

/// Derive a key pair from the contract key
///
/// # Examples
/// ```ignore
/// let privkey = derive_sr25519_key(b"a spoon of salt");
/// let pubkey = get_public_key(&privkey, SigType::Sr25519);
/// let message = b"hello world";
/// let signature = sign(message, &privkey, SigType::Sr25519);
/// let pass = verify(message, &pubkey, &signature, SigType::Sr25519);
/// assert!(pass);
/// ```
pub fn derive_sr25519_key(salt: &[u8]) -> Vec<u8> {
    crate::ext().derive_sr25519_key(salt.into())
}

/// Get the public key from a private key
///
/// # Examples
/// ```ignore
/// let privkey = derive_sr25519_key(b"a spoon of salt");
/// let pubkey = get_public_key(&privkey, SigType::Sr25519);
/// let message = b"hello world";
/// let signature = sign(message, &privkey, SigType::Sr25519);
/// let pass = verify(message, &pubkey, &signature, SigType::Sr25519);
/// assert!(pass);
/// ```
pub fn get_public_key(key: &[u8], sigtype: SigType) -> Vec<u8> {
    crate::ext().get_public_key(sigtype, key)
}

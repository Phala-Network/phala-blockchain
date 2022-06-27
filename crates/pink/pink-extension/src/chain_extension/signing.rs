use alloc::borrow::Cow;
use alloc::vec::Vec;

#[derive(scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum SigType {
    Ed25519,
    Sr25519,
    Ecdsa,
}

#[derive(scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct SignArgs<'a> {
    pub sigtype: SigType,
    pub key: Cow<'a, [u8]>,
    pub message: Cow<'a, [u8]>,
}

#[derive(scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct VerifyArgs<'a> {
    pub sigtype: SigType,
    pub pubkey: Cow<'a, [u8]>,
    pub message: Cow<'a, [u8]>,
    pub signature: Cow<'a, [u8]>,
}

#[derive(scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct PublicKeyForArgs<'a> {
    pub sigtype: SigType,
    pub key: Cow<'a, [u8]>,
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
    let args = SignArgs {
        sigtype,
        message: message.into(),
        key: key.into(),
    };
    crate::ext().sign(args)
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
    let args = VerifyArgs {
        sigtype,
        message: message.into(),
        pubkey: pubkey.into(),
        signature: signature.into(),
    };
    crate::ext().verify(args)
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
    let args = PublicKeyForArgs {
        sigtype,
        key: key.into(),
    };
    crate::ext().get_public_key(args)
}

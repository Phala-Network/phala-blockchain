use alloc::borrow::Cow;

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
/// let (privkey, pubkey) = derive_sr25519_pair!(b"a spoon of salt");
/// let message = b"hello world";
/// let signature = sign!(message, &privkey, SigType::Sr25519);
/// let pass = verify!(message, &pubkey, &signature, SigType::Sr25519);
/// assert!(pass);
/// ```
#[macro_export]
macro_rules! sign {
    ($message: expr, $key: expr, $sigtype: expr) => {{
        use $crate::chain_extension::SignArgs;
        let message = $message.into();
        let key = $key.into();
        let sigtype = $sigtype;
        let args = SignArgs {
            sigtype,
            message,
            key,
        };
        $crate::pink_extension_instance().sign(args)
    }};
}

/// Verify a message with a pubkey and signature.
///
/// # Examples
/// ```ignore
/// let (privkey, pubkey) = derive_sr25519_pair!(b"a spoon of salt");
/// let message = b"hello world";
/// let signature = sign!(message, &privkey, SigType::Sr25519);
/// let pass = verify!(message, &pubkey, &signature, SigType::Sr25519);
/// assert!(pass);
/// ```
#[macro_export]
macro_rules! verify {
    ($message: expr, $pubkey: expr, $signature: expr, $sigtype: expr) => {{
        use $crate::chain_extension::VerifyArgs;
        let message = $message.into();
        let pubkey = $pubkey.into();
        let signature = $signature.into();
        let sigtype = $sigtype;
        let args = VerifyArgs {
            sigtype,
            message,
            pubkey,
            signature,
        };
        $crate::pink_extension_instance().verify(args)
    }};
}


/// Derive a key pair from the contract key
///
/// # Examples
/// ```ignore
/// let privkey = derive_sr25519_key!(b"a spoon of salt");
/// let pubkey = get_public_key!(&privkey, SigType::Sr25519);
/// let message = b"hello world";
/// let signature = sign!(message, &privkey, SigType::Sr25519);
/// let pass = verify!(message, &pubkey, &signature, SigType::Sr25519);
/// assert!(pass);
/// ```
#[macro_export]
macro_rules! derive_sr25519_key {
    ($salt: expr) => {{
        let salt: &[u8] = $salt.as_ref();
        $crate::pink_extension_instance().derive_sr25519_key(salt.into())
    }};
}


/// Get the public key from a private key
///
/// # Examples
/// ```ignore
/// let privkey = derive_sr25519_key!(b"a spoon of salt");
/// let pubkey = get_public_key!(&privkey, SigType::Sr25519);
/// let message = b"hello world";
/// let signature = sign!(message, &privkey, SigType::Sr25519);
/// let pass = verify!(message, &pubkey, &signature, SigType::Sr25519);
/// assert!(pass);
/// ```
#[macro_export]
macro_rules! get_public_key {
    ($key: expr, $sigtype: expr) => {{
        use $crate::chain_extension::PublicKeyForArgs;
        let key: &[u8] = $key.as_ref();
        let sigtype = $sigtype;
        let args = PublicKeyForArgs {
            sigtype,
            key: key.into(),
        };
        $crate::pink_extension_instance().get_public_key(args)
    }};
}

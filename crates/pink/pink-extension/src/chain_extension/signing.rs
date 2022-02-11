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
pub struct SignArgs {
    pub sigtype: SigType,
    pub key: Vec<u8>,
    pub message: Vec<u8>,
}

#[derive(scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct VerifyArgs {
    pub sigtype: SigType,
    pub pubkey: Vec<u8>,
    pub message: Vec<u8>,
    pub signature: Vec<u8>,
}


/// Sign a message with a private key.
/// # Example
/// ```ignore
/// let privkey = todo!();
/// let pubkey = todo!();
/// let message = b"hello world";
/// let signature = sign!(message, privkey, SigType::Sr25519);
/// let pass = verify!(message, pubkey, signature, SigType::Sr25519);
/// assert!(pass);
/// ```
#[macro_export]
macro_rules! sign {
    ($message: expr, $key: expr, $sigtype: expr) => {{
        use pink_extension::chain_extension::SignArgs;
        let message = $message.into();
        let key = $key.into();
        let sigtype = $sigtype;
        let args = SignArgs {
            sigtype,
            message,
            key,
        };
        Self::env().extension().sign(args)
    }};
}

/// Verify a message with a pubkey and signature.
/// # Example
/// ```ignore
/// let privkey = todo!();
/// let pubkey = todo!();
/// let message = b"hello world";
/// let signature = sign!(message, privkey, SigType::Sr25519);
/// let pass = verify!(message, pubkey, signature, SigType::Sr25519);
/// assert!(pass);
/// ```
#[macro_export]
macro_rules! verify {
    ($message: expr, $pubkey: expr, $signature: expr, $sigtype: expr) => {{
        use pink_extension::chain_extension::VerifyArgs;
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
        Self::env().extension().verify(args)
    }};
}

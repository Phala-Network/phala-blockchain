use crate::std::string::String;
use crate::std::vec::Vec;
use anyhow::{Context, Result};
use core::fmt;

use parity_scale_codec::{Decode, Encode};
use sp_core::crypto::Pair;

pub mod aead;

#[derive(Debug, Clone, Encode, Decode)]
pub struct AeadCipher {
    pub iv_b64: String,
    pub cipher_b64: String,
    pub pubkey_b64: String,
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct Origin {
    pub origin: String,
    pub sig_b64: String,
    pub sig_type: SignatureType,
}

#[derive(Encode, Decode, Debug, Clone)]
pub enum SignatureType {
    Ed25519,
    Sr25519,
    Ecdsa,
}

impl Origin {
    pub fn verify(&self, msg: &[u8]) -> Result<bool> {
        let sig = base64::decode(&self.sig_b64)
            .map_err(|_| anyhow::Error::msg(Error::BadInput("sig_b64")))?;
        let pubkey: Vec<_> = hex::decode(&self.origin)
            .map_err(anyhow::Error::msg)
            .context("Failed to decode origin hex")?;

        let result = match self.sig_type {
            SignatureType::Ed25519 => verify::<sp_core::ed25519::Pair>(&sig, msg, &pubkey),
            SignatureType::Sr25519 => verify::<sp_core::sr25519::Pair>(&sig, msg, &pubkey),
            SignatureType::Ecdsa => verify::<sp_core::ecdsa::Pair>(&sig, msg, &pubkey),
        };
        Ok(result)
    }
}

fn verify<T>(sig: &[u8], msg: &[u8], pubkey: &[u8]) -> bool
where
    T: Pair,
{
    T::verify_weak(sig, msg, pubkey)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    BadInput(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::BadInput(e) => write!(f, "bad input: {}", e),
        }
    }
}

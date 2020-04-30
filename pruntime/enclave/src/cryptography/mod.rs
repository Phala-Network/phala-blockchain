use crate::std::string::String;
use crate::std::vec::Vec;
use serde::{Serialize, Deserialize};

use sp_core::crypto::Pair;

pub mod aead;
pub mod ecdh;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AeadCipher {
  pub iv_b64: String,
  pub cipher_b64: String,
  pub pubkey_b64: String
}

pub struct DecryptOutput {
  pub msg: Vec<u8>,
  pub secret: Vec<u8>
}

// Decrypt by AEAD-AES-GCM with secret key agreeded by ECDH.
pub fn decrypt(cipher: &AeadCipher, privkey: &ring::agreement::EphemeralPrivateKey)
  -> Result<DecryptOutput, Error> 
{
  let pubkey = base64::decode(&cipher.pubkey_b64)
      .map_err(|_| Error::BadInput("pubkey_b64"))?;
  let mut data = base64::decode(&cipher.cipher_b64)
      .map_err(|_| Error::BadInput("cipher_b64"))?;
  let iv = base64::decode(&cipher.iv_b64)
      .map_err(|_| Error::BadInput("iv_b64"))?;
  // ECDH derived secret
  let secret = ecdh::agree(privkey, &pubkey);
  println!("Agreed SK: {:?}", crate::hex::encode_hex_compact(&secret));
  let msg = aead::decrypt(iv.as_slice(), secret.as_slice(), &mut data);
  Ok(DecryptOutput {
    msg: msg.to_vec(),
    secret
  })
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Origin {
  pub origin: String,
  pub sig_b64: String,
  pub sig_type: SignatureType
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SignatureType {
  #[serde(rename = "ed25519")]
  Ed25519,
  #[serde(rename = "sr25519")]
  Sr25519,
  #[serde(rename = "ecdsa")]
  Ecdsa,
}

impl Origin {
  pub fn verify(&self, msg: &[u8]) -> Result<bool, Error> {
    let sig = base64::decode(&self.sig_b64)
      .map_err(|_| Error::BadInput("sig_b64"))?;
    let pubkey = crate::hex::decode_hex(&self.origin);
      // .map_err(|_| Error::BadInput("origin"))?;   TODO: handle error

    let result = match self.sig_type {
      SignatureType::Ed25519 => verify::<sp_core::ed25519::Pair>(&sig, msg, &pubkey),
      SignatureType::Sr25519 => verify::<sp_core::sr25519::Pair>(&sig, msg, &pubkey),
      SignatureType::Ecdsa => verify::<sp_core::ecdsa::Pair>(&sig, msg, &pubkey)
    };
    Ok(result)
  }
}

fn verify<T>(sig: &[u8], msg: &[u8], pubkey: &[u8]) -> bool
where T: Pair {
  T::verify_weak(sig, msg, pubkey)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
  BadInput(&'static str),
}
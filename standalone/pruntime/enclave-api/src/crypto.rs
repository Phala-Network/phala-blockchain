use alloc::vec::Vec;
use parity_scale_codec::{Decode, Encode};

use crate::prpc::{Signature, SignatureType};
pub use phala_crypto::{aead, ecdh, CryptoError};

#[derive(Clone, Encode, Decode)]
pub struct EncryptedData {
    pub iv: aead::IV,
    pub pubkey: ecdh::EcdhPublicKey,
    pub data: Vec<u8>,
}

impl EncryptedData {
    pub fn decrypt(&self, key: &ecdh::EcdhKey) -> Result<Vec<u8>, CryptoError> {
        let sk = ecdh::agree(key, &self.pubkey)?;
        let mut tmp_data = self.data.clone();
        let msg = aead::decrypt(&self.iv, &sk, &mut tmp_data)?;
        Ok(msg.to_vec())
    }

    pub fn encrypt(
        key: &ecdh::EcdhKey,
        remote_pubkey: &ecdh::EcdhPublicKey,
        iv: aead::IV,
        data: &[u8],
    ) -> Result<Self, CryptoError> {
        let sk = ecdh::agree(&key, &remote_pubkey[..])?;
        let mut data = data.to_vec();
        aead::encrypt(&iv, &sk, &mut data)?;
        Ok(Self {
            iv,
            pubkey: key.public(),
            data,
        })
    }
}

impl Signature {
    pub fn verify(&self, msg: &[u8]) -> bool {
        let sig_type = match SignatureType::from_i32(self.signature_type) {
            Some(val) => val,
            None => {
                return false;
            }
        };

        match sig_type {
            SignatureType::Ed25519 => {
                verify::<sp_core::ed25519::Pair>(&self.origin, &self.signature, msg)
            }
            SignatureType::Sr25519 => {
                verify::<sp_core::sr25519::Pair>(&self.origin, &self.signature, msg)
            }
            SignatureType::Ecdsa => {
                verify::<sp_core::ecdsa::Pair>(&self.origin, &self.signature, msg)
            }
        }
    }
}

fn verify<T>(pubkey: &[u8], sig: &[u8], msg: &[u8]) -> bool
where
    T: sp_core::crypto::Pair,
{
    T::verify_weak(sig, msg, pubkey)
}

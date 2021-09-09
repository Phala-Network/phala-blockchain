use alloc::vec;
use alloc::vec::Vec;
use parity_scale_codec::{Decode, Encode, Error as CodecError};

use crate::prpc::{Signature, SignatureType};
pub use phala_crypto::{aead, ecdh, CryptoError};

#[derive(Clone, Encode, Decode, Debug)]
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
        let sk = ecdh::agree(key, &remote_pubkey[..])?;
        let mut data = data.to_vec();
        aead::encrypt(&iv, &sk, &mut data)?;
        Ok(Self {
            iv,
            pubkey: key.public(),
            data,
        })
    }
}

#[derive(Clone, Debug)]
pub enum SignatureVerifyError {
    InvalidSignatureType,
    InvalidSignature,
    CertificateMissing,
    CertificateExpired,
    TooLongCertificateChain,
    DecodeFailed(CodecError),
}

impl From<CodecError> for SignatureVerifyError {
    fn from(err: CodecError) -> Self {
        SignatureVerifyError::DecodeFailed(err)
    }
}

impl Signature {
    /// Verify signature and return the siger pubkey chain in top-down order.
    pub fn verify(
        &self,
        msg: &[u8],
        current_block: u32,
        max_depth: u32,
    ) -> Result<Vec<Vec<u8>>, SignatureVerifyError> {
        if max_depth == 0 {
            return Err(SignatureVerifyError::TooLongCertificateChain);
        }
        let sig_type = match SignatureType::from_i32(self.signature_type) {
            Some(val) => val,
            None => {
                return Err(SignatureVerifyError::InvalidSignatureType);
            }
        };

        match &self.signed_by {
            Some(cert) => {
                let cert = *cert.clone();
                let body = cert.decode_body()?;

                if body.ttl < current_block {
                    return Err(SignatureVerifyError::CertificateExpired);
                }

                body.verify(msg, sig_type, &self.signature)?;

                let key_chain = if let Some(cert_sig) = &cert.signature {
                    let mut key_chain =
                        cert_sig.verify(&body.encode(), current_block, max_depth - 1)?;
                    key_chain.push(body.pubkey.clone());
                    key_chain
                } else {
                    vec![body.pubkey.clone()]
                };
                Ok(key_chain)
            }
            None => return Err(SignatureVerifyError::CertificateMissing),
        }
    }
}

fn verify<T>(pubkey: &[u8], sig: &[u8], msg: &[u8]) -> bool
where
    T: sp_core::crypto::Pair,
{
    T::verify_weak(sig, msg, pubkey)
}

#[derive(Clone, Encode, Decode, Debug)]
pub struct CertificateBody {
    pub pubkey: Vec<u8>,
    pub ttl: u32,
    pub config_bits: u32,
}

impl CertificateBody {
    fn verify(
        &self,
        msg: &[u8],
        sig_type: SignatureType,
        signature: &[u8],
    ) -> Result<(), SignatureVerifyError> {
        let valid = match sig_type {
            SignatureType::Ed25519 => {
                verify::<sp_core::ed25519::Pair>(&self.pubkey, &signature, msg)
            }
            SignatureType::Sr25519 => {
                verify::<sp_core::sr25519::Pair>(&self.pubkey, &signature, msg)
            }
            SignatureType::Ecdsa => verify::<sp_core::ecdsa::Pair>(&self.pubkey, &signature, msg),
        };
        if valid {
            Ok(())
        } else {
            Err(SignatureVerifyError::InvalidSignature)
        }
    }
}

use std::convert::TryFrom;

use alloc::vec;
use alloc::vec::Vec;
use parity_scale_codec::{Decode, Encode, Error as CodecError};

use crate::prpc::{Signature, SignatureType};
pub use phala_crypto::{aead, ecdh, CryptoError};

mod eip712;

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
    InvalidPublicKey,
    Eip712EncodingError,
}

impl From<CodecError> for SignatureVerifyError {
    fn from(err: CodecError) -> Self {
        SignatureVerifyError::DecodeFailed(err)
    }
}

pub enum MessageType {
    Certificate { ttl: u32 },
    ContractQuery,
}

impl Signature {
    /// Verify signature and return the siger pubkey chain in top-down order.
    pub fn verify(
        &self,
        msg: &[u8],
        msg_type: MessageType,
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

                let pubkey = body.recover(msg, msg_type, sig_type, &self.signature)?;

                let key_chain = if let Some(cert_sig) = &cert.signature {
                    let mut key_chain = cert_sig.verify(
                        &body.encode(),
                        MessageType::Certificate { ttl: body.ttl },
                        current_block,
                        max_depth - 1,
                    )?;
                    key_chain.push(pubkey);
                    key_chain
                } else {
                    vec![pubkey]
                };
                Ok(key_chain)
            }
            None => Err(SignatureVerifyError::CertificateMissing),
        }
    }
}

pub fn verify<T>(pubkey: &[u8], sig: &[u8], msg: &[u8]) -> bool
where
    T: sp_core::crypto::Pair,
    T::Public: for<'a> TryFrom<&'a [u8]>,
    T::Signature: for<'a> TryFrom<&'a [u8]>,
{
    let Ok(public) = T::Public::try_from(pubkey) else {
        return false;
    };
    let Ok(signature) = T::Signature::try_from(sig) else {
        return false;
    };
    T::verify(&signature, msg, &public)
}

/// Dummy "recover" function to verify the Substrate signatures and return the public key
fn recover<T>(pubkey: &[u8], sig: &[u8], msg: &[u8]) -> Result<Vec<u8>, SignatureVerifyError>
where
    T: sp_core::crypto::Pair,
    T::Public: for<'a> TryFrom<&'a [u8]>,
    T::Signature: for<'a> TryFrom<&'a [u8]>,
{
    verify::<T>(pubkey, sig, msg)
        .then_some(pubkey.to_vec())
        .ok_or(SignatureVerifyError::InvalidSignature)
}

/// Wraps the message in the same format as it defined in Polkadot.js extension:
///   https://github.com/polkadot-js/extension/blob/e4ce268b1cad5e39e75a2195e3aa6d0344de7745/packages/extension-dapp/src/wrapBytes.ts
fn wrap_bytes(msg: &[u8]) -> Vec<u8> {
    let mut wrapped = Vec::<u8>::new();
    wrapped.extend_from_slice(b"<Bytes>");
    wrapped.extend_from_slice(msg);
    wrapped.extend_from_slice(b"</Bytes>");
    wrapped
}

pub fn ecdsa_recover(
    mut signature: [u8; 65],
    message_hash: [u8; 32],
) -> Result<Vec<u8>, SignatureVerifyError> {
    if signature[64] >= 27 {
        signature[64] -= 27;
    }
    let signature = sp_core::ecdsa::Signature::from_raw(signature);
    let recovered_pubkey = signature
        .recover_prehashed(&message_hash)
        .ok_or(SignatureVerifyError::InvalidSignature)?;
    Ok(recovered_pubkey.as_ref().to_vec())
}

#[derive(Clone, Encode, Decode, Debug)]
pub struct CertificateBody {
    pub pubkey: Vec<u8>,
    pub ttl: u32,
    pub config_bits: u32,
}

impl CertificateBody {
    fn recover(
        &self,
        msg: &[u8],
        msg_type: MessageType,
        sig_type: SignatureType,
        signature: &[u8],
    ) -> Result<Vec<u8>, SignatureVerifyError> {
        match sig_type {
            SignatureType::Ed25519 => {
                recover::<sp_core::ed25519::Pair>(&self.pubkey, signature, msg)
            }
            SignatureType::Sr25519 => {
                recover::<sp_core::sr25519::Pair>(&self.pubkey, signature, msg)
            }
            SignatureType::Ecdsa => recover::<sp_core::ecdsa::Pair>(&self.pubkey, signature, msg),
            SignatureType::Ed25519WrapBytes => {
                let wrapped = wrap_bytes(msg);
                recover::<sp_core::ed25519::Pair>(&self.pubkey, signature, &wrapped)
            }
            SignatureType::Sr25519WrapBytes => {
                let wrapped = wrap_bytes(msg);
                recover::<sp_core::sr25519::Pair>(&self.pubkey, signature, &wrapped)
            }
            SignatureType::EcdsaWrapBytes => {
                let wrapped = wrap_bytes(msg);
                recover::<sp_core::ecdsa::Pair>(&self.pubkey, signature, &wrapped)
            }
            SignatureType::Eip712 => eip712::recover(&self.pubkey, signature, msg, msg_type),
        }
    }
}

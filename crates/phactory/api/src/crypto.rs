use std::convert::TryFrom;

use alloc::vec;
use alloc::vec::Vec;
use parity_scale_codec::{Decode, Encode, Error as CodecError};
use sp_core::{H160, H256};
use sp_runtime::AccountId32;

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

#[derive(Default, Clone, Encode, Decode, Debug)]
pub enum MessageType {
    Certificate { ttl: u32 },
    #[default]
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
    ) -> Result<Vec<AccountId32>, SignatureVerifyError> {
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

                let body_signer = body.recover(msg, msg_type, sig_type, &self.signature)?;

                let signers = if let Some(cert_sig) = &cert.signature {
                    let mut signers = cert_sig.verify(
                        &body.encode(),
                        MessageType::Certificate { ttl: body.ttl },
                        current_block,
                        max_depth - 1,
                    )?;
                    signers.push(body_signer);
                    signers
                } else {
                    vec![body_signer]
                };
                Ok(signers)
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
fn recover<T>(pubkey: &[u8], sig: &[u8], msg: &[u8]) -> Result<T::Public, SignatureVerifyError>
where
    T: sp_core::crypto::Pair,
    T::Public: for<'a> TryFrom<&'a [u8]>,
    T::Signature: for<'a> TryFrom<&'a [u8]>,
{
    let Ok(public) = T::Public::try_from(pubkey) else {
        return Err(SignatureVerifyError::InvalidPublicKey);
    };
    verify::<T>(pubkey, sig, msg)
        .then_some(public)
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

fn evm_ecdsa_recover(
    mut signature: [u8; 65],
    message_hash: [u8; 32],
) -> Result<sp_core::ecdsa::Public, SignatureVerifyError> {
    if signature[64] >= 27 {
        signature[64] -= 27;
    }
    let signature = sp_core::ecdsa::Signature::from_raw(signature);
    let recovered_pubkey = signature
        .recover_prehashed(&message_hash)
        .ok_or(SignatureVerifyError::InvalidSignature)?;
    Ok(recovered_pubkey)
}

/// Convert EVM public key to Substrate account ID.
///
/// account_id = keccak256(pubkey)[12..] + b"@evm_address"
fn account_id_from_evm_pubkey(pubkey: sp_core::ecdsa::Public) -> AccountId32 {
    let pubkey =
        secp256k1::PublicKey::from_slice(pubkey.as_ref()).expect("Should always be a valid pubkey");
    let h32 = H256(sp_core::hashing::keccak_256(
        &pubkey.serialize_uncompressed()[1..],
    ));
    let h20 = H160::from(h32);
    let mut raw_account: [u8; 32] = [0; 32];
    let postfix = b"@evm_address";
    raw_account[..20].copy_from_slice(h20.as_bytes());
    raw_account[20..].copy_from_slice(postfix);
    AccountId32::from(raw_account)
}

#[test]
fn test_account_id_from_evm_pubkey() {
    let pubkey = sp_core::ecdsa::Public(hex_literal::hex!(
        "029df1e69b8b7c2da2efe0069dc141c2cec0317bf3fd135abaeb69ee33801f5970"
    ));
    let account_id = account_id_from_evm_pubkey(pubkey);
    assert_eq!(
        hex::encode(account_id),
        format!(
            "77bb3d64ea13e4f0beafdd5d92508d4643bb09cb{}",
            hex::encode(b"@evm_address")
        )
    );
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
    ) -> Result<AccountId32, SignatureVerifyError> {
        recover_signer_account(&self.pubkey, msg, msg_type, sig_type, signature)
    }
}

pub fn recover_signer_account(
    pubkey: &[u8],
    msg: &[u8],
    msg_type: MessageType,
    sig_type: SignatureType,
    signature: &[u8],
) -> Result<AccountId32, SignatureVerifyError> {
    use account_id_from_evm_pubkey as evm_account;
    let signer = match sig_type {
        SignatureType::Ed25519 => recover::<sp_core::ed25519::Pair>(pubkey, signature, msg)?.into(),
        SignatureType::Sr25519 => recover::<sp_core::sr25519::Pair>(pubkey, signature, msg)?.into(),
        SignatureType::Ecdsa => {
            sp_core::blake2_256(recover::<sp_core::ecdsa::Pair>(pubkey, signature, msg)?.as_ref())
                .into()
        }
        SignatureType::Ed25519WrapBytes => {
            let wrapped = wrap_bytes(msg);
            recover::<sp_core::ed25519::Pair>(pubkey, signature, &wrapped)?.into()
        }
        SignatureType::Sr25519WrapBytes => {
            let wrapped = wrap_bytes(msg);
            recover::<sp_core::sr25519::Pair>(pubkey, signature, &wrapped)?.into()
        }
        SignatureType::EcdsaWrapBytes => {
            let wrapped = wrap_bytes(msg);
            sp_core::blake2_256(
                recover::<sp_core::ecdsa::Pair>(pubkey, signature, &wrapped)?.as_ref(),
            )
            .into()
        }
        SignatureType::Eip712 => evm_account(eip712::recover(pubkey, signature, msg, msg_type)?),
        SignatureType::EvmEcdsa => {
            evm_account(recover::<sp_core::ecdsa::Pair>(pubkey, signature, msg)?)
        }
        SignatureType::EvmEcdsaWrapBytes => {
            let wrapped = wrap_bytes(msg);
            evm_account(recover::<sp_core::ecdsa::Pair>(
                pubkey, signature, &wrapped,
            )?)
        }
    };
    Ok(signer)
}

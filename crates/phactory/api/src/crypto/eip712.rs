use std::convert::TryInto;

use super::{ecdsa_recover, MessageType, SignatureVerifyError};

use ethers::{
    contract::{Eip712, EthAbiType},
    types::{transaction::eip712::Eip712, Bytes},
};

#[derive(Debug, Clone, Eip712, EthAbiType)]
#[eip712(
    name = "Phat Contract Query",
    version = "1",
    salt = "phala/phat-contract"
)]
#[allow(non_snake_case)]
pub struct PhatContractQuery {
    pub encodedQuery: Bytes,
}

#[derive(Debug, Clone, Eip712, EthAbiType)]
#[eip712(
    name = "Phat Query Certificate",
    version = "1",
    salt = "phala/phat-contract"
)]
#[allow(non_snake_case)]
pub struct IssueQueryCertificate {
    pub finalValidBlock: u32,
    pub encodedCert: Bytes,
}

pub(crate) fn recover(
    pubkey: &[u8],
    signature: &[u8],
    msg: &[u8],
    msg_type: MessageType,
) -> Result<Vec<u8>, SignatureVerifyError> {
    let signature = signature
        .try_into()
        .or(Err(SignatureVerifyError::InvalidSignature))?;
    let message: Bytes = msg.to_vec().into();
    let message_hash = match msg_type {
        MessageType::Certificate { ttl } => IssueQueryCertificate {
            finalValidBlock: ttl,
            encodedCert: message,
        }
        .encode_eip712(),
        MessageType::ContractQuery => PhatContractQuery {
            encodedQuery: message,
        }
        .encode_eip712(),
    }
    .or(Err(SignatureVerifyError::Eip712EncodingError))?;
    let recovered_pubkey = ecdsa_recover(signature, message_hash)?;
    if recovered_pubkey != pubkey {
        return Err(SignatureVerifyError::InvalidSignature);
    }
    Ok(sp_core::blake2_256(&recovered_pubkey).to_vec())
}

#[test]
fn signing_cert_works() {
    let message = b"Hello".to_vec();
    let pubkey =
        hex::decode("029df1e69b8b7c2da2efe0069dc141c2cec0317bf3fd135abaeb69ee33801f5970").unwrap();
    let mm_signature = hex::decode("92127c7a62eaaa4e7af9039bb749d250e638a861d36703ae21267e338346fef57bce4f115ac1a517844966ba46caa64305e6acf48db2b6372f558e1a3c9663db1c").unwrap();
    assert!(recover(
        &pubkey,
        &mm_signature,
        &message,
        MessageType::Certificate { ttl: 42 }
    )
    .is_ok());
}

#[test]
fn signing_query_works() {
    let message = b"Hello".to_vec();
    let pubkey =
        hex::decode("029df1e69b8b7c2da2efe0069dc141c2cec0317bf3fd135abaeb69ee33801f5970").unwrap();
    let mm_signature = hex::decode("5b7a4c7db4889547d4cff51d928b95a2a86737d301a0bed7337c112467b138eb704c15d174df08424d4ec5eba50c7335c9a43db8f95f1ad03805d172a8f53b621c").unwrap();
    assert!(recover(&pubkey, &mm_signature, &message, MessageType::ContractQuery).is_ok());
}

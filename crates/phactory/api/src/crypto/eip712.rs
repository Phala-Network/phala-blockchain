use std::convert::TryInto;

use super::{evm_ecdsa_recover, MessageType, SignatureVerifyError};

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
    pub description: String,
    pub encodedQuery: Bytes,
}

impl PhatContractQuery {
    pub fn new(encoded_query: Bytes) -> Self {
        Self {
            description: "You are signing a query request that would be sent to a Phat contract."
                .into(),
            encodedQuery: encoded_query,
        }
    }
}

#[derive(Debug, Clone, Eip712, EthAbiType)]
#[eip712(
    name = "Phat Query Certificate",
    version = "1",
    salt = "phala/phat-contract"
)]
#[allow(non_snake_case)]
pub struct IssueQueryCertificate {
    pub description: String,
    pub timeToLive: String,
    pub encodedCert: Bytes,
}

impl IssueQueryCertificate {
    pub fn new(cert_bytes: Bytes, ttl: u32) -> Self {
        Self {
            description: "You are signing a Certificate that can be used to query Phat contracts using your identity without further prompts.".into(),
            timeToLive: format!("The Certificate will be valid till block {ttl}."),
            encodedCert: cert_bytes,
        }
    }
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
        MessageType::Certificate { ttl } => {
            IssueQueryCertificate::new(message, ttl).encode_eip712()
        }
        MessageType::ContractQuery => PhatContractQuery::new(message).encode_eip712(),
    }
    .or(Err(SignatureVerifyError::Eip712EncodingError))?;
    let recovered_pubkey = evm_ecdsa_recover(signature, message_hash)?;
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
    let mm_signature = hex::decode("878fc9275582e02c0bba158d201bdedcf2adbe3979dbd642a1f23a04ea98d2094a248fa53997d4529d3c0cbd805461f672b37bd76018740b20c868e0f4569c3e1c").unwrap();
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
    let mm_signature = hex::decode("f4f8cd7c6dc211f29d6df58617acf6d0f206a55f5c77f101f2af310204dbf82d5573cf067a2e119642341d185cd646b34ad84f98d27e8835b40812fabd2d94131b").unwrap();
    assert!(recover(&pubkey, &mm_signature, &message, MessageType::ContractQuery).is_ok());
}

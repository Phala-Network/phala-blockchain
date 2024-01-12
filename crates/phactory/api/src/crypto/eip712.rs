use std::convert::TryInto;

use super::{evm_ecdsa_recover, MessageType, SignatureVerifyError};

use ethers::{
    contract::{Eip712, EthAbiType},
    types::{transaction::eip712::Eip712, Bytes},
};
use sp_core::ecdsa::Public;

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
            description: "You are signing a query request that would be sent to a Phat Contract."
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
            description: "You are signing a Certificate that can be used to query Phat Contracts using your identity without further prompts.".into(),
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
) -> Result<Public, SignatureVerifyError> {
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
    if recovered_pubkey.as_ref() != pubkey {
        return Err(SignatureVerifyError::InvalidSignature);
    }
    Ok(recovered_pubkey)
}

#[test]
fn signing_cert_works() {
    let message = b"Hello".to_vec();
    let pubkey =
        hex::decode("029df1e69b8b7c2da2efe0069dc141c2cec0317bf3fd135abaeb69ee33801f5970").unwrap();
    let mm_signature = hex::decode("885074f9b49de7bb73d23a801a69fb160f7cfef50cbf710c7aaff70c5581d7ab63bb7a80e4d8b3983498a9ffb10130fdcb98d5d5c49b7f84e3f99f194c26dcdf1b").unwrap();
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
    let mm_signature = hex::decode("b0ba4176ef624a71837c1a63e9c502c02314414883413913c126e05bfb3fda60474e0758b49c91647cd735c11d2a9575647509bdd2aef4eb5c5eba0019463a681c").unwrap();
    assert!(recover(&pubkey, &mm_signature, &message, MessageType::ContractQuery).is_ok());
}

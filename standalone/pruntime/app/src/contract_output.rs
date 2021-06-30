use serde_json::{Map, Value, Error};
use crate::attestation::Attestation;

#[derive(Serialize, Deserialize)]
pub struct ContractOutput {
    pub output: String,
    pub nonce: Map<String, Value>,
    pub attestation: Attestation
}

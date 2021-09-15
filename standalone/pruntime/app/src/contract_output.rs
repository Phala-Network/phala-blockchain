use crate::attestation::Attestation;
use serde_json::{Map, Value};

#[derive(Serialize, Deserialize)]
pub struct ContractOutput {
    pub output: String,
    pub nonce: Map<String, Value>,
    pub attestation: Attestation,
}

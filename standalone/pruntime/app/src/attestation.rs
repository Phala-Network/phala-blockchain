use serde_json::{Map, Value};

#[derive(Serialize, Deserialize)]
pub struct Attestation {
    pub version: u8,
    pub provider: String,
    pub payload: Map<String, Value>,
}

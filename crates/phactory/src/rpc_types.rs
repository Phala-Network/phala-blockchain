use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct InitRuntimeReq {
    pub skip_ra: bool,
    pub bridge_genesis_info_b64: String,
    pub debug_set_key: Option<String>,
    pub genesis_state_b64: String,
    pub operator_hex: Option<String>,
    pub is_parachain: bool,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InitRuntimeResp {
    pub encoded_runtime_info: Vec<u8>,
    pub genesis_block_hash: String,
    pub public_key: String,
    pub ecdh_public_key: String,
    pub attestation: Option<InitRespAttestation>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InitRespAttestation {
    pub version: i32,
    pub provider: String,
    pub payload: AttestationReport,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AttestationReport {
    pub report: String,
    pub signature: String,
    pub signing_cert: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TestReq {
    pub test_parse_block: Option<bool>,
    pub test_ecdh: Option<TestEcdhParam>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct TestEcdhParam {
    pub pubkey_hex: Option<String>,
    pub message_b64: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SyncHeaderReq {
    pub headers_b64: Vec<String>,
    pub authority_set_change_b64: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DispatchBlockReq {
    pub blocks_b64: Vec<String>,
}

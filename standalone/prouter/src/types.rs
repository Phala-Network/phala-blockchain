use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[repr(u8)]
pub enum PRouterRequestMethod {
    GET = 0,
    POST,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PRouterSendJsonRequest {
    pub target_pubkey: [u8; 32],
    pub path: String,
    pub method: PRouterRequestMethod,
    pub data: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PRouterSendJsonResponse {
    pub status: u8,
    pub msg: Vec<u8>,
}

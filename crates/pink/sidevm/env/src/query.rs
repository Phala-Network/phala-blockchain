use scale::{Decode, Encode};

pub type AccountId = [u8; 32];

#[derive(Encode, Decode)]
pub struct QueryRequest {
    pub origin: AccountId,
    pub payload: Vec<u8>,
    pub reply_tx: i32,
}

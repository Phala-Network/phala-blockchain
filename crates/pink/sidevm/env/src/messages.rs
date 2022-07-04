use scale::{Decode, Encode};

pub type AccountId = [u8; 32];
pub type H256 = [u8; 32];

#[derive(Encode, Decode)]
pub struct QueryRequest {
    pub origin: Option<AccountId>,
    pub payload: Vec<u8>,
    pub reply_tx: i32,
}

#[derive(Encode, Decode)]
#[non_exhaustive]
pub enum SystemMessage {
    PinkLog {
        block_number: u32,
        contract: AccountId,
        in_query: bool,
        timestamp_ms: u64,
        level: u8,
        message: String,
    },
    PinkEvent {
        block_number: u32,
        contract: AccountId,
        topics: Vec<H256>,
        payload: Vec<u8>,
    },
    PinkMessageOutput {
        block_number: u32,
        origin: AccountId,
        contract: AccountId,
        output: Vec<u8>,
    },
}

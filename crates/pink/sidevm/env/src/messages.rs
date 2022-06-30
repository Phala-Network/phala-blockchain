use scale::{Decode, Encode};

pub type AccountId = [u8; 32];

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
        in_query: bool,
        from: AccountId,
        level: u8,
        message: String,
    },
}

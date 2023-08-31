use alloc::string::String;
use parity_scale_codec::{Decode, Encode};

#[derive(Debug, Encode, Decode)]
pub enum QueryError {
    BadOrigin,
    RuntimeError(String),
    SidevmNotFound,
    NoResponse,
    ServiceUnavailable,
    Timeout,
}

impl std::error::Error for QueryError {}
impl std::fmt::Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryError::BadOrigin => write!(f, "Bad origin"),
            QueryError::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
            QueryError::SidevmNotFound => write!(f, "Sidevm not found"),
            QueryError::NoResponse => write!(f, "No response"),
            QueryError::ServiceUnavailable => write!(f, "Service unavailable"),
            QueryError::Timeout => write!(f, "Timeout"),
        }
    }
}


#[derive(Debug, Encode, Decode)]
pub enum Query {
    InkMessage {
        payload: Vec<u8>,
        /// Amount of tokens deposit to the caller.
        deposit: u128,
        /// Amount of tokens transfer from the caller to the target contract.
        transfer: u128,
        /// Whether to use the gas estimation mode.
        estimating: bool,
    },
    SidevmQuery(Vec<u8>),
    InkInstantiate {
        code_hash: sp_core::H256,
        salt: Vec<u8>,
        instantiate_data: Vec<u8>,
        /// Amount of tokens deposit to the caller.
        deposit: u128,
        /// Amount of tokens transfer from the caller to the target contract.
        transfer: u128,
    },
}

#[derive(Debug, Encode, Decode)]
pub enum Response {
    Payload(Vec<u8>),
}

use scale::{Decode, Encode};

use crate::OcallError;

pub type AccountId = [u8; 32];
pub type H256 = [u8; 32];

#[derive(Encode, Decode)]
pub struct QueryRequest {
    pub origin: Option<AccountId>,
    pub payload: Vec<u8>,
    pub reply_tx: i32,
}

#[derive(Encode, Decode, Debug)]
pub struct HttpHead {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
}

impl HttpHead {
    pub fn get_header(&self, key: &str) -> Option<&str> {
        let key = key.to_ascii_lowercase();
        for (k, v) in &self.headers {
            if k.to_ascii_lowercase() == key {
                return Some(v);
            }
        }
        None
    }
}

#[derive(Encode, Decode, Debug)]
pub struct HttpRequest {
    pub head: HttpHead,
    pub response_tx: i32,
    pub io_stream: i32,
}

#[derive(Encode, Decode, Debug)]
pub struct HttpResponseHead {
    pub status: u16,
    pub headers: Vec<(String, String)>,
}

#[derive(Encode, Decode)]
pub enum SystemMessage {
    PinkLog {
        block_number: u32,
        contract: AccountId,
        entry: AccountId,
        exec_mode: String,
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
        nonce: Vec<u8>,
        output: Vec<u8>,
    },
    Metric(Metric),
}

#[derive(Encode, Decode)]
pub enum Metric {
    PinkQueryIn([u8; 8]),
}

/// Errors that may occur during the contract query.
#[derive(Debug, Encode, Decode)]
pub enum QueryError {
    /// Ocall error.
    OcallError(OcallError),
    /// The origin is invalid.
    BadOrigin,
    /// An error occurred during the contract execution.
    RuntimeError(String),
    /// The contract is not found.
    SidevmNotFound,
    /// No response received from the contract. Maybe the contract was panicked during execution.
    NoResponse,
    /// The contract is busy and cannot process the request.
    ServiceUnavailable,
    /// The request is timeout.
    Timeout,
    /// Signature is invalid.
    InvalidSignature,
    /// No such contract.
    ContractNotFound,
    /// Unable to decode the request data.
    DecodeError,
    /// Other errors reported during the contract query execution.
    OtherError(String),
    /// The operation is unsupported.
    Unsupported,
    /// Failed to decode the ContractExecResult.
    InvalidContractExecResult,
    /// Error reported by the pink runtime.
    DispatchError(String),
}

impl From<OcallError> for QueryError {
    fn from(err: OcallError) -> Self {
        Self::OcallError(err)
    }
}

#[derive(Encode, Decode)]
pub enum QueryResponse {
    OutputWithGasEstimation {
        output: Vec<u8>,
        gas_consumed: u64,
        gas_required: u64,
        storage_deposit_value: u128,
        storage_deposit_is_charge: bool,
    },
    SimpleOutput(Vec<u8>),
}

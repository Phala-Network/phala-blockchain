//! Support for query to local contract

use scale::{Decode, Encode};
use sidevm_env::messages::{QueryError, QueryResponse};

use crate::{channel, ocall};

/// Query a local contract.
///
/// # Limitation
/// Only one query can be processed at a time.
pub async fn query_pink(contract_id: [u8; 32], payload: Vec<u8>) -> Result<Vec<u8>, QueryError> {
    #[derive(Debug, Encode)]
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
    }
    let query = Query::InkMessage {
        payload,
        deposit: 0,
        transfer: 0,
        estimating: false,
    };
    let res = ocall::query_local_contract(contract_id, query.encode())?;
    let rx = channel::Receiver::<Vec<u8>>::new(res.into());

    let Some(reply) = rx.next().await else {
        return Err(sidevm_env::OcallError::EndOfFile.into());
    };
    let result = Result::<QueryResponse, QueryError>::decode(&mut &reply[..])
        .map_err(|_| QueryError::DecodeError)??;
    match result {
        QueryResponse::EstimatedOutput { output, .. } | QueryResponse::SimpleOutput(output) => {
            Ok(output)
        }
    }
}

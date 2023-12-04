//! Support for query to local contract

use scale::{Decode, Encode};
use sidevm_env::messages::{QueryError, QueryResponse};

use crate::{channel, ocall};

/// Query a local contract within the same worker.
///
/// The call's origin would be `blake2_256(vmid ++ '/sidevm')`, where vmid always equals
/// its contract id.
///
/// # Limitation
///
/// Only one query can be handled simultaneously. A second simultaneous query will be rejected
/// with error `OcallError::ResourceLimited``. If you need to perform parallel queries in
/// coroutines, create a single coroutine to proxy and sequence the queries.
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
        QueryResponse::OutputWithGasEstimation { output, .. } | QueryResponse::SimpleOutput(output) => {
            Ok(output)
        }
    }
}

//! Support for query to local contract

use crate::{channel, ocall};

/// Query a local pink contract.
///
/// # Limitation
/// Only one query can be processed at a time.
pub async fn query_pink<T: scale::Decode>(
    contract_id: [u8; 32],
    payload: Vec<u8>,
) -> Result<Vec<u8>, sidevm_env::OcallError> {
    let res = ocall::query_local_contract(contract_id, payload)?;
    let rx = channel::Receiver::<Vec<u8>>::new(res.into());

    let Some(reply) = rx.next().await else {
        return Err(sidevm_env::OcallError::EndOfFile);
    };

    Ok(reply)
}

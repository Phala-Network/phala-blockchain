use crate::std::fmt::Debug;
use crate::std::vec::Vec;
use anyhow::Result;
use core::fmt;
use parity_scale_codec::{Decode, Encode, Error as CodecError};
use phala_types::contract::ContractQueryError;
use thiserror::Error;

extern crate runtime as chain;

// supportive

pub struct BlockInfo<'a> {
    /// The block number.
    pub block_number: chain::BlockNumber,
    /// The timestamp of this block.
    pub now_ms: u64,
    /// The storage snapshot after this block executed.
    pub storage: &'a crate::Storage,
    /// The message queue
    pub recv_mq: &'a mut phala_mq::MessageDispatcher,
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct TxRef {
    pub blocknum: chain::BlockNumber,
    pub index: u64,
}

// for contracts
pub type OpaqueQuery<'a> = &'a [u8];
pub type OpaqueReply = Vec<u8>;
pub type OpaqueError = ContractQueryError;

pub fn deopaque_query<T>(mut data: &[u8]) -> Result<T, ContractQueryError>
where
    T: Decode + Debug,
{
    Decode::decode(&mut data).or(Err(ContractQueryError::DecodeError))
}

#[derive(Debug, Error)]
#[error("{:?}", self)]
pub enum Error {
    IoError(#[from] anyhow::Error),
    DecodeError(#[from] CodecError),
    PersistentRuntimeNotFound,
}

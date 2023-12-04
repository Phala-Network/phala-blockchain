use anyhow::Result;
use parity_scale_codec::{Decode, Encode, Error as CodecError};
use phala_types::contract::ContractQueryError;
use sidevm::service::Spawner;
use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};
use thiserror::Error;

extern crate runtime as chain;

// supportive

pub struct BaseBlockInfo<'a> {
    /// The block number.
    pub block_number: chain::BlockNumber,
    /// The timestamp of this block.
    pub now_ms: u64,
    /// The storage snapshot after this block executed.
    pub storage: &'a crate::ChainStorage,
    /// The message queue
    pub send_mq: &'a phala_mq::MessageSendQueue,
    pub recv_mq: &'a mut phala_mq::MessageDispatcher,
}

pub struct BlockInfo<'a> {
    pub base: BaseBlockInfo<'a>,
    pub sidevm_spawner: &'a Spawner,
}

impl<'a> Deref for BlockInfo<'a> {
    type Target = BaseBlockInfo<'a>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for BlockInfo<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct TxRef {
    pub blocknum: chain::BlockNumber,
    pub index: u64,
}

// for contracts
pub type OpaqueQuery = Vec<u8>;
pub type OpaqueError = ContractQueryError;

pub fn deopaque_query<T>(mut data: &[u8]) -> Result<T, ContractQueryError>
where
    T: Decode + Debug,
{
    Decode::decode(&mut data).or(Err(ContractQueryError::DecodeError))
}

#[derive(Debug, Error)]
#[error("{:?}", self)]
#[allow(clippy::enum_variant_names)]
pub enum Error {
    IoError(#[from] anyhow::Error),
    DecodeError(#[from] CodecError),
    PersistentRuntimeNotFound,
}

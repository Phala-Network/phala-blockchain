use std::fmt::Debug;
use anyhow::Result;
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
    pub send_mq: &'a phala_mq::MessageSendQueue,
    pub recv_mq: &'a mut phala_mq::MessageDispatcher,
    /// The side-task manager.
    pub side_task_man: &'a mut crate::side_task::SideTaskManager,
}

#[cfg(test)]
impl BlockInfo<'_> {
    pub(crate) fn builder() -> BlockInfoBuilder {
        Default::default()
    }
}

#[cfg(test)]
#[derive(Default)]
pub(crate) struct BlockInfoBuilder {
    /// The block number.
    pub block_number: chain::BlockNumber,
    /// The timestamp of this block.
    pub now_ms: u64,
    pub storage: crate::Storage,
    /// The message queue
    pub send_mq: phala_mq::MessageSendQueue,
    pub recv_mq: phala_mq::MessageDispatcher,
    /// The side-task manager.
    pub side_task_man: crate::side_task::SideTaskManager,
}

#[cfg(test)]
impl BlockInfoBuilder {
    pub fn block_number(self, block_number: chain::BlockNumber) -> Self {
        Self {
            block_number,
            ..self
        }
    }
    pub fn now_ms(self, now_ms: u64) -> Self {
        Self {
            now_ms,
            ..self
        }
    }
    pub fn build(&mut self) -> BlockInfo {
        BlockInfo {
            block_number: self.block_number,
            now_ms: self.now_ms,
            storage: &self.storage,
            send_mq: &self.send_mq,
            recv_mq: &mut self.recv_mq,
            side_task_man: &mut self.side_task_man,
        }
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct TxRef {
    pub blocknum: chain::BlockNumber,
    pub index: u64,
}

// for contracts
pub type OpaqueQuery = Vec<u8>;
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
#[allow(clippy::enum_variant_names)]
pub enum Error {
    IoError(#[from] anyhow::Error),
    DecodeError(#[from] CodecError),
    PersistentRuntimeNotFound,
}

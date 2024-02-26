use core::fmt;
use phactory_api::{
    blocks::{BlockHeader, StorageProof},
    pruntime_client,
};
use serde::{Deserialize, Serialize};
use sp_runtime::{generic::SignedBlock as SpSignedBlock, Justifications, OpaqueExtrinsic};

use phaxt::{subxt::rpc::types::Justification, Config};

pub use phaxt::rpc::{self, StorageData, StorageKey};
pub use phaxt::{self, subxt, ParachainApi, RelaychainApi};
pub use subxt::rpc::types::NumberOrHex;

use subxt::rpc::types::ChainBlockResponse;

use codec::{Decode, Encode};

use crate::headers_cache::BlockInfo;

pub type PrClient = pruntime_client::PRuntimeClient;
pub type SrSigner = phaxt::PairSigner;

pub type SignedBlock<Hdr, Ext> = SpSignedBlock<sp_runtime::generic::Block<Hdr, Ext>>;

pub type BlockNumber = u32;
pub type Hash = sp_core::H256;
pub type Header = sp_runtime::generic::Header<BlockNumber, sp_runtime::traits::BlakeTwo256>;
pub type Block = SignedBlock<Header, OpaqueExtrinsic>;
pub type UnsigedBlock = sp_runtime::generic::Block<Header, OpaqueExtrinsic>;

// API: notify
#[derive(Serialize, Deserialize, Debug)]
pub struct NotifyReq {
    pub headernum: BlockNumber,
    pub blocknum: BlockNumber,
    pub pruntime_initialized: bool,
    pub pruntime_new_init: bool,
    pub initial_sync_finished: bool,
}

pub mod utils {
    use super::StorageProof;
    use phaxt::subxt::rpc::types::ReadProof;
    pub fn raw_proof<T>(read_proof: ReadProof<T>) -> StorageProof {
        read_proof.proof.into_iter().map(|p| p.0).collect()
    }
}

pub enum SyncOperation {
    RelaychainHeader,
    CachedRelaychainHeader(Vec<BlockInfo>),
    ParachainHeader((u32, Vec<Vec<u8>>)),
    Block,
    ReachedChainTip,
}

impl fmt::Display for SyncOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyncOperation::RelaychainHeader => write!(f, "RelaychainHeader"),
            SyncOperation::CachedRelaychainHeader(cached_headers) =>
                write!(f, "RelaychainHeader with {} cached headers", cached_headers.len()),
            SyncOperation::ParachainHeader((para_number, _)) =>
                write!(f, "ParachainHeader to block #{}", para_number),
            SyncOperation::Block => write!(f, "Block"),
            SyncOperation::ReachedChainTip => write!(f, "ReachedChainTip"),
        }
    }
}

pub trait ConvertTo<T> {
    fn convert_to(&self) -> T;
}

fn recode<F: Encode, T: Decode>(f: &F) -> Result<T, codec::Error> {
    Decode::decode(&mut &f.encode()[..])
}

impl<H> ConvertTo<BlockHeader> for H
where
    H: subxt::config::Header,
{
    fn convert_to(&self) -> BlockHeader {
        recode(self).expect("Failed to convert subxt header to block header")
    }
}

impl ConvertTo<Block> for ChainBlockResponse<Config> {
    fn convert_to(&self) -> Block {
        Block {
            block: sp_runtime::generic::Block {
                header: self.block.header.convert_to(),
                extrinsics: vec![],
            },
            justifications: self.justifications.as_ref().map(|x| x.convert_to()),
        }
    }
}

impl ConvertTo<Justifications> for Vec<Justification> {
    fn convert_to(&self) -> Justifications {
        recode(self).expect("Failed to convert ChainBlockResponse to Block")
    }
}
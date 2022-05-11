use phactory_api::{blocks::StorageProof, pruntime_client};
use serde::{Deserialize, Serialize};
use sp_core::sr25519;
use sp_runtime::{generic::SignedBlock as SpSignedBlock, OpaqueExtrinsic};

pub use sp_core::storage::{StorageData, StorageKey};

pub use khala::runtime_types::phala_mq::types::*;
pub use phaxt::{self, *};
pub use subxt::rpc::NumberOrHex;

pub type PrClient = pruntime_client::PRuntimeClient;
pub type SrSigner = subxt::PairSigner<phaxt::Config, sr25519::Pair>;

pub type SignedBlock<Hdr, Ext> = SpSignedBlock<sp_runtime::generic::Block<Hdr, Ext>>;

pub type Block = SignedBlock<Header, OpaqueExtrinsic>;
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
    use phaxt::subxt::ReadProof;
    pub fn raw_proof<T>(read_proof: ReadProof<T>) -> StorageProof {
        read_proof.proof.into_iter().map(|p| p.0).collect()
    }
}

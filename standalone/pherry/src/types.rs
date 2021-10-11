use phactory_api::{blocks::{StorageChanges, StorageProof}, pruntime_client};
use serde::{Deserialize, Serialize};
use sp_core::sr25519;
use sp_runtime::{generic::SignedBlock, OpaqueExtrinsic};

pub use sp_rpc::number::NumberOrHex;

// Node Runtime

use crate::runtimes::PhalaNodeRuntime;
pub type Runtime = PhalaNodeRuntime;
pub type Header = <Runtime as subxt::system::System>::Header;
pub type Hash = <Runtime as subxt::system::System>::Hash;
pub type Hashing = <Runtime as subxt::system::System>::Hashing;
pub type OpaqueBlock = sp_runtime::generic::Block<Header, OpaqueExtrinsic>;
pub type OpaqueSignedBlock = SignedBlock<OpaqueBlock>;
pub type BlockNumber = <Runtime as subxt::system::System>::BlockNumber;

pub type XtClient = subxt::Client<Runtime>;
pub type PrClient = pruntime_client::PRuntimeClient;
pub type SrSigner = subxt::PairSigner<Runtime, sr25519::Pair>;

#[derive(Clone, Debug)]
pub struct BlockWithChanges {
    pub block: OpaqueSignedBlock,
    pub storage_changes: StorageChanges,
}

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
    use subxt::ReadProof;
    pub fn raw_proof<T>(read_proof: ReadProof<T>) -> StorageProof {
        read_proof.proof.into_iter().map(|p| p.0).collect()
    }
}

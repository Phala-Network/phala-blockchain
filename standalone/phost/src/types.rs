use serde::{de::DeserializeOwned, Deserialize, Serialize};

use codec::{Decode, Encode};
use sp_finality_grandpa::AuthorityList;
use sp_runtime::{generic::SignedBlock, OpaqueExtrinsic};

pub(crate) use enclave_api::blocks::{
    AuthoritySet, AuthoritySetChange, BlockHeaderWithChanges, HeaderToSync, StorageProof,
};
use trie_storage::ser::StorageChanges;

use std::vec::Vec;

// Node Runtime

use crate::runtimes::PhalaNodeRuntime;
pub type Runtime = PhalaNodeRuntime;
pub type Header = <Runtime as subxt::system::System>::Header;
pub type Hash = <Runtime as subxt::system::System>::Hash;
pub type OpaqueBlock = sp_runtime::generic::Block<Header, OpaqueExtrinsic>;
pub type OpaqueSignedBlock = SignedBlock<OpaqueBlock>;
pub type BlockNumber = <Runtime as subxt::system::System>::BlockNumber;

// pRuntime APIs

pub trait Resp {
    type Resp: DeserializeOwned;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SignedResp {
    pub payload: String,
    pub status: String,
    pub signature: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Nonce {
    value: u32,
}

impl Nonce {
    pub fn new() -> Nonce {
        Nonce {
            value: rand::random::<u32>(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RuntimeReq<T: Serialize> {
    pub input: T,
    pub nonce: Nonce,
}
impl<T: Serialize> RuntimeReq<T> {
    pub fn new(input: T) -> Self {
        Self {
            input: input,
            nonce: Nonce::new(),
        }
    }
}

// API: get_info

#[derive(Serialize, Deserialize, Debug)]
pub struct GetInfoReq {}
#[derive(Serialize, Deserialize, Debug)]
pub struct GetInfoResp {
    pub headernum: BlockNumber,
    pub para_headernum: BlockNumber,
    pub blocknum: BlockNumber,
    pub initialized: bool,
    pub public_key: String,
    pub ecdh_public_key: String,
    pub machine_id: Vec<u8>,
}

impl Resp for GetInfoReq {
    type Resp = GetInfoResp;
}

// API: init_runtime

#[derive(Serialize, Deserialize, Debug)]
pub struct InitRuntimeReq {
    pub skip_ra: bool,
    pub bridge_genesis_info_b64: String,
    pub debug_set_key: Option<String>,
    pub genesis_state_b64: String,
    pub operator_hex: Option<String>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct InitRuntimeResp {
    pub encoded_runtime_info: Vec<u8>,
    pub public_key: String,
    pub ecdh_public_key: String,
    pub attestation: Option<InitRespAttestation>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct InitRespAttestation {
    pub version: i32,
    pub provider: String,
    pub payload: AttestationReport,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct AttestationReport {
    pub report: String,
    pub signature: String,
    pub signing_cert: String,
}
impl Resp for InitRuntimeReq {
    type Resp = InitRuntimeResp;
}
#[derive(Serialize, Deserialize, Debug)]
pub struct GetRuntimeInfoReq {}
impl Resp for GetRuntimeInfoReq {
    type Resp = InitRuntimeResp;
}

#[derive(Encode, Decode)]
pub struct GenesisInfo {
    pub header: Header,
    pub validators: AuthorityList,
    pub proof: StorageProof,
}

// API: sync_header

#[derive(Serialize, Deserialize, Debug)]
pub struct SyncHeaderReq {
    pub headers_b64: Vec<String>,
    pub authority_set_change_b64: Option<String>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct SyncHeaderResp {
    pub synced_to: BlockNumber,
}
impl Resp for SyncHeaderReq {
    type Resp = SyncHeaderResp;
}

#[derive(Clone, Debug)]
pub struct BlockWithChanges {
    pub block: OpaqueSignedBlock,
    pub storage_changes: StorageChanges,
}

// API: dispatch_block

#[derive(Serialize, Deserialize, Debug)]
pub struct DispatchBlockReq {
    pub blocks_b64: Vec<String>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct DispatchBlockResp {
    pub dispatched_to: BlockNumber,
}
impl Resp for DispatchBlockReq {
    type Resp = DispatchBlockResp;
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

// API: get_egress_messages
#[derive(Serialize, Deserialize, Debug)]
pub struct GetEgressMessagesReq {}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetEgressMessagesResp {
    pub messages: String,
}
impl Resp for GetEgressMessagesReq {
    type Resp = GetEgressMessagesResp;
}

pub mod utils {
    use super::StorageProof;
    use subxt::ReadProof;
    pub fn raw_proof<T>(read_proof: ReadProof<T>) -> StorageProof {
        read_proof.proof.into_iter().map(|p| p.0).collect()
    }
}

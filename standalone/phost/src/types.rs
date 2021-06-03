use serde::{de::DeserializeOwned, Deserialize, Serialize};

use codec::{Decode, Encode};
use sp_finality_grandpa::{AuthorityList, SetId};
use sp_runtime::{generic::SignedBlock, OpaqueExtrinsic};

use phala_types::pruntime::{self, StorageProof};
use trie_storage::ser::StorageChanges;

use std::vec::Vec;

// Node Runtime

use crate::runtimes::PhalaNodeRuntime;
pub type Runtime = PhalaNodeRuntime;
pub type Header = <Runtime as subxt::system::System>::Header;
pub type Hash = <Runtime as subxt::system::System>::Hash;
pub type Hashing = <Runtime as subxt::system::System>::Hashing;
pub type OpaqueBlock = sp_runtime::generic::Block<Header, OpaqueExtrinsic>;
pub type OpaqueSignedBlock = SignedBlock<OpaqueBlock>;
pub type BlockNumber = <Runtime as subxt::system::System>::BlockNumber;
pub type AccountId = <Runtime as subxt::system::System>::AccountId;
pub type Balance = <Runtime as subxt::balances::Balances>::Balance;

pub type RawEvents = Vec<u8>;
pub type HeaderToSync = pruntime::HeaderToSync<BlockNumber, Hashing>;
pub type BlockHeaderWithEvents = pruntime::BlockHeaderWithEvents<BlockNumber, Hashing, Balance>;

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
    pub blocknum: BlockNumber,
    pub initialized: bool,
    pub public_key: String,
    pub ecdh_public_key: String,
    pub machine_id: Vec<u8>,
    pub system_egress: EgressInfo,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct EgressInfo {
    sequence: u64,
    len: u64,
}
impl Resp for GetInfoReq {
    type Resp = GetInfoResp;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Payload {
    Plain(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Query {
    pub contract_id: u32,
    pub nonce: u32,
    pub request: ReqData,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ReqData {
    PendingChainTransfer { sequence: u64 },  // Balances
    PendingKittyTransfer { sequence: u64 },  // Kitties
    PendingLotteryEgress { sequence: u64 },  // Btc lottery
    GetWorkerEgress { start_sequence: u64 }, // System
}

#[derive(Serialize, Deserialize, Debug)]
pub enum QueryRespData {
    PendingChainTransfer {
        transfer_queue_b64: String,
    },
    PendingKittyTransfer {
        transfer_queue_b64: String,
    },
    PendingLotteryEgress {
        lottery_queue_b64: String,
    },
    GetWorkerEgress {
        length: usize,
        encoded_egress_b64: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryReq {
    pub query_payload: String,
}
impl Resp for QueryReq {
    type Resp = Payload;
}

#[derive(Serialize, Deserialize, Debug, Encode, Decode)]
pub struct Transfer {
    pub dest: [u8; 32],
    pub amount: u128,
    pub sequence: u64,
}
#[derive(Serialize, Deserialize, Debug, Encode, Decode)]
pub struct TransferData {
    pub data: Transfer,
    pub signature: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Encode, Decode)]
pub struct KittyTransfer {
    pub dest: [u8; 32],
    pub kitty_id: Vec<u8>,
    pub sequence: u64,
}
#[derive(Serialize, Deserialize, Debug, Encode, Decode)]
pub struct KittyTransferData {
    pub data: KittyTransfer,
    pub signature: Vec<u8>,
}

// API: init_runtime

#[derive(Serialize, Deserialize, Debug)]
pub struct InitRuntimeReq {
    pub skip_ra: bool,
    pub bridge_genesis_info_b64: String,
    pub debug_set_key: Option<String>,
    pub genesis_state: Vec<(Vec<u8>, Vec<u8>)>, // TODO: serialize efficiently.
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
pub struct BlockWithEvents {
    pub block: OpaqueSignedBlock,
    pub events: Option<Vec<u8>>,
    pub proof: Option<StorageProof>,
    pub storage_changes: StorageChanges,
}

#[derive(Encode, Decode, Clone, PartialEq, Debug)]
pub struct AuthoritySet {
    pub authority_set: AuthorityList,
    pub set_id: SetId,
}
#[derive(Encode, Decode, Clone, PartialEq)]
pub struct AuthoritySetChange {
    pub authority_set: AuthoritySet,
    pub authority_proof: StorageProof,
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

pub mod utils {
    use super::StorageProof;
    use subxt::ReadProof;
    pub fn raw_proof<T>(read_proof: ReadProof<T>) -> StorageProof {
        read_proof.proof.into_iter().map(|p| p.0).collect()
    }
}

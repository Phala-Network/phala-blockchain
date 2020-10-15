use serde::{Serialize, Deserialize, de::DeserializeOwned};

use codec::{Encode, Decode};
use sp_finality_grandpa::{AuthorityList, SetId};
use sp_runtime::{
    generic::SignedBlock,
    Justification,
    OpaqueExtrinsic
};

use std::vec::Vec;

// Node Runtime

use crate::runtimes::PhalaNodeRuntime;
pub type Runtime = PhalaNodeRuntime;
pub type Header = <Runtime as subxt::system::System>::Header;
pub type Hash = <Runtime as subxt::system::System>::Hash;
pub type OpaqueBlock = sp_runtime::generic::Block<Header, OpaqueExtrinsic>;
pub type OpaqueSignedBlock = SignedBlock<OpaqueBlock>;
pub type BlockNumber = <Runtime as subxt::system::System>::BlockNumber;

pub type RawStorageKey = Vec<u8>;
pub type StorageProof = Vec<Vec<u8>>;
pub type RawEvents = Vec<u8>;

// pRuntime APIs

pub trait Resp {
    type Resp: DeserializeOwned;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SignedResp {
    pub payload: String,
    pub status: String,
    pub signature: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Nonce {
    value: u32,
}

impl Nonce {
    pub fn new() -> Nonce {
        Nonce { value: rand::random::<u32>() }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RuntimeReq<T: Serialize> {
    pub input: T,
    pub nonce: Nonce,
}
impl<T: Serialize> RuntimeReq<T> {
    pub fn new(input: T) -> Self {
        Self { input: input, nonce: Nonce::new() }
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
    PendingChainTransfer {sequence: u64}
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryReq {
    pub query_payload: String,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct QueryResp {
    pub plain: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct PendingChainTransfer {
    pub pending_chain_transfer: TransferQueue,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TransferQueue {
    pub transfer_queue_b64: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[derive(Encode, Decode)]
pub struct Transfer {
    pub dest: [u8; 32],
    pub amount: u128,
    pub sequence: u64,
}
#[derive(Serialize, Deserialize, Debug)]
#[derive(Encode, Decode)]
pub struct TransferData {
    pub data: Transfer,
    pub signature: Vec<u8>,
}

impl Resp for QueryReq {
    type Resp = QueryResp;
}

// API: init_runtime

#[derive(Serialize, Deserialize, Debug)]
pub struct InitRuntimeReq {
  pub skip_ra: bool,
  pub bridge_genesis_info_b64: String,
  pub debug_set_key: Option<String>,
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
    pub authority_set_change_b64: Option<String>
}
#[derive(Serialize, Deserialize, Debug)]
pub struct SyncHeaderResp {
    pub synced_to: BlockNumber
}
impl Resp for SyncHeaderReq {
    type Resp = SyncHeaderResp;
}
#[derive(Encode, Decode, Clone, Debug)]
pub struct BlockWithEvents {
    pub block: OpaqueSignedBlock,
    pub events: Option<Vec<u8>>,
    pub proof: Option<StorageProof>,
    pub key: Option<Vec<u8>>,
}
#[derive(Encode, Decode, Clone, Debug)]
pub struct HeaderToSync {
    pub header: Header,
    pub justification: Option<Justification>,
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

// API: ping

#[derive(Serialize, Deserialize, Debug, Clone)]
#[derive(Encode, Decode)]
pub struct Heartbeat {
    pub block_num: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[derive(Encode, Decode)]
pub struct HeartbeatData {
    pub data: Heartbeat,
    pub signature: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PingReq {}
#[derive(Serialize, Deserialize, Debug)]
pub struct PingResp {
    pub status: String,
    pub encoded_data: String
}
impl Resp for PingReq {
    type Resp = PingResp;
}

// API: dispatch_block

#[derive(Serialize, Deserialize, Debug)]
pub struct DispatchBlockReq {
    pub blocks_b64: Vec<String>
}
#[derive(Serialize, Deserialize, Debug)]
pub struct DispatchBlockResp {
    pub dispatched_to: BlockNumber
}
impl Resp for DispatchBlockReq {
    type Resp = DispatchBlockResp;
}

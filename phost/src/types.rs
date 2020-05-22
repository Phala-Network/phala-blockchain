use serde::{Serialize, Deserialize, de::DeserializeOwned};

use codec::{Encode, Decode};
use sp_finality_grandpa::AuthorityList;
use sp_runtime::{
    generic::SignedBlock,
    OpaqueExtrinsic
};


// Nod Runtime

use crate::runtimes::PhalaNodeRuntime;
pub type Runtime = PhalaNodeRuntime;
pub type Header = <Runtime as subxt::system::System>::Header;
pub type Hash = <Runtime as subxt::system::System>::Hash;
pub type OpaqueBlock = sp_runtime::generic::Block<Header, OpaqueExtrinsic>;
pub type OpaqueSignedBlock = SignedBlock<OpaqueBlock>;


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
    pub blocknum: phala_node_runtime::BlockNumber,
    pub initialized: bool,
    pub public_key: String,
    pub ecdh_public_key: String,
}
impl Resp for GetInfoReq {
    type Resp = GetInfoResp;
}

// API: init_runtime

#[derive(Serialize, Deserialize, Debug)]
pub struct InitRuntimeReq {
    pub skip_ra: bool,
    pub bridge_genesis_info_b64: String
}
#[derive(Serialize, Deserialize, Debug)]
pub struct InitRuntimeResp {
    pub public_key: String,
    pub attestation: InitRespAttestation,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct InitRespAttestation {
    pub version: i32,
    pub provider: String,
}
impl Resp for InitRuntimeReq {
    type Resp = InitRuntimeResp;
}
#[derive(Encode, Decode)]
pub struct GenesisInfo {
    pub header: Header,
    pub validators: AuthorityList,
    pub proof: Vec<Vec<u8>>,
}

// API: sync_block

#[derive(Serialize, Deserialize, Debug)]
pub struct SyncBlockReq {
    pub blocks_b64: Vec<String>,
    pub set_id: u64
}
#[derive(Serialize, Deserialize, Debug)]
pub struct SyncBlockResp {
    pub synced_to: phala_node_runtime::BlockNumber
}
impl Resp for SyncBlockReq {
    type Resp = SyncBlockResp;
}
#[derive(Encode, Decode, Clone, Debug)]
pub struct BlockWithEvents {
    pub block: phala_node_runtime::SignedBlock,
    pub events: Option<Vec<u8>>,
    pub proof: Option<Vec<Vec<u8>>>,
    pub key: Option<Vec<u8>>,
}

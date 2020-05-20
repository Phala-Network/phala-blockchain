use tokio::time::delay_for;
use std::time::Duration;

use serde::{Serialize, Deserialize, de::DeserializeOwned};

extern crate hyper;
use hyper::Client as HttpClient;
use hyper::{Body, Method, Request};
use bytes::buf::BufExt as _;

use phala_node_runtime;
use sp_rpc::number::NumberOrHex;
use codec::{Encode, Decode};
use sp_runtime::{
    generic::SignedBlock,
    OpaqueExtrinsic
};

use sp_finality_grandpa::{AuthorityList, VersionedAuthorityList, GRANDPA_AUTHORITIES_KEY};
use sp_core::{storage::StorageKey, twox_128};

mod error;
use crate::error::Error;

mod runtimes;
use crate::runtimes::PhalaNodeRuntime;

#[derive(structopt::StructOpt)]
struct Args {
    /// Should init pRuntime?
    #[structopt(short = "n", long = "no-init")]
    no_init: bool,
    /// Should enable Remote Attestation
    #[structopt(short = "r", long = "remote-attestation")]
    ra: bool,
}

#[derive(Encode, Decode)]
struct GenesisInfo {
	header: Header,
	validators: AuthorityList,
	proof: Vec<Vec<u8>>,
}

// type Runtime = phala_node_runtime::Runtime;
type Runtime = PhalaNodeRuntime;
type Header = <Runtime as subxt::system::System>::Header;
type Hash = <Runtime as subxt::system::System>::Hash;
type OpaqueBlock = sp_runtime::generic::Block<Header, OpaqueExtrinsic>;
type OpaqueSignedBlock = SignedBlock<OpaqueBlock>;


fn deopaque_signedblock(opaque_block: OpaqueSignedBlock) -> phala_node_runtime::SignedBlock {
    let raw_block = Encode::encode(&opaque_block);
    phala_node_runtime::SignedBlock::decode(&mut raw_block.as_slice()).expect("Block decode failed")
}

async fn get_block_at(client: &subxt::Client<Runtime>, h: Option<u32>, with_events: bool)
        -> Result<BlockWithEvents, Error> {
    let pos = h.map(|h| subxt::BlockNumber::from(NumberOrHex::Number(h)));
    // let hash = if pos == None {
    //     client.finalized_head().await?
    // } else {
    //     client.block_hash(pos).await?
    //         .ok_or(Error::BlockHashNotFound())?
    // };
    let hash = match pos {
        Some(_) => client.block_hash(pos).await?.ok_or(Error::BlockHashNotFound())?,
        None => client.finalized_head().await?
    };

    println!("get_block_at: Got block {:?} hash {:?}", h, hash);

    let opaque_block = client.block(Some(hash)).await?
                             .ok_or(Error::BlockNotFound())?;

    let block = deopaque_signedblock(opaque_block);

	if with_events {
		return Ok(fetch_events(&client, &block).await?);
	}

	Ok(BlockWithEvents {
		block,
		events: None,
		proof: None,
		key: None,
	})
}

async fn get_storage(client: &subxt::Client<Runtime>, hash: Option<Hash>, storage_key: StorageKey) -> Option<Vec<u8>> {
	client.storage(storage_key, hash).await.expect("Error when getting storage")
}

async fn read_proof(client: &subxt::Client<Runtime>, hash: Option<Hash>, storage_key: StorageKey) -> subxt::ReadProof<Hash> {
	client.read_proof(vec![storage_key], hash).await.expect("Error when reading proof")
}

trait Resp {
    type Resp: DeserializeOwned;
}

#[derive(Serialize, Deserialize, Debug)]
struct SignedResp {
    payload: String,
    status: String,
    signature: String
}

#[derive(Serialize, Deserialize, Debug)]
struct Nonce {
    value: u32,
}

impl Nonce {
    fn new() -> Nonce {
        Nonce { value: rand::random::<u32>() }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct RuntimeReq<T: Serialize> {
    input: T,
    nonce: Nonce,
}
impl<T: Serialize> RuntimeReq<T> {
    fn new(input: T) -> Self {
        Self { input: input, nonce: Nonce::new() }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct GetInfoReq {}
#[derive(Serialize, Deserialize, Debug)]
struct GetInfoResp {
    blocknum: phala_node_runtime::BlockNumber,
    initialized: bool,
    public_key: String,
    ecdh_public_key: String,
}
impl Resp for GetInfoReq {
    type Resp = GetInfoResp;
}

#[derive(Serialize, Deserialize, Debug)]
struct InitRuntimeReq {
    skip_ra: bool,
    bridge_genesis_info_b64: String
}
#[derive(Serialize, Deserialize, Debug)]
struct InitRuntimeResp {
    public_key: String,
    attestation: InitRespAttestation,
}
#[derive(Serialize, Deserialize, Debug)]
struct InitRespAttestation {
    version: i32,
    provider: String,
    // payload: { report, signature, signing_cert }
}
impl Resp for InitRuntimeReq {
    type Resp = InitRuntimeResp;
}

#[derive(Encode, Decode, Clone, Debug)]
struct BlockWithEvents {
	block: phala_node_runtime::SignedBlock,
	events: Option<Vec<u8>>,
	proof: Option<Vec<Vec<u8>>>,
	key: Option<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct SyncBlockReq {
    blocks_b64: Vec<String>
}
#[derive(Serialize, Deserialize, Debug)]
struct SyncBlockResp {
    synced_to: phala_node_runtime::BlockNumber
}
impl Resp for SyncBlockReq {
    type Resp = SyncBlockResp;
}

const PRUNTIME_RPC_BASE: &'static str = "http://127.0.0.1:8000";

async fn req<T>(command: &str, param: &T) -> Result<SignedResp, Error>  where T: Serialize {
    let client = HttpClient::new();
    let endpoint = format!("{}/{}", PRUNTIME_RPC_BASE, command);

    let body_json = serde_json::to_string(param)?;

    let req = Request::builder()
        .method(Method::POST)
        .uri(endpoint)
        .header("content-type", "application/json")
        .body(Body::from(body_json))?;

    let res = client.request(req).await?;

    println!("Response: {}", res.status());

    let body = hyper::body::aggregate(res.into_body()).await?;
    let signed_resp: SignedResp = serde_json::from_reader(body.reader())?;

    // TODO: validate the response from pRuntime

    Ok(signed_resp)
}

async fn req_decode<Req>(command: &str, request: Req) -> Result<Req::Resp, Error>
where Req: Serialize + Resp {
    let payload = RuntimeReq::new(request);
    let resp = req(command, &payload).await?;
    let result: Req::Resp = serde_json::from_str(&resp.payload).unwrap();
    Ok(result)
}

async fn req_sync_block(blocks: &Vec<BlockWithEvents>) -> Result<SyncBlockResp, Error> {
    let blocks_b64 = blocks
        .iter()
        .map(|ref block| {
            let raw_block = Encode::encode(block);
            let b64_block = base64::encode(&raw_block);
            b64_block
        })
        .collect();

    let resp = req_decode("sync_block", SyncBlockReq { blocks_b64 }).await?;
    println!("req_sync_block: {:?}", resp);
    Ok(resp)
}

async fn bridge(args: Args) -> Result<(), Error> {
    // Connect to substrate
    // let client = subxt::ClientBuilder::<Runtime>::new().build().compat().await?;
    let client = subxt::ClientBuilder::<Runtime>::new().build().await?;

    let mut info = req_decode("get_info", GetInfoReq {}).await?;
    if !info.initialized && !args.no_init {
        println!("pRuntime not initialized. Requesting init");
		let block = get_block_at(&client, Some(0), false).await?.block;
		let hash = client.block_hash(Some(subxt::BlockNumber::from(NumberOrHex::Number(0)))).await?;
		let storage_key = StorageKey(GRANDPA_AUTHORITIES_KEY.to_vec());
		let value = get_storage(&client, hash, storage_key.clone()).await.unwrap();

		let proof = read_proof(&client, hash, storage_key).await.proof;
		let mut prf = Vec::new();
		for p in proof {
			prf.push(p.to_vec());
		}

		let v: AuthorityList = VersionedAuthorityList::decode(&mut value.as_slice()).expect("Failed to decode VersionedAuthorityList").into();
		let info = GenesisInfo {
			header: block.block.header,
			validators: v,
			proof: prf,
		};

		let info_b64 = base64::encode(&info.encode());
		req_decode("init_runtime", InitRuntimeReq {
			skip_ra: !args.ra,
			bridge_genesis_info_b64: info_b64,
		}).await?;
    }

    loop {
        println!("pRuntime get_info response: {:?}", info);
        let block_tip = get_block_at(&client, None, false).await?.block;
        // info.blocknum is the next required block
        println!("try to upload blocks. next required: {}, finalized tip: {}",
            info.blocknum, block_tip.block.header.number);

        // check if pRuntime has already reached the chain tip.
        if info.blocknum > block_tip.block.header.number {
            println!("waiting for new blocks");
            delay_for(Duration::from_millis(5000)).await;
            continue;
        }

        // no, then catch up to the chain tip
        let mut blocks = Vec::<BlockWithEvents>::new();
        for h in info.blocknum ..= block_tip.block.header.number {
            let block = get_block_at(&client, Some(h), true).await?;
            blocks.push(block.clone());
        }

        println!("feeding {} blocks (from {} to {}) into pRuntime",
                 blocks.len(), info.blocknum, block_tip.block.header.number);
        let r = req_sync_block(&blocks).await?;
        println!("  ..sync_block: {:?}", r);


        // update the latest pRuntime state
        info = req_decode("get_info", GetInfoReq {}).await?;
    }
}

async fn fetch_events(client: &subxt::Client<Runtime>, block: &phala_node_runtime::SignedBlock) -> Result<BlockWithEvents, Error> {
	let hash = client.block_hash(Some(subxt::BlockNumber::from(block.block.header.number))).await?;
	let key = storage_value_key_vec("System", "Events");
	let storage_key = StorageKey(key.clone());
	let block_with_events = match get_storage(&client, hash, storage_key.clone()).await {
		Some(value) => {
			let proof = read_proof(&client, hash, storage_key).await.proof;
			let mut prf = Vec::new();
			for p in proof {
				prf.push(p.to_vec());
			}

			BlockWithEvents {
				block: block.clone(),
				events: Some(value),
				proof: Some(prf),
				key: Some(key),
			}
		},

		None => BlockWithEvents {
			block: block.clone(),
			events: None,
			proof: None,
			key: None,
		}
	};

	Ok(block_with_events)
}

fn storage_value_key_vec(module: &str, storage_key_name: &str) -> Vec<u8> {
	let mut key = twox_128(module.as_bytes()).to_vec();
	key.extend(&twox_128(storage_key_name.as_bytes()));
	key
}

#[paw::main]
#[tokio::main]
async fn main(args: Args) {
    // async_main(args);
    let r = bridge(args).await;
    println!("bridge() exited with result: {:?}", r);
    // TODO: when got any error, we should wait and retry until it works just like a daemon.
}

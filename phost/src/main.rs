use tokio::time::delay_for;
use std::time::Duration;
use std::fs;

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
use sp_core::{storage::StorageKey, Bytes};

mod utils;
use crate::utils::{storage_value_key_vec};

mod error;
use crate::error::Error;

mod runtimes;
use crate::runtimes::PhalaNodeRuntime;

#[derive(Encode, Decode)]
struct GenesisInfo {
	header: Header,
	validators: AuthorityList,
	proof: Vec<Vec<u8>>,
}

#[derive(structopt::StructOpt)]
struct Args {
    /// Should init pRuntime?
    #[structopt(short = "n", long = "no-init")]
    no_init: bool,
    /// The genesis grandpa info data for bridge init, in base64
    #[structopt(short = "g", long = "genesis", default_value = "")]
    genesis: String,
    /// The genesis grandpa info data for bridge init, in base64
    #[structopt(short = "f", long = "genesis-file",
                default_value = "/tmp/alice/chains/local_testnet/genesis-info.txt")]
    genesis_file: String,
    /// Should enable Remote Attestation
    #[structopt(short = "r", long = "remote-attestation")]
    ra: bool,
}

impl Args {
    fn get_genesis(&self) -> String {
        if !self.genesis.is_empty() {
            self.genesis.clone()
        } else {
            let data = fs::read(&self.genesis_file).expect("Missing genesis file");
            String::from_utf8_lossy(&data).to_string()
        }
    }
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

async fn get_block_at(client: &subxt::Client<Runtime>, h: Option<u32>)
        -> Result<phala_node_runtime::SignedBlock, Error> {
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
    Ok(block)
}

async fn get_storage(client: &subxt::Client<Runtime>, hash: Option<Hash>, storage_key: StorageKey) -> Option<Vec<u8>> {
	client.storage(storage_key, hash).await.unwrap()
}

async fn read_proof(client: &subxt::Client<Runtime>, hash: Option<Hash>, storage_key: StorageKey) -> subxt::ReadProof<Hash> {
	client.read_proof(vec![storage_key], hash).await.unwrap()
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
struct SyncEventsReq {
	events: Vec<u8>,
	proof: Vec<Vec<u8>>,
	root: Vec<u8>,
	key: Vec<u8>,
	block_num: u32,
}
#[derive(Serialize, Deserialize, Debug)]
struct SyncEventsResp {
	synced_to: phala_node_runtime::BlockNumber
}
impl Resp for SyncEventsReq {
	type Resp = SyncBlockResp;
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

#[derive(Serialize, Deserialize, Debug)]
struct SyncBlockReq {
    // base64 encoded raw SignedBlock
    data: String
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

async fn req_sync_block(block: &phala_node_runtime::SignedBlock) -> Result<SyncBlockResp, Error> {
    let raw_block = Encode::encode(block);
    let b64_block = base64::encode(&raw_block);
    let resp = req_decode("sync_block", SyncBlockReq { data: b64_block }).await?;
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
		/*req_decode("init_runtime", InitRuntimeReq {
            skip_ra: !args.ra,
            bridge_genesis_info_b64: args.get_genesis()
        }).await?;*/
		let block = get_block_at(&client, Some(0)).await?;
		let hash = client.block_hash(Some(subxt::BlockNumber::from(NumberOrHex::Number(0)))).await?;
		let storage_key = StorageKey(GRANDPA_AUTHORITIES_KEY.to_vec());
		let value = get_storage(&client, hash, storage_key.clone()).await.unwrap();

		let proof = read_proof(&client, hash, storage_key).await.proof;
		let mut prf = Vec::new();
		for p in proof {
			prf.push(p.to_vec());
		}

		let v: AuthorityList = VersionedAuthorityList::decode(&mut value.as_slice()).unwrap().into();
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
        let block_tip = get_block_at(&client, None).await?;
        // info.blocknum is the next needed block
        println!("try to upload block. next required: {}, finalized tip: {}",
            info.blocknum, block_tip.block.header.number);

        // check if pRuntime has already reached the chain tip.
        if info.blocknum > block_tip.block.header.number {
            println!("waiting for new blocks");
            delay_for(Duration::from_millis(5000)).await;
            continue;
        }

        // no, then catch up to the chain tip
        for h in info.blocknum ..= block_tip.block.header.number {
            let block = get_block_at(&client, Some(h)).await?;
            let r = req_sync_block(&block).await?;
			println!("feeded block {} into pRuntime: {:?}", block.block.header.number, r);

			if h > 0 {
				let _ = sync_events(&client, &block, h).await;
			}
        }

        // update the latest pRuntime state
        info = req_decode("get_info", GetInfoReq {}).await?;
    }
}

async fn sync_events(client: &subxt::Client<Runtime>, block: &phala_node_runtime::SignedBlock, h: u32) -> Result<(), Error> {
	let hash = client.block_hash(Some(subxt::BlockNumber::from(NumberOrHex::Number(h)))).await?;
	let key = storage_value_key_vec("System", "Events");
	let storage_key = StorageKey(key.clone());
	let value = get_storage(&client, hash, storage_key.clone()).await.unwrap();

	if filter_events(&client, value.clone()) {
		let proof = read_proof(&client, hash, storage_key).await.proof;
		let mut prf = Vec::new();
		for p in proof {
			prf.push(p.to_vec());
		}
		req_decode("sync_events", SyncEventsReq {
			events: value,
			proof: prf,
			root: block.block.header.state_root.as_bytes().to_vec(),
			key,
			block_num: h,
		}).await?;
	}

	Ok(())
}

fn filter_events(client: &subxt::Client<Runtime>, events: Vec<u8>) -> bool {
	match client.decoder().decode_events(&mut &events[..]) {
		Ok(raw_events) => {
			for (phase, event) in raw_events {
				match event {
					subxt::RuntimeEvent::Raw(re) => {
						if re.module == "PhalaModule" {
							println!("phalaModule event");
							return true;
						}
					},
					_ => (),
				}
			}
		}
		Err(_err) => (),
	}

	false
}

#[paw::main]
#[tokio::main]
async fn main(args: Args) {
    // async_main(args);
    let r = bridge(args).await;
    println!("bridge() exited with result: {:?}", r);
    // TODO: when got any error, we should wait and retry until it works just like a daemon.
}

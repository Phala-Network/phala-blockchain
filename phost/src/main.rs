// use tokio::io;
// use tokio::prelude::*;
use futures_03::compat::Future01CompatExt;
use tokio::time::delay_for;
use std::time::Duration;

use serde::{Serialize, Deserialize};

extern crate hyper;
use hyper::Client as HttpClient;
use hyper::{Body, Method, Request};
use bytes::buf::BufExt as _;

use pnode_runtime;
use sp_rpc::number::NumberOrHex;
use codec::{Encode, Decode};
use sp_runtime::{
    generic::SignedBlock,
    OpaqueExtrinsic
};

// use sc_finality_grandpa::GrandpaJustification;

mod error;
use crate::error::Error;

type Runtime = pnode_runtime::Runtime;
type Header = <Runtime as subxt::system::System>::Header;
type OpaqueBlock = sp_runtime::generic::Block<Header, OpaqueExtrinsic>;
type OpaqueSignedBlock = SignedBlock<OpaqueBlock>;

fn deopaque_signedblock(opaque_block: OpaqueSignedBlock) -> pnode_runtime::SignedBlock {
    let raw_block = Encode::encode(&opaque_block);
    pnode_runtime::SignedBlock::decode(&mut raw_block.as_slice()).expect("Block decode failed")
}

// fn print_jutification(justification: &Vec<u8>) {
//     let grandpa_j = match GrandpaJustification::<pnode_runtime::Block>::decode(&mut justification.as_slice()) {
//         Ok(j) => j,
//         Err(err) => {
//             println!("Err: {:?}", err);
//             return;
//         }
//     };
//     println!("GrandpaJustification:: <private>");
//     // println!("Justification: {{ round: {}, commit: {:?}, votes_ancestries: {:?} }}",
//     //          grandpa_j.round,
//     //          grandpa_j.commit,
//     //          grandpa_j.votes_ancestries);
// }

async fn get_block_at(client: &subxt::Client<Runtime>, h: Option<u32>)
        -> Result<pnode_runtime::SignedBlock, Error> {
    let pos = match h {
        Some(h) => Some(NumberOrHex::Number(h)),
        None => None
    };
    let hash = match client.block_hash(pos).compat().await? {
        Some(hash) => hash,
        None => { eprintln!("Block hash not found!"); return Err(Error::BlockHashNotFound()) }
    };
    println!("get_block_at: Got block {:?} hash {:?}", h, hash);

    let opaque_block = match client.block(Some(hash)).compat().await? {
        Some(block) => block,
        None => { eprintln!("Block not found"); return Err(Error::BlockNotFound()) },
    };

    // let raw_block = Encode::encode(&opaque_block);
    // println!("raw block: {}", hex::encode(&raw_block));

    let block = deopaque_signedblock(opaque_block);
    // println!("block: {:?}", block);

    Ok(block)
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
struct Null {}

#[derive(Serialize, Deserialize, Debug)]
struct GetInfo {
    input: Null,
    nonce: Nonce
}

impl GetInfo {
    fn new() -> GetInfo {
        GetInfo {
            input: Null {},
            nonce: Nonce::new()
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct GetInfoResp {
    blocknum: pnode_runtime::BlockNumber,
    initialized: bool,
    public_key: String
}

#[derive(Serialize, Deserialize, Debug)]
struct SyncBlockInput {
    // base64 encoded raw SignedBlock
    data: String
}

#[derive(Serialize, Deserialize, Debug)]
struct SyncBlock {
    input: SyncBlockInput,
    nonce: Nonce
}

impl SyncBlock {
    fn new(raw_block: String) -> SyncBlock {
        SyncBlock {
            input: SyncBlockInput { data: raw_block },
            nonce: Nonce::new()
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct SyncBlockResp {
    synced_to: pnode_runtime::BlockNumber
}

const PRUNTIME_RPC_BASE: &'static str = "http://172.20.20.211:8000";

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

async fn req_get_info() -> Result<GetInfoResp, Error> {
    let resp = req("get_info", &GetInfo::new()).await?;
    let result: GetInfoResp = serde_json::from_str(&*(resp.payload)).unwrap();
    Ok(result)
}

async fn req_sync_block(block: &pnode_runtime::SignedBlock) -> Result<SyncBlockResp, Error> {
    let raw_block = Encode::encode(block);
    let resp = req("sync_block", &SyncBlock::new(base64::encode(&raw_block))).await?;
    println!("req_sync_block: {:?}", resp);
    let result: SyncBlockResp = serde_json::from_str(&*(resp.payload)).unwrap();
    Ok(result)
}

async fn bridge() -> Result<(), Error> {
    // Connect to substrate
    let client = subxt::ClientBuilder::<Runtime>::new().build().compat().await?;

    let mut info = req_get_info().await?;
    if !info.initialized {
        // TODO: request RA and register to chain
        println!("pRuntime not initialized. TODO: init it!");
    }

    loop {
        println!("pRuntime get_info response: {:?}", info);
        let block_tip = get_block_at(&client, None).await?;
        // info.blocknum is the next needed block
        println!("try to upload block from {} to {}", info.blocknum, block_tip.block.header.number);

        // check if pRuntime has already reached the chain tip.
        if info.blocknum > block_tip.block.header.number {
            // yes, so wait for new blocks...
            println!("waiting for new blocks");
            delay_for(Duration::from_millis(5000)).await;
            continue;
        }

        // no, then catch up to the chain tip
        for h in info.blocknum ..= block_tip.block.header.number {
            let block = get_block_at(&client, Some(h)).await?;
            let r = req_sync_block(&block).await?;
            println!("feeded block {} into pRuntime: {:?}", block.block.header.number, r);
        }

        // update the latest pRuntime state
        info = req_get_info().await?;
    }
}

async fn async_main() {
    // start the bridge
    let r = bridge().await;
    println!("bridge() exited with result: {:?}", r);
    // TODO: when got any error, we should wait and retry until it works just like a daemon.
}

fn main() {
    // tokio 0.1 compatible construction
    use tokio_compat::runtime;
    let mut rt = runtime::Runtime::new().unwrap();
    rt.block_on_std(async_main());
}
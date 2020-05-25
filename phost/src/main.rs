use tokio::time::delay_for;
use std::time::Duration;

use serde::Serialize;
use hyper::Client as HttpClient;
use hyper::{Body, Method, Request};
use bytes::buf::BufExt as _;

use phala_node_runtime::{self, BlockNumber};
use sp_rpc::number::NumberOrHex;
use codec::{Encode, Decode};

use sp_finality_grandpa::{AuthorityList, VersionedAuthorityList, GRANDPA_AUTHORITIES_KEY, SetId};
use sp_core::{storage::StorageKey, twox_128};

mod error;
mod runtimes;
mod types;

use crate::error::Error;
use crate::types::{
    Runtime, Header, Hash, OpaqueSignedBlock,
    Resp, SignedResp, RuntimeReq,
    GetInfoReq,
    InitRuntimeReq, GenesisInfo,
    SyncBlockReq, SyncBlockResp, BlockWithEvents, AuthoritySet, AuthoritySetChange
};

type XtClient = subxt::Client<Runtime>;

#[derive(structopt::StructOpt)]
struct Args {
    /// Should init pRuntime?
    #[structopt(short = "n", long = "no-init")]
    no_init: bool,
    /// Should enable Remote Attestation
    #[structopt(short = "r", long = "remote-attestation")]
    ra: bool,
}

struct BlockSyncState {
    blocks: Vec<BlockWithEvents>,
    authory_set_state: Option<(BlockNumber, SetId)>
}

const PRUNTIME_RPC_BASE: &'static str = "http://127.0.0.1:8000";

fn deopaque_signedblock(opaque_block: OpaqueSignedBlock) -> phala_node_runtime::SignedBlock {
    let raw_block = Encode::encode(&opaque_block);
    phala_node_runtime::SignedBlock::decode(&mut raw_block.as_slice()).expect("Block decode failed")
}

async fn get_block_at(client: &XtClient, h: Option<u32>, with_events: bool)
-> Result<BlockWithEvents, Error> {
    let pos = h.map(|h| subxt::BlockNumber::from(NumberOrHex::Number(h)));
    let hash = match pos {
        Some(_) => client.block_hash(pos).await?.ok_or(Error::BlockHashNotFound)?,
        None => client.finalized_head().await?
    };

    println!("get_block_at: Got block {:?} hash {:?}", h, hash);

    let opaque_block = client.block(Some(hash)).await?
    .ok_or(Error::BlockNotFound)?;

    let block = deopaque_signedblock(opaque_block);

    if with_events {
        let block_with_events = fetch_events(&client, &block).await?;
        if let Some(ref events) = block_with_events.events {
            println!("          ... with events {} bytes", events.len());
        }
        return Ok(block_with_events)
    }

    Ok(BlockWithEvents {
        block,
        events: None,
        proof: None,
        key: None,
    })
}

async fn get_storage(client: &XtClient, hash: Option<Hash>, storage_key: StorageKey) -> Result<Option<Vec<u8>>, Error> {
    Ok(client.storage(storage_key, hash).await?)
}

async fn read_proof(client: &XtClient, hash: Option<Hash>, storage_key: StorageKey) -> Result<subxt::ReadProof<Hash>, Error> {
    Ok(client.read_proof(vec![storage_key], hash).await?)
}

async fn get_authority_with_proof_at(client: &XtClient, hash: Hash) -> Result<AuthoritySetChange, Error> {
    // Storage
    let storage_key = StorageKey(GRANDPA_AUTHORITIES_KEY.to_vec());
    let value = get_storage(&client, Some(hash), storage_key.clone()).await?
        .expect("No authority key found");
    let authority_set: AuthorityList = VersionedAuthorityList::decode(&mut value.as_slice())
        .expect("Failed to decode VersionedAuthorityList").into();
    // Proof
    let proof = read_proof(&client, Some(hash), storage_key).await?.proof;
    let mut prf = Vec::new();
    for p in proof {
        prf.push(p.to_vec());
    }
    // Set id
    let set_id = client
        .fetch(runtimes::grandpa::CurrentSetIdStore::new(), Some(hash))
        .await
        .map_err(|_| Error::NoSetIdAtBlock)?;
    Ok(AuthoritySetChange {
        authority_set: AuthoritySet {
            authority_set,
            set_id,
        },
        authority_proof: prf,
    })
}

/// Returns the next set_id change by a binary search on the known blocks
///
/// `known_blocks` must have at least one block with block justification, otherwise raise an error
/// `NoJustificationInRange`. If there's no set_id change in the given blocks, it returns None.
async fn bisec_setid_change(
    client: &XtClient,
    last_set: (BlockNumber, SetId),
    known_blocks: &Vec<BlockWithEvents>
) -> Result<Option<BlockNumber>, Error> {
    if known_blocks.is_empty() {
        return Err(Error::SearchSetIdChangeInEmptyRange);
    }
    let (last_block, last_id) = last_set;
    // Run binary search only on blocks with justification
    let headers: Vec<&Header> = known_blocks
        .iter()
        .filter(|b| b.block.block.header.number > last_block && b.block.justification.is_some())
        .map(|b| &b.block.block.header)
        .collect();
    let mut l = 0i64;
    let mut r = (headers.len() as i64) - 1;
    while l <= r {
        let mid = (l + r) / 2;
        let hash = headers[mid as usize].hash();
        let set_id = client
            .fetch(runtimes::grandpa::CurrentSetIdStore::new(), Some(hash))
            .await
            .map_err(|_| Error::NoSetIdAtBlock)?;
        // Left: set_id == last_id, Right: set_id > last_id
        if set_id == last_id {
            l = mid + 1;
        } else {
            r = mid - 1;
        }
    }
    // Return the first occurance of bigger set_id; return (last_id + 1) if not found
    let result = if (l as usize) < headers.len() {
        Some(headers[l as usize].number)
    } else {
        None
    };
    Ok(result)
}

async fn fetch_events(client: &XtClient, block: &phala_node_runtime::SignedBlock) -> Result<BlockWithEvents, Error> {
    let hash = client.block_hash(Some(subxt::BlockNumber::from(block.block.header.number))).await?;
    let key = storage_value_key_vec("System", "Events");
    let storage_key = StorageKey(key.clone());
    let block_with_events = match get_storage(&client, hash, storage_key.clone()).await? {
        Some(value) => {
            let proof = read_proof(&client, hash, storage_key).await?.proof;
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
    let result: Req::Resp = serde_json::from_str(&resp.payload)?;
    Ok(result)
}

async fn req_sync_block(blocks: &Vec<BlockWithEvents>, authority_set_change: Option<&AuthoritySetChange>) -> Result<SyncBlockResp, Error> {
    let blocks_b64 = blocks
        .iter()
        .map(|block| {
            let raw_block = Encode::encode(&block);
            base64::encode(&raw_block)
        })
        .collect();
    let authority_set_change_b64 = authority_set_change.map(|change| {
        let raw_change = Encode::encode(change);
        base64::encode(&raw_change)
    });

    let req = SyncBlockReq { blocks_b64, authority_set_change_b64 };
    let resp = req_decode("sync_block", req).await?;
    Ok(resp)
}

async fn batch_sync_block(
    client: &XtClient,
    sync_state: &mut BlockSyncState
) -> Result<usize, Error> {
    const BATCH_WINDOW: usize = 500;
    let block_buf = &mut sync_state.blocks;
    if block_buf.is_empty() {
        return Ok(0);
    }
    // Current authority set id
    let last_set = if let Some(set) = sync_state.authory_set_state {
        set
    } else {
        let header = &block_buf.first().unwrap().block.block.header;
        let hash = header.hash();
        let number = header.number;
        let set_id = client
            .fetch(runtimes::grandpa::CurrentSetIdStore::new(), Some(hash))
            .await
            .map_err(|_| Error::NoSetIdAtBlock)?;
        let set = (number, set_id);
        sync_state.authory_set_state = Some(set.clone());
        set
    };
    // Find the next set id change
    let set_id_change_at = bisec_setid_change(client, last_set, block_buf).await?;
    // Search
    let mut synced_blocks: usize = 0;
    while !block_buf.is_empty() {
        // Find the longest batch within the window
        let first_block_number = block_buf.first().unwrap().block.block.header.number;
        let end_window = BATCH_WINDOW as isize - 1;
        let end_buffer = block_buf.len() as isize - 1;
        let end_set_id_change = match set_id_change_at {
            Some(change_at) => (change_at - first_block_number) as isize,
            None => block_buf.len() as isize,
        };
        let end = std::cmp::min(end_window, std::cmp::min(end_buffer, end_set_id_change));
        let mut i = end;
        while i >= 0 {
            if block_buf[i as usize].block.justification.is_some() {
                break;
            }
            i -= 1;
        }
        if i < 0 {
            let window_reached = end_window > end_buffer && end_window > end_set_id_change;
            if window_reached {
                println!(
                    "Cannot find justification within BATCH_WINDOW (window: {}, from: {}, to: {})",
                    BATCH_WINDOW, first_block_number,
                    block_buf[end as usize].block.block.header.number,
                );
                return Err(Error::NoJustification);
            } else {
                break;
            }
        }
        // send out the longest batch and remove it from the input buffer
        let block_batch: Vec<BlockWithEvents> = block_buf.drain(..=(i as usize)).collect();

        /* print collected blocks */ {
            for b in block_batch.iter() {
                println!("Block {} :: {} :: {}",
                    b.block.block.header.number,
                    b.block.block.header.hash().to_string(),
                    b.block.block.header.parent_hash.to_string()
                );
            }
        }

        let last_block = &block_batch.last().unwrap();
        let last_block_hash = last_block.block.block.header.hash();
        let last_block_number = last_block.block.block.header.number;

        let mut authrotiy_change: Option<AuthoritySetChange> = None;
        if let Some(change_at) = set_id_change_at {
            if change_at == last_block_number {
                authrotiy_change = Some(
                    get_authority_with_proof_at(&client, last_block_hash).await?);
            }
        }

        println!(
            "sending a batch of {} blocks (last: {}, change: {:?})",
            block_batch.len(), last_block_number,
            authrotiy_change.as_ref().map(|change| &change.authority_set));

        let r = req_sync_block(&block_batch, authrotiy_change.as_ref()).await?;
        println!("  ..sync_block: {:?}", r);
        // Update sync state
        synced_blocks += block_batch.len();
    }
    sync_state.authory_set_state = Some(match set_id_change_at {
        // set_id changed at next block
        Some(change_at) => (change_at + 1, last_set.1 + 1),
        // not changed
        None => (block_buf.last().unwrap().block.block.header.number, last_set.1),
    });
    Ok(synced_blocks)
}

async fn bridge(args: Args) -> Result<(), Error> {
    // Connect to substrate
    let client = subxt::ClientBuilder::<Runtime>::new().build().await?;

    let mut info = req_decode("get_info", GetInfoReq {}).await?;
    if !info.initialized && !args.no_init {
        println!("pRuntime not initialized. Requesting init");
        let block = get_block_at(&client, Some(0), false).await?.block;
        let hash = client.block_hash(Some(subxt::BlockNumber::from(NumberOrHex::Number(0)))).await?
            .expect("No genesis block?");
        let set_proof = get_authority_with_proof_at(&client, hash).await?;
        let info = GenesisInfo {
            header: block.block.header,
            validators: set_proof.authority_set.authority_set,
            proof: set_proof.authority_proof,
        };

        let info_b64 = base64::encode(&info.encode());
        req_decode("init_runtime", InitRuntimeReq {
            skip_ra: !args.ra,
            bridge_genesis_info_b64: info_b64,
        }).await?;
    }

    let mut sync_state = BlockSyncState {
        blocks: Vec::new(),
        authory_set_state: None
    };

    loop {
        // update the latest pRuntime state
        info = req_decode("get_info", GetInfoReq {}).await?;
        println!("pRuntime get_info response: {:?}", info);
        let block_tip = get_block_at(&client, None, false).await?.block;
        // remove the blocks not needed in the buffer. info.blocknum is the next required block
        while let Some(ref b) = sync_state.blocks.first() {
            if b.block.block.header.number >= info.blocknum {
                break;
            }
            sync_state.blocks.remove(0);
        }
        println!("try to upload blocks. next required: {}, finalized tip: {}, buffered {}",
        info.blocknum, block_tip.block.header.number, sync_state.blocks.len());

        // no, then catch up to the chain tip
        let next_block = match sync_state.blocks.last() {
            Some(b) => b.block.block.header.number + 1,
            None => info.blocknum
        };
        for h in next_block ..= block_tip.block.header.number {
            let block = get_block_at(&client, Some(h), true).await?;
            if block.block.justification.is_some() {
                println!("block with justification at: {}", block.block.block.header.number);
            }
            sync_state.blocks.push(block.clone());
        }

        // send the blocks to pRuntime in batch
        let synced_blocks = batch_sync_block(&client, &mut sync_state).await?;

        // check if pRuntime has already reached the chain tip.
        if synced_blocks == 0 {
            println!("waiting for new blocks");
            delay_for(Duration::from_millis(5000)).await;
            continue;
        }
    }
}

#[paw::main]
#[tokio::main]
async fn main(args: Args) {
    // async_main(args);
    let r = bridge(args).await;
    println!("bridge() exited with result: {:?}", r);
    // TODO: when got any error, we should wait and retry until it works just like a daemon.
}

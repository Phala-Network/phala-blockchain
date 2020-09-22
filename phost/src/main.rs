use tokio::time::delay_for;
use std::time::Duration;
use structopt::StructOpt;

use phala_node_runtime::{self, BlockNumber};
use sp_rpc::number::NumberOrHex;
use sc_rpc_api::state::ReadProof;
use codec::{Encode, Decode};
use core::marker::PhantomData;
use sp_finality_grandpa::{AuthorityList, VersionedAuthorityList, GRANDPA_AUTHORITIES_KEY, SetId};
use sp_core::{storage::StorageKey, twox_128, sr25519, crypto::Pair};
use sp_runtime::{
    traits::{
        IdentifyAccount,
        SignedExtension,
        Verify,
    },
};

mod error;
mod pruntime_client;
mod runtimes;
mod types;

use crate::error::Error;
use crate::types::{
    Runtime, Header, Hash, OpaqueSignedBlock,
    GetInfoReq, QueryReq, ReqData, Payload, Query, PendingChainTransfer, TransferData,
    InitRuntimeReq, GenesisInfo,
    SyncHeaderReq, SyncHeaderResp, BlockWithEvents, HeaderWithEvents, AuthoritySet, AuthoritySetChange
};

use subxt::Signer;
use subxt::system::AccountStoreExt;
type XtClient = subxt::Client<Runtime>;
type PrClient = pruntime_client::PRuntimeClient;

#[derive(Debug, StructOpt)]
#[structopt(name = "phost")]
struct Args {
    #[structopt(short = "n", long = "no-init", help = "Should init pRuntime?")]
    no_init: bool,

    #[structopt(long = "no-sync", help = "Don't sync pRuntime. Quit right after initialization.")]
    no_sync: bool,

    #[structopt(long = "no-write-back", help = "Don't write pRuntime egress data back to Substarte.")]
    no_write_back: bool,

    #[structopt(
    short = "r", long = "remote-attestation",
    help = "Should enable Remote Attestation")]
    ra: bool,

    #[structopt(
    default_value = "ws://localhost:9944", long,
    help = "Substrate rpc websocket endpoint")]
    substrate_ws_endpoint: String,

    #[structopt(
    default_value = "http://localhost:8000", long,
    help = "pRuntime http endpoint")]
    pruntime_endpoint: String,

    #[structopt(required = true,
    short = "m", long = "mnemonic",
    help = "SR25519 keypair mnemonic")]
    mnemonic: String,

    #[structopt(default_value = "500", long = "fetch-blocks",
    help = "The batch size to fetch blocks from Substrate.")]
    fetch_blocks: u32,

    #[structopt(default_value = "200", long = "sync-blocks",
    help = "The batch size to sync blocks to pRuntime.")]
    sync_blocks: usize,
}

struct HeaderSyncState {
    headers: Vec<HeaderWithEvents>,
    authory_set_state: Option<(BlockNumber, SetId)>
}

fn deopaque_signedblock(opaque_block: OpaqueSignedBlock) -> phala_node_runtime::SignedBlock {
    let raw_block = Encode::encode(&opaque_block);
    phala_node_runtime::SignedBlock::decode(&mut raw_block.as_slice()).expect("Block decode failed")
}

async fn get_block_at(client: &XtClient, h: Option<u32>, with_events: bool)
                      -> Result<BlockWithEvents, Error> {
    let pos = h.map(|h| subxt::BlockNumber::from(NumberOrHex::Number(h.into())));
    let hash = match pos {
        Some(_) => client.block_hash(pos).await?.ok_or(Error::BlockHashNotFound)?,
        None => client.finalized_head().await?
    };

    println!("get_block_at: Got block {:?} hash {}", h, hash.to_string());

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
    let storage = client.rpc.storage(storage_key, hash).await?;
    Ok(storage.map(|data| (&data.0[..]).to_vec()))
}

async fn read_proof(client: &XtClient, hash: Option<Hash>, storage_key: StorageKey) -> Result<ReadProof<Hash>, Error> {
    client.read_proof(vec![storage_key], hash).await.map_err(Into::into)
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
        .fetch_or_default(runtimes::grandpa::CurrentSetIdStore::new(), Some(hash))
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
/// `known_headers` must have at least one block header with block justification, otherwise raise an error
/// `NoJustificationInRange`. If there's no set_id change in the given blocks, it returns None.
async fn bisec_setid_change(
    client: &XtClient,
    last_set: (BlockNumber, SetId),
    known_headers: &Vec<HeaderWithEvents>
) -> Result<Option<BlockNumber>, Error> {
    if known_headers.is_empty() {
        return Err(Error::SearchSetIdChangeInEmptyRange);
    }
    let (last_block, last_id) = last_set;
    // Run binary search only on blocks with justification
    let headers: Vec<&Header> = known_headers
        .iter()
        .filter(|h| h.header.number > last_block && h.justification.is_some())
        .map(|h| &h.header)
        .collect();
    let mut l = 0i64;
    let mut r = (headers.len() as i64) - 1;
    while l <= r {
        let mid = (l + r) / 2;
        let hash = headers[mid as usize].hash();
        let set_id = client
            .fetch_or_default(runtimes::grandpa::CurrentSetIdStore::new(), Some(hash))
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


async fn req_sync_header(pr: &PrClient, headers: &Vec<HeaderWithEvents>, authority_set_change: Option<&AuthoritySetChange>) -> Result<SyncHeaderResp, Error> {
    let headers_b64 = headers
        .iter()
        .map(|header| {
            let raw_header = Encode::encode(&header);
            base64::encode(&raw_header)
        })
        .collect();
    let authority_set_change_b64 = authority_set_change.map(|change| {
        let raw_change = Encode::encode(change);
        base64::encode(&raw_change)
    });

    let req = SyncHeaderReq { headers_b64, authority_set_change_b64 };
    let resp = pr.req_decode("sync_header", req).await?;
    Ok(resp)
}

async fn batch_sync_header(
    client: &XtClient,
    pr: &PrClient,
    sync_state: &mut HeaderSyncState,
    batch_window: usize
) -> Result<usize, Error> {
    let header_buf = &mut sync_state.headers;
    if header_buf.is_empty() {
        return Ok(0);
    }
    // Current authority set id
    let last_set = if let Some(set) = sync_state.authory_set_state {
        set
    } else {
        let header = &header_buf.first().unwrap().header;
        let hash = header.hash();
        let number = header.number;
        let set_id = client
            .fetch_or_default(runtimes::grandpa::CurrentSetIdStore::new(), Some(hash))
            .await
            .map_err(|_| Error::NoSetIdAtBlock)?;
        let set = (number, set_id);
        sync_state.authory_set_state = Some(set.clone());
        set
    };
    // Find the next set id change
    let set_id_change_at = bisec_setid_change(client, last_set, header_buf).await?;
    let last_number_in_buff = header_buf.last().unwrap().header.number;
    // Search
    let mut synced_headers: usize = 0;
    while !header_buf.is_empty() {
        // Find the longest batch within the window
        let first_block_number = header_buf.first().unwrap().header.number;
        let end_window = batch_window as isize - 1;
        let end_buffer = header_buf.len() as isize - 1;
        let end_set_id_change = match set_id_change_at {
            Some(change_at) => (change_at as isize - first_block_number as isize),
            None => header_buf.len() as isize,
        };
        let end = std::cmp::min(end_window, std::cmp::min(end_buffer, end_set_id_change));
        let mut i = end;
        while i >= 0 {
            if header_buf[i as usize].justification.is_some() {
                break;
            }
            i -= 1;
        }
        if i < 0 {
            let window_reached = end_window < end_buffer && end_window < end_set_id_change;
            if window_reached {
                println!(
                    "Cannot find justification within batch_window (window: {}, from: {}, to: {})",
                    batch_window, first_block_number,
                    header_buf[end as usize].header.number,
                );
                return Err(Error::NoJustification);
            } else {
                break;
            }
        }
        // send out the longest batch and remove it from the input buffer
        let header_batch: Vec<HeaderWithEvents> = header_buf.drain(..=(i as usize)).collect();

        /* print collected blocks */ {
            for h in header_batch.iter() {
                println!("Header {} :: {} :: {}",
                         h.header.number,
                         h.header.hash().to_string(),
                         h.header.parent_hash.to_string()
                );
            }
        }

        let last_header = &header_batch.last().unwrap();
        let last_header_hash = last_header.header.hash();
        let last_header_number = last_header.header.number;

        let mut authrotiy_change: Option<AuthoritySetChange> = None;
        if let Some(change_at) = set_id_change_at {
            if change_at == last_header_number {
                authrotiy_change = Some(
                    get_authority_with_proof_at(&client, last_header_hash).await?);
            }
        }

        println!(
            "sending a batch of {} headers (last: {}, change: {:?})",
            header_batch.len(), last_header_number,
            authrotiy_change.as_ref().map(|change| &change.authority_set));

        let r = req_sync_header(pr, &header_batch, authrotiy_change.as_ref()).await?;
        println!("  ..sync_header: {:?}", r);
        // Update sync state
        synced_headers += header_batch.len();
    }
    sync_state.authory_set_state = Some(match set_id_change_at {
        // set_id changed at next block
        Some(change_at) => (change_at + 1, last_set.1 + 1),
        // not changed
        None => (last_number_in_buff, last_set.1),
    });
    Ok(synced_headers)
}

async fn get_latest_sequence(client: &XtClient) -> Result<u32, Error> {
    let block_tip = get_block_at(&client, None, false).await?.block;
    let hash = block_tip.block.header.hash();
    client.fetch_or_default(runtimes::phala::SequenceStore::new(), Some(hash)).await.or(Ok(0))
}



async fn update_singer_nonce(client: &XtClient, signer: &mut subxt::PairSigner<Runtime, sr25519::Pair>) -> Result<(), Error>
{
    let account_id = signer.account_id();
    let nonce = client.account(account_id, None).await?.nonce;
    signer.set_nonce(nonce);
    Ok(())
}

async fn sync_tx_to_chain(client: &XtClient, pr: &PrClient, sequence: &mut u32, pair: sr25519::Pair) -> Result<(), Error> {
    let query = Query {
        contract_id: 2,
        nonce: 0,
        request: ReqData::PendingChainTransfer {sequence: *sequence},
    };

    let query_value = serde_json::to_value(&query)?;
    let payload = Payload::Plain(query_value.to_string());
    let query_payload = serde_json::to_string(&payload)?;
    println!("query_payload:{}", query_payload);
    let info = pr.req_decode("query", QueryReq { query_payload: query_payload}).await?;
    println!("info:{:}", info.plain);
    let pending_chain_transfer: PendingChainTransfer = serde_json::from_str(&info.plain)?;
    let transfer_data = base64::decode(&pending_chain_transfer.pending_chain_transfer.transfer_queue_b64)
        .map_err(|_|Error::FailedToDecode)?;
    let transfer_queue: Vec<TransferData> = Decode::decode(&mut &transfer_data[..])
        .map_err(|_|Error::FailedToDecode)?;
    if transfer_queue.len() == 0 {
        return Ok(());
    }

    let mut signer = subxt::PairSigner::<Runtime, _>::new(pair);
    update_singer_nonce(&client, &mut signer).await?;

    let mut max_seq = *sequence;
    for transfer_data in &transfer_queue {
        if transfer_data.data.sequence <= *sequence {
            println!("The tx has been submitted.");
            continue;
        }
        if transfer_data.data.sequence > max_seq {
            max_seq = transfer_data.data.sequence;
        }

        let call = runtimes::phala::TransferToChainCall { _runtime: PhantomData, data: transfer_data.encode() };
        let ret = client.submit(call, &signer).await;
        if ret.is_ok() {
            println!("Submit tx successfully");
        } else {
            println!("Failed to submit tx: {:?}", ret);
        }
        signer.increment_nonce();
    }

    *sequence = max_seq;

    Ok(())
}

async fn bridge(args: Args) -> Result<(), Error> {
    // Connect to substrate
    let client = subxt::ClientBuilder::<Runtime>::new()
        .set_url(args.substrate_ws_endpoint.clone())
        .build().await?;
    println!("Connected to substrate at: {}", args.substrate_ws_endpoint.clone());

    // Other initialization
    let pr = PrClient::new(&args.pruntime_endpoint);
    let (pair, _seed) = <sr25519::Pair as Pair>::from_phrase(&args.mnemonic, None).expect("Bad mnemonic");

    // Try to initialize pRuntime and register on-chain
    let mut info = pr.req_decode("get_info", GetInfoReq {}).await?;
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
        let runtime_info = pr.req_decode("init_runtime", InitRuntimeReq {
            skip_ra: !args.ra,
            bridge_genesis_info_b64: info_b64,
        }).await?;

        println!("runtime_info:{:?}", runtime_info);
        if let Some(attestation) = runtime_info.attestation {
            let signature = base64::decode(&attestation.payload.signature).expect("Failed to decode signature");
            let raw_signing_cert = base64::decode_config(&attestation.payload.signing_cert, base64::STANDARD).expect("Failed to decode certificate");
            let call = runtimes::phala::RegisterWorkerCall {
                _runtime: PhantomData,
                encoded_runtime_info: runtime_info.encoded_runtime_info.to_vec(),
                report: attestation.payload.report.as_bytes().to_vec(),
                signature,
                raw_signing_cert,
            };
            let signer = subxt::PairSigner::new(pair.clone());
            let ret = client.watch(call, &signer).await;
            if !ret.is_ok() {
                return Err(Error::FailedToCallRegisterWorker);
            }

            ()
        }
    }

    if args.no_sync {
        println!("Block sync disabled.");
        return Ok(())
    }

    let mut sequence = get_latest_sequence(&client).await?;
    let mut sync_state = HeaderSyncState {
        headers: Vec::new(),
        authory_set_state: None
    };

    loop {
        // update the latest pRuntime state
        info = pr.req_decode("get_info", GetInfoReq {}).await?;
        println!("pRuntime get_info response: {:?}", info);
        let block_tip = get_block_at(&client, None, false).await?.block;
        // remove the blocks not needed in the buffer. info.blocknum is the next required block
        while let Some(ref h) = sync_state.headers.first() {
            if h.header.number >= info.blocknum {
                break;
            }
            sync_state.headers.remove(0);
        }
        println!("try to upload headers. next required: {}, finalized tip: {}, buffered {}",
                 info.blocknum, block_tip.block.header.number, sync_state.headers.len());

        // no, then catch up to the chain tip
        let next_header = match sync_state.headers.last() {
            Some(h) => h.header.number + 1,
            None => info.blocknum
        };
        let batch_end = std::cmp::min(block_tip.block.header.number, next_header + args.fetch_blocks - 1);
        for h in next_header ..= batch_end {
            let block = get_block_at(&client, Some(h), true).await?;
            if block.block.justification.is_some() {
                println!("block with justification at: {}", block.block.block.header.number);
            }
            sync_state.headers.push(HeaderWithEvents {
                header: block.block.block.header.clone(),
                justification: block.block.justification.clone(),
                events: block.events.clone(),
                proof: block.proof.clone(),
                key: block.key.clone()
            });
        }

        if !args.no_write_back {
            sync_tx_to_chain(&client, &pr, &mut sequence, pair.clone()).await?;
        }

        // send the blocks to pRuntime in batch
        let synced_headers = batch_sync_header(&client, &pr, &mut sync_state, args.sync_blocks).await?;

        // check if pRuntime has already reached the chain tip.
        if synced_headers == 0 {
            println!("waiting for new headers");
            delay_for(Duration::from_millis(5000)).await;
            continue;
        }
    }
}

#[tokio::main]
async fn main() {
    let args = Args::from_args();
    let r = bridge(args).await;
    println!("bridge() exited with result: {:?}", r);
    // TODO: when got any error, we should wait and retry until it works just like a daemon.
}

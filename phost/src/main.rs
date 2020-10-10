use tokio::time::delay_for;
use std::time::Duration;
use structopt::StructOpt;

use sp_rpc::number::NumberOrHex;
use sc_rpc_api::state::ReadProof;
use codec::{Encode, Decode};
use core::marker::PhantomData;
use sp_finality_grandpa::{AuthorityList, VersionedAuthorityList, GRANDPA_AUTHORITIES_KEY, SetId};
use sp_core::{storage::StorageKey, twox_128, sr25519, crypto::Pair};

mod error;
mod pruntime_client;
mod runtimes;
mod types;

use crate::error::Error;
use crate::types::{
    Runtime, Header, Hash, BlockNumber, RawEvents, StorageProof, RawStorageKey,
    GetInfoReq, QueryReq, ReqData, Payload, Query, PendingChainTransfer, TransferData,
    InitRuntimeReq, GenesisInfo,
    SyncHeaderReq, SyncHeaderResp, BlockWithEvents, HeaderToSync, AuthoritySet, AuthoritySetChange,
    DispatchBlockReq, DispatchBlockResp, PingReq, /*PingResp,*/ HeartbeatData, /*Heartbeat*/
};

use subxt::Signer;
use subxt::system::AccountStoreExt;
type XtClient = subxt::Client<Runtime>;
type PrClient = pruntime_client::PRuntimeClient;

#[derive(Debug, StructOpt)]
#[structopt(name = "phost")]
struct Args {
    #[structopt(long, help = "Dev mode (equivalent to `--ra=false --use-dev-key --mnenomic='//Alice'`)")]
    dev: bool,

    #[structopt(short = "n", long = "no-init", help = "Should init pRuntime?")]
    no_init: bool,

    #[structopt(long = "no-sync", help = "Don't sync pRuntime. Quit right after initialization.")]
    no_sync: bool,

    #[structopt(long = "no-write-back", help = "Don't write pRuntime egress data back to Substarte.")]
    no_write_back: bool,

    #[structopt(long, help = "Inject dev key (0x1) to pRuntime. Cannot be used with remote attestation enabled.")]
    use_dev_key: bool,

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

    #[structopt(
        required = true,
        default_value = "//Alice",
        short = "m", long = "mnemonic",
        help = "Controller SR25519 private key mnemonic, private key seed, or derive path")]
    mnemonic: String,

    #[structopt(default_value = "1000", long = "fetch-blocks",
    help = "The batch size to fetch blocks from Substrate.")]
    fetch_blocks: u32,

    #[structopt(default_value = "100", long = "sync-blocks",
    help = "The batch size to sync blocks to pRuntime.")]
    sync_blocks: usize,
}

struct BlockSyncState {
    blocks: Vec<BlockWithEvents>,
    authory_set_state: Option<(BlockNumber, SetId)>
}

async fn get_block_at(client: &XtClient, h: Option<u32>, with_events: bool)
                      -> Result<BlockWithEvents, Error> {
    let pos = h.map(|h| subxt::BlockNumber::from(NumberOrHex::Number(h.into())));
    let hash = match pos {
        Some(_) => client.block_hash(pos).await?.ok_or(Error::BlockHashNotFound)?,
        None => client.finalized_head().await?
    };

    println!("get_block_at: Got block {:?} hash {}", h, hash.to_string());

    let block = client.block(Some(hash.clone())).await?
        .ok_or(Error::BlockNotFound)?;

    let (events, proof, key) = if with_events {
        let events_with_proof = fetch_events(&client, &hash).await?;
        if let Some((raw_events, proof, key)) = events_with_proof {
            println!("          ... with events {} bytes", raw_events.len());
            (Some(raw_events), Some(proof), Some(key))
        } else {
            (None, None, None)
        }
    } else {
        (None, None, None)
    };

    Ok(BlockWithEvents { block, events, proof, key })
}

async fn get_storage(client: &XtClient, hash: Option<Hash>, storage_key: StorageKey) -> Result<Option<Vec<u8>>, Error> {
    let storage = client.rpc.storage(&storage_key, hash).await?;
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
        .fetch_or_default(&runtimes::grandpa::CurrentSetIdStore::new(), Some(hash))
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
            .fetch_or_default(&runtimes::grandpa::CurrentSetIdStore::new(), Some(hash))
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

async fn fetch_events(client: &XtClient, hash: &Hash)
-> Result<Option<(RawEvents, StorageProof, RawStorageKey)>, Error> {
    // let hash = client.block_hash(Some(subxt::BlockNumber::from(block_number))).await?;
    let key = storage_value_key_vec("System", "Events");
    let storage_key = StorageKey(key.clone());
    let result = match get_storage(&client, Some(hash.clone()), storage_key.clone()).await? {
        Some(value) => {
            let proof = read_proof(&client, Some(hash.clone()), storage_key).await?
                .proof
                .iter()
                .map(|x| x.to_vec())
                .collect();
            Some((value, proof, key))
        },
        None => None,
    };
    Ok(result)
}

fn storage_value_key_vec(module: &str, storage_key_name: &str) -> Vec<u8> {
    let mut key = twox_128(module.as_bytes()).to_vec();
    key.extend(&twox_128(storage_key_name.as_bytes()));
    key
}

async fn req_sync_header(pr: &PrClient, headers: &Vec<HeaderToSync>, authority_set_change: Option<&AuthoritySetChange>) -> Result<SyncHeaderResp, Error> {
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

async fn req_dispatch_block(pr: &PrClient, blocks: &Vec<BlockWithEvents>) -> Result<DispatchBlockResp, Error> {
    let blocks_b64 = blocks
        .iter()
        .map(|block| {
            let raw_block = Encode::encode(&block);
            base64::encode(&raw_block)
        })
        .collect();

    let req = DispatchBlockReq { blocks_b64 };
    let resp = pr.req_decode("dispatch_block", req).await?;
    Ok(resp)
}

async fn batch_sync_block(
    client: &XtClient,
    pr: &PrClient,
    sync_state: &mut BlockSyncState,
    batch_window: usize
) -> Result<usize, Error> {
    let block_buf = &mut sync_state.blocks;
    if block_buf.is_empty() {
        return Ok(0);
    }

    let mut synced_blocks: usize = 0;
    while !block_buf.is_empty() {
        // Current authority set id
        let last_set = if let Some(set) = sync_state.authory_set_state {
            set
        } else {
            let header = &block_buf.first().unwrap().block.block.header;
            let hash = header.hash();
            let number = header.number;
            let set_id = client
                .fetch_or_default(&runtimes::grandpa::CurrentSetIdStore::new(), Some(hash))
                .await
                .map_err(|_| Error::NoSetIdAtBlock)?;
            let set = (number, set_id);
            sync_state.authory_set_state = Some(set.clone());
            set
        };
        // Find the next set id change
        let set_id_change_at = bisec_setid_change(client, last_set, block_buf).await?;
        let last_number_in_buff = block_buf.last().unwrap().block.block.header.number;
        // Search
        // Find the longest batch within the window
        let first_block_number = block_buf.first().unwrap().block.block.header.number;
        // TODO: fix the potential overflow here
        let end_buffer = block_buf.len() as isize - 1;
        let end_set_id_change = match set_id_change_at {
            Some(change_at) => (change_at as isize - first_block_number as isize),
            None => block_buf.len() as isize,
        };
        let header_end = std::cmp::min(end_buffer, end_set_id_change);
        let mut header_idx = header_end;
        while header_idx >= 0 {
            if block_buf[header_idx as usize].block.justification.is_some() {
                break;
            }
            header_idx -= 1;
        }
        if header_idx < 0 {
            println!(
                "Cannot find justification within window (from: {}, to: {})",
                first_block_number,
                block_buf.last().unwrap().block.block.header.number,
            );
            break;
        }
        // send out the longest batch and remove it from the input buffer
        let mut block_batch: Vec<BlockWithEvents> =  block_buf
            .drain(..=(header_idx as usize))
            .collect();
        let header_batch: Vec<HeaderToSync> = block_batch
            .iter()
            .map(|b| HeaderToSync {
                header: b.block.block.header.clone(),
                justification: b.block.justification.clone(),
            })
            .collect();

        /* print collected headers */ {
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

        let dispatch_window = batch_window - 1;
        while !block_batch.is_empty() {
            // TODO: fix the potential overflow here
            let end_batch = block_batch.len() as isize - 1;
            let batch_end = std::cmp::min(dispatch_window as isize, end_batch);
            if batch_end >= 0 {
                let dispatch_batch: Vec<BlockWithEvents> = block_batch
                    .drain(..=(batch_end as usize))
                    .collect();
                let r = req_dispatch_block(pr, &dispatch_batch).await?;
                println!("  ..dispatch_block: {:?}", r);

                // Update sync state
                synced_blocks += dispatch_batch.len();
            }
        }
        sync_state.authory_set_state = Some(match set_id_change_at {
            // set_id changed at next block
            Some(change_at) => (change_at + 1, last_set.1 + 1),
            // not changed
            None => (last_number_in_buff, last_set.1),
        });
    }
    Ok(synced_blocks)
}

async fn get_balances_ingress_seq(client: &XtClient) -> Result<u64, Error> {
    let latest_block = get_block_at(&client, None, false).await?.block;
    let hash = latest_block.block.header.hash();
    client.fetch_or_default(&runtimes::phala::IngressSequenceStore::new(2), Some(hash)).await.or(Ok(0))
}

async fn update_singer_nonce(client: &XtClient, signer: &mut subxt::PairSigner<Runtime, sr25519::Pair>) -> Result<(), Error>
{
    let account_id = signer.account_id();
    let nonce = client.account(account_id, None).await?.nonce;
    signer.set_nonce(nonce);
    Ok(())
}

async fn sync_tx_to_chain(client: &XtClient, pr: &PrClient, sequence: &mut u64, pair: sr25519::Pair) -> Result<(), Error> {
    let query = Query {
        contract_id: 2,
        nonce: 0,
        request: ReqData::PendingChainTransfer {sequence: *sequence},
    };

    let query_value = serde_json::to_value(&query)?;
    let payload = Payload::Plain(query_value.to_string());
    let query_payload = serde_json::to_string(&payload)?;
    println!("query_payload:{}", query_payload);
    let info = pr.req_decode("query", QueryReq { query_payload }).await?;
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

async fn send_heartbeat_to_chain(client: &XtClient, pr: &PrClient, pair: sr25519::Pair) -> Result<(), Error> {
    let result = pr.req_decode("ping", PingReq {}).await?;
    if result.status != "ok" {
        println!("Ping api returns: {}, skip", result.status);
        return Ok(())
    }

    let data = base64::decode(&mut &result.encoded_data).unwrap();

    let heartbeat_data = HeartbeatData::decode(&mut &data[..]).unwrap();
    println!("Heartbeat at block {}", heartbeat_data.data.block_num);

    let mut signer = subxt::PairSigner::<Runtime, _>::new(pair);
    update_singer_nonce(&client, &mut signer).await?;

    let call = runtimes::phala::HeartbeatCall { _runtime: PhantomData, data };
    let ret = client.submit(call, &signer).await;
    if ret.is_ok() {
        println!("Submit heartbeat successfully");
    } else {
        println!("Failed to submit heartbeat: {:?}", ret);
    }
    signer.increment_nonce();

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
    let pair = <sr25519::Pair as Pair>::from_string(&args.mnemonic, None)
        .expect("Bad privkey derive path");

    // Try to initialize pRuntime and register on-chain
    let mut info = pr.req_decode("get_info", GetInfoReq {}).await?;
    if !info.initialized && !args.no_init {
        println!("pRuntime not initialized. Requesting init");
        let genesis_block = get_block_at(&client, Some(0), false).await?.block;
        let hash = client.block_hash(Some(subxt::BlockNumber::from(NumberOrHex::Number(0)))).await?
            .expect("No genesis block?");
        let set_proof = get_authority_with_proof_at(&client, hash).await?;
        let info = GenesisInfo {
            header: genesis_block.block.header,
            validators: set_proof.authority_set.authority_set,
            proof: set_proof.authority_proof,
        };

        let info_b64 = base64::encode(&info.encode());
        let runtime_info = pr.req_decode("init_runtime", InitRuntimeReq {
            skip_ra: !args.ra,
            bridge_genesis_info_b64: info_b64,
            debug_set_key: match args.use_dev_key {
                true => Some(String::from("0000000000000000000000000000000000000000000000000000000000000001")),
                false => None
            }
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

    let mut sequence = get_balances_ingress_seq(&client).await?;
    let mut sync_state = BlockSyncState {
        blocks: Vec::new(),
        authory_set_state: None
    };

    loop {
        // update the latest pRuntime state
        info = pr.req_decode("get_info", GetInfoReq {}).await?;
        println!("pRuntime get_info response: {:?}", info);
        // for now, we require info.headernum == info.blocknum for simplification
        if info.headernum != info.blocknum {
            return Err(Error::BlockHeaderMismatch);
        }
        let latest_block = get_block_at(&client, None, false).await?.block;
        // remove the headers not needed in the buffer. info.headernum is the next required header
        while let Some(ref b) = sync_state.blocks.first() {
            if b.block.block.header.number >= info.headernum {
                break;
            }
            sync_state.blocks.remove(0);
        }
        println!("try to upload headers. next required: {}, finalized tip: {}, buffered {}",
                 info.headernum, latest_block.block.header.number, sync_state.blocks.len());

        // no, then catch up to the chain tip
        let next_block = match sync_state.blocks.last() {
            Some(b) => b.block.block.header.number + 1,
            None => info.headernum
        };
        let batch_end = std::cmp::min(latest_block.block.header.number, next_block + args.fetch_blocks - 1);
        for b in next_block ..= batch_end {
            let block = get_block_at(&client, Some(b), true).await?;
            if block.block.justification.is_some() {
                println!("block with justification at: {}", block.block.block.header.number);
            }
            sync_state.blocks.push(block.clone());
        }

        if !args.no_write_back {
            sync_tx_to_chain(&client, &pr, &mut sequence, pair.clone()).await?;
        }

        // send the blocks to pRuntime in batch
        let synced_blocks = batch_sync_block(&client, &pr, &mut sync_state, args.sync_blocks).await?;

        // check if pRuntime has already reached the chain tip.
        // println!("synced_blocks: {}, info.initialized: {}, args.no_write_back: {}, next_block: {}", synced_blocks, info.initialized, args.no_write_back, next_block);
        if synced_blocks == 0 {
            // Send heartbeat
            if info.initialized && !args.no_write_back && next_block % 5 == 0 {
                println!("send heartbeat");
                send_heartbeat_to_chain(&client, &pr, pair.clone()).await?;
            }

            println!("waiting for new blocks");
            delay_for(Duration::from_millis(5000)).await;
            continue;
        }
    }
}

fn preprocess_args(args: &mut Args) {
    if args.dev {
        args.ra = false;
        args.use_dev_key = true;
        args.mnemonic = String::from("//Alice");
    }
}

#[tokio::main]
async fn main() {
    let mut args = Args::from_args();
    preprocess_args(&mut args);
    let r = bridge(args).await;
    println!("bridge() exited with result: {:?}", r);
    // TODO: when got any error, we should wait and retry until it works just like a daemon.
}

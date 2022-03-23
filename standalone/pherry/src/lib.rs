use anyhow::{anyhow, Context, Result};
use log::{debug, error, info, warn};
use sp_core::crypto::AccountId32;
use std::cmp;
use std::str::FromStr;
use std::time::Duration;
use structopt::StructOpt;
use tokio::time::sleep;

use codec::Decode;
use phaxt::rpc::ExtraRpcExt as _;
use phaxt::subxt;
use sp_core::{crypto::Pair, sr25519, storage::StorageKey};
use sp_finality_grandpa::{AuthorityList, SetId, VersionedAuthorityList, GRANDPA_AUTHORITIES_KEY};

mod error;
mod msg_sync;
mod notify_client;

pub mod chain_client;
pub mod types;

use crate::error::Error;
use crate::types::{
    BlockNumber, BlockWithChanges, Hash, Header, NotifyReq, NumberOrHex, ParachainApi, PrClient,
    RelaychainApi, SignedBlock, SrSigner,
};
use phactory_api::blocks::{
    self, AuthoritySet, AuthoritySetChange, BlockHeaderWithChanges, HeaderToSync, StorageProof,
};
use phactory_api::prpc::{self, InitRuntimeResponse};
use phactory_api::pruntime_client;

use msg_sync::{Error as MsgSyncError, Receiver, Sender};
use notify_client::NotifyClient;
use phala_types::AttestationReport;

#[derive(Debug, StructOpt)]
#[structopt(name = "pherry")]
struct Args {
    #[structopt(
        long,
        help = "Dev mode (equivalent to `--use-dev-key --mnemonic='//Alice'`)"
    )]
    dev: bool,

    #[structopt(short = "n", long = "no-init", help = "Should init pRuntime?")]
    no_init: bool,

    #[structopt(
        long = "no-sync",
        help = "Don't sync pRuntime. Quit right after initialization."
    )]
    no_sync: bool,

    #[structopt(long, help = "Don't write pRuntime egress data back to Substarte.")]
    no_msg_submit: bool,

    #[structopt(long, help = "Skip registering the worker.")]
    no_register: bool,

    #[structopt(
        long,
        help = "Inject dev key (0x1) to pRuntime. Cannot be used with remote attestation enabled."
    )]
    use_dev_key: bool,

    #[structopt(
        default_value = "",
        long = "inject-key",
        help = "Inject key to pRuntime."
    )]
    inject_key: String,

    #[structopt(
        short = "r",
        long = "remote-attestation",
        help = "Should enable Remote Attestation"
    )]
    ra: bool,

    #[structopt(
        default_value = "ws://localhost:9944",
        long,
        help = "Substrate rpc websocket endpoint"
    )]
    substrate_ws_endpoint: String,

    #[structopt(
        default_value = "ws://localhost:9977",
        long,
        help = "Parachain collator rpc websocket endpoint"
    )]
    collator_ws_endpoint: String,

    #[structopt(
        default_value = "http://localhost:8000",
        long,
        help = "pRuntime http endpoint"
    )]
    pruntime_endpoint: String,

    #[structopt(default_value = "", long, help = "notify endpoint")]
    notify_endpoint: String,

    #[structopt(
        required = true,
        default_value = "//Alice",
        short = "m",
        long = "mnemonic",
        help = "Controller SR25519 private key mnemonic, private key seed, or derive path"
    )]
    mnemonic: String,

    #[structopt(
        default_value = "1000",
        long = "fetch-blocks",
        help = "The batch size to fetch blocks from Substrate."
    )]
    fetch_blocks: u32,

    #[structopt(
        default_value = "100",
        long = "sync-blocks",
        help = "The batch size to sync blocks to pRuntime."
    )]
    sync_blocks: usize,

    #[structopt(
        long = "operator",
        help = "The operator account to set the miner for the worker."
    )]
    operator: Option<String>,

    #[structopt(long = "parachain", help = "Parachain mode")]
    parachain: bool,

    #[structopt(
        long,
        help = "The first parent header to be synced, default to auto-determine"
    )]
    start_header: Option<BlockNumber>,

    #[structopt(long, help = "Don't wait the substrate nodes to sync blocks")]
    no_wait: bool,

    #[structopt(
        default_value = "5000",
        long,
        help = "(Debug only) Set the wait block duration in ms"
    )]
    dev_wait_block_ms: u64,

    #[structopt(
        default_value = "0",
        long,
        help = "The charge transaction payment, unit: balance"
    )]
    tip: u64,
    #[structopt(
        default_value = "4",
        long,
        help = "The transaction longevity, should be a power of two between 4 and 65536. unit: block"
    )]
    longevity: u64,
    #[structopt(
        default_value = "200",
        long,
        help = "Max number of messages to be submitted per-round"
    )]
    max_sync_msgs_per_round: u64,

    #[structopt(long, help = "Auto restart self after an error occurred")]
    auto_restart: bool,

    #[structopt(
        default_value = "10",
        long,
        help = "Max auto restart retries if it continiously failing. Only used with --auto-restart"
    )]
    max_restart_retries: u32,

    #[structopt(long, help = "Restart if number of rpc errors reaches the threshold")]
    restart_on_rpc_error_threshold: Option<u64>,
}

struct RunningFlags {
    worker_registered: bool,
    restart_failure_count: u32,
}

struct BlockSyncState {
    blocks: Vec<BlockWithChanges>,
    /// Tracks the latest known authority set id at a certain block.
    authory_set_state: Option<(BlockNumber, SetId)>,
}

async fn get_header_hash<T: subxt::Config>(
    client: &subxt::Client<T>,
    h: Option<u32>,
) -> Result<T::Hash> {
    let pos = h.map(|h| subxt::BlockNumber::from(NumberOrHex::Number(h.into())));
    let hash = match pos {
        Some(_) => client
            .rpc()
            .block_hash(pos)
            .await?
            .ok_or(Error::BlockHashNotFound)?,
        None => client.rpc().finalized_head().await?,
    };
    Ok(hash)
}

async fn get_block_at<T: subxt::Config>(
    client: &subxt::Client<T>,
    h: Option<u32>,
) -> Result<(SignedBlock<T::Header, T::Extrinsic>, T::Hash)> {
    let hash = get_header_hash(client, h).await?;
    let block = client
        .rpc()
        .block(Some(hash.clone()))
        .await?
        .ok_or(Error::BlockNotFound)?;

    Ok((block, hash))
}

async fn get_block_without_storage_changes(
    api: &RelaychainApi,
    h: Option<u32>,
) -> Result<BlockWithChanges> {
    let (block, hash) = get_block_at(&api.client, h).await?;
    info!("get_block: Got block {:?} hash {}", h, hash.to_string());
    return Ok(BlockWithChanges {
        block,
        storage_changes: Default::default(),
    });
}

pub async fn get_block_with_storage_changes(
    api: &ParachainApi,
    h: Option<u32>,
) -> Result<BlockWithChanges> {
    let (block, hash) = get_block_at(&api.client, h).await?;
    info!(
        "get_block (w/changes): Got block {:?} hash {}",
        h,
        hash.to_string()
    );
    let hash = block.block.header.hash();
    let storage_changes = chain_client::fetch_storage_changes(api, &hash).await?;
    return Ok(BlockWithChanges {
        block,
        storage_changes,
    });
}

async fn get_authority_with_proof_at(
    api: &RelaychainApi,
    hash: Hash,
) -> Result<AuthoritySetChange> {
    // Storage
    let authority_set_key = StorageKey(GRANDPA_AUTHORITIES_KEY.to_vec());
    let id_key = phaxt::storage_key(phaxt::kusama::grandpa::storage::CurrentSetId);
    // Authority set
    let value = api
        .client
        .rpc()
        .storage(&authority_set_key, Some(hash))
        .await?
        .expect("No authority key found")
        .0;
    let list: AuthorityList = VersionedAuthorityList::decode(&mut value.as_slice())
        .expect("Failed to decode VersionedAuthorityList")
        .into();

    // Set id
    let id = api
        .storage()
        .grandpa()
        .current_set_id(Some(hash))
        .await
        .map_err(|_| Error::NoSetIdAtBlock)?;
    // Proof
    let proof =
        chain_client::read_proofs(&api, Some(hash), vec![authority_set_key, id_key]).await?;
    Ok(AuthoritySetChange {
        authority_set: AuthoritySet { list, id },
        authority_proof: proof,
    })
}

async fn get_paraid(api: &ParachainApi, hash: Option<Hash>) -> Result<u32, Error> {
    api.storage()
        .parachain_info()
        .parachain_id(hash)
        .await
        .or(Err(Error::ParachainIdNotFound))
        .map(|id| id.0)
}

/// Returns the next set_id change by a binary search on the known blocks
///
/// `known_blocks` must have at least one block with block justification, otherwise raise an error
/// `NoJustificationInRange`. If there's no set_id change in the given blocks, it returns None.
async fn bisec_setid_change(
    api: &RelaychainApi,
    last_set: (BlockNumber, SetId),
    known_blocks: &Vec<BlockWithChanges>,
) -> Result<Option<BlockNumber>> {
    debug!("bisec_setid_change(last_set: {:?})", last_set);
    if known_blocks.is_empty() {
        return Err(anyhow!(Error::SearchSetIdChangeInEmptyRange));
    }
    let (last_block, last_id) = last_set;
    // Run binary search only on blocks with justification
    let headers: Vec<&Header> = known_blocks
        .iter()
        .filter(|b| b.block.block.header.number > last_block && b.block.justifications.is_some())
        .map(|b| &b.block.block.header)
        .collect();
    let mut l = 0i64;
    let mut r = (headers.len() as i64) - 1;
    while l <= r {
        let mid = (l + r) / 2;
        let hash = headers[mid as usize].hash();
        let set_id = api
            .storage()
            .grandpa()
            .current_set_id(Some(hash))
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
    debug!("bisec_setid_change result: {:?}", result);
    Ok(result)
}

async fn req_sync_header(
    pr: &PrClient,
    headers: Vec<HeaderToSync>,
    authority_set_change: Option<AuthoritySetChange>,
) -> Result<prpc::SyncedTo> {
    let resp = pr
        .sync_header(prpc::HeadersToSync::new(headers, authority_set_change))
        .await?;
    Ok(resp)
}

async fn req_sync_para_header(
    pr: &PrClient,
    headers: blocks::Headers,
    proof: StorageProof,
) -> Result<prpc::SyncedTo> {
    let resp = pr
        .sync_para_header(prpc::ParaHeadersToSync::new(headers, proof))
        .await?;
    Ok(resp)
}

async fn req_dispatch_block(
    pr: &PrClient,
    blocks: Vec<BlockHeaderWithChanges>,
) -> Result<prpc::SyncedTo> {
    let resp = pr.dispatch_blocks(prpc::Blocks::new(blocks)).await?;
    Ok(resp)
}

/// Syncs only the events to pRuntime till `sync_to`
async fn sync_events_only(
    pr: &PrClient,
    sync_state: &mut BlockSyncState,
    sync_to: BlockNumber,
    batch_window: usize,
) -> Result<()> {
    let block_buf = &mut sync_state.blocks;
    // Count the blocks to sync
    let mut n = 0usize;
    for bwe in block_buf.iter() {
        if bwe.block.block.header.number <= sync_to {
            n += 1;
        } else {
            break;
        }
    }
    let blocks: Vec<BlockHeaderWithChanges> = block_buf
        .drain(..n)
        .map(|bwe| BlockHeaderWithChanges {
            block_header: bwe.block.block.header,
            storage_changes: bwe.storage_changes,
        })
        .collect();
    for chunk in blocks.chunks(batch_window) {
        let r = req_dispatch_block(pr, chunk.to_vec()).await?;
        debug!("  ..dispatch_block: {:?}", r);
    }
    Ok(())
}

const GRANDPA_ENGINE_ID: sp_runtime::ConsensusEngineId = *b"FRNK";

async fn batch_sync_block(
    api: &RelaychainApi,
    paraclient: &ParachainApi,
    pr: &PrClient,
    sync_state: &mut BlockSyncState,
    batch_window: usize,
    info: &prpc::PhactoryInfo,
    parachain: bool,
) -> Result<usize> {
    let block_buf = &mut sync_state.blocks;
    if block_buf.is_empty() {
        return Ok(0);
    }
    let mut next_headernum = info.headernum;
    let mut next_blocknum = info.blocknum;
    let mut next_para_headernum = info.para_headernum;

    let mut synced_blocks: usize = 0;
    while !block_buf.is_empty() {
        // Current authority set id
        let last_set = if let Some(set) = sync_state.authory_set_state {
            set
        } else {
            // Construct the authority set from the last block we have synced (the genesis)
            let number = &block_buf.first().unwrap().block.block.header.number - 1;
            let hash = api.client.rpc().block_hash(Some(number.into())).await?;
            let set_id = api
                .storage()
                .grandpa()
                .current_set_id(hash)
                .await
                .map_err(|_| Error::NoSetIdAtBlock)?;
            let set = (number, set_id);
            sync_state.authory_set_state = Some(set.clone());
            set
        };
        // Find the next set id change
        let set_id_change_at = bisec_setid_change(api, last_set, block_buf).await?;
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
        let header_end = cmp::min(end_buffer, end_set_id_change);
        let mut header_idx = header_end;
        while header_idx >= 0 {
            if block_buf[header_idx as usize]
                .block
                .justifications
                .as_ref()
                .map(|v| v.get(GRANDPA_ENGINE_ID))
                .flatten()
                .is_some()
            {
                break;
            }
            header_idx -= 1;
        }
        if header_idx < 0 {
            warn!(
                "Cannot find justification within window (from: {}, to: {})",
                first_block_number,
                block_buf.last().unwrap().block.block.header.number,
            );
            break;
        }
        // send out the longest batch and remove it from the input buffer
        let mut block_batch: Vec<BlockWithChanges> =
            block_buf.drain(..=(header_idx as usize)).collect();
        let header_batch: Vec<HeaderToSync> = block_batch
            .iter()
            .map(|b| HeaderToSync {
                header: b.block.block.header.clone(),
                justification: b
                    .block
                    .justifications
                    .clone()
                    .map(|v| v.into_justification(GRANDPA_ENGINE_ID))
                    .flatten(),
            })
            .collect();

        /* print collected headers */
        {
            for h in header_batch.iter() {
                debug!(
                    "Header {} :: {} :: {}",
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
                authrotiy_change = Some(get_authority_with_proof_at(&api, last_header_hash).await?);
            }
        }

        info!(
            "sending a batch of {} headers (last: {}, change: {:?})",
            header_batch.len(),
            last_header_number,
            authrotiy_change
                .as_ref()
                .map(|change| &change.authority_set)
        );

        let mut header_batch = header_batch;
        header_batch.retain(|h| h.header.number >= next_headernum);
        let r = req_sync_header(pr, header_batch, authrotiy_change).await?;
        info!("  ..sync_header: {:?}", r);
        next_headernum = r.synced_to + 1;

        if parachain {
            let hdr_synced_to =
                sync_parachain_header(pr, api, paraclient, last_header_hash, next_para_headernum)
                    .await?;
            next_para_headernum = hdr_synced_to + 1;
            let mut para_blocks = Vec::new();
            if next_blocknum <= hdr_synced_to {
                for b in next_blocknum..=hdr_synced_to {
                    let block = get_block_with_storage_changes(&paraclient, Some(b)).await?;
                    para_blocks.push(block.clone());
                }
            }
            block_batch = para_blocks;
        }

        let dispatch_window = batch_window - 1;
        while !block_batch.is_empty() {
            // TODO: fix the potential overflow here
            let end_batch = block_batch.len() as isize - 1;
            let batch_end = cmp::min(dispatch_window as isize, end_batch);
            if batch_end >= 0 {
                let dispatch_batch: Vec<BlockHeaderWithChanges> = block_batch
                    .drain(..=(batch_end as usize))
                    .map(|bwe| BlockHeaderWithChanges {
                        block_header: bwe.block.block.header,
                        storage_changes: bwe.storage_changes,
                    })
                    .collect();
                let blocks_count = dispatch_batch.len();
                let r = req_dispatch_block(pr, dispatch_batch).await?;
                debug!("  ..dispatch_block: {:?}", r);
                next_blocknum = r.synced_to + 1;

                // Update sync state
                synced_blocks += blocks_count;
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

async fn sync_parachain_header(
    pr: &PrClient,
    api: &RelaychainApi,
    para_api: &ParachainApi,
    last_header_hash: Hash,
    next_headernum: BlockNumber,
) -> Result<BlockNumber> {
    let para_id = get_paraid(para_api, None).await?;
    let para_head_storage_key = chain_client::paras_heads_key(para_id);

    let raw_header = api
        .client
        .rpc()
        .storage(&para_head_storage_key, Some(last_header_hash))
        .await?;

    let raw_header = if let Some(hdr) = raw_header {
        hdr.0
    } else {
        return Ok(0);
    };

    let para_fin_header_data = chain_client::decode_parachain_heads(raw_header.clone())?;

    let para_fin_header =
        sp_runtime::generic::Header::<BlockNumber, sp_runtime::traits::BlakeTwo256>::decode(
            &mut para_fin_header_data.as_slice(),
        )
        .or(Err(Error::FailedToDecode))?;

    let para_fin_block_number = para_fin_header.number;
    info!(
        "relaychain finalized paraheader number: {}",
        para_fin_block_number
    );

    let header_proof =
        chain_client::read_proof(api, Some(last_header_hash), para_head_storage_key.clone())
            .await?;

    if next_headernum > para_fin_block_number {
        return Ok(next_headernum - 1);
    }
    let mut para_headers = Vec::new();
    for b in next_headernum..=para_fin_block_number {
        let num = subxt::BlockNumber::from(NumberOrHex::Number(b.into()));
        let hash = para_api.client.rpc().block_hash(Some(num)).await?;
        let hash = match hash {
            Some(hash) => hash,
            None => {
                info!("Hash not found for block {}, fetch it next turn", b);
                return Ok(next_headernum - 1);
            }
        };
        let header = para_api
            .client
            .rpc()
            .header(Some(hash))
            .await?
            .ok_or(Error::BlockNotFound)?;
        para_headers.push(header);
    }
    let r = req_sync_para_header(pr, para_headers, header_proof).await?;
    info!("..req_sync_para_header: {:?}", r);
    Ok(r.synced_to)
}

/// Resolves the starting block header for the genesis block.
///
/// It returns the specified value if `start_header` is Some. Otherwise, it returns 0 for
/// standalone blockchain, and resolve to the last relay chain block before the frist parachain
/// parent block. This behavior matches the one on PRB.
async fn resolve_start_header(
    para_api: &ParachainApi,
    is_parachain: bool,
    start_header: Option<BlockNumber>,
) -> Result<BlockNumber> {
    if let Some(start_header) = start_header {
        return Ok(start_header);
    }
    if !is_parachain {
        return Ok(0);
    }
    let h1 = para_api
        .client
        .rpc()
        .block_hash(Some(subxt::BlockNumber::from(NumberOrHex::Number(1))))
        .await?;
    let validation_data = para_api
        .storage()
        .parachain_system()
        .validation_data(h1)
        .await
        .ok()
        .flatten()
        .ok_or(Error::ParachainValidationDataNotFound)?;
    Ok((validation_data.relay_parent_number - 1) as BlockNumber)
}

async fn init_runtime(
    api: &RelaychainApi,
    para_api: &ParachainApi,
    pr: &PrClient,
    skip_ra: bool,
    use_dev_key: bool,
    inject_key: &str,
    operator: Option<AccountId32>,
    is_parachain: bool,
    start_header: BlockNumber,
) -> Result<InitRuntimeResponse> {
    let genesis_block = get_block_at(&api.client, Some(start_header)).await?.0.block;
    let hash = api
        .client
        .rpc()
        .block_hash(Some(subxt::BlockNumber::from(NumberOrHex::Number(
            start_header as _,
        ))))
        .await?
        .expect("No genesis block?");
    let set_proof = get_authority_with_proof_at(api, hash).await?;
    let genesis_state = chain_client::fetch_genesis_storage(para_api).await?;
    let genesis_info = blocks::GenesisBlockInfo {
        block_header: genesis_block.header,
        authority_set: set_proof.authority_set,
        proof: set_proof.authority_proof,
    };
    let mut debug_set_key = None;
    if !inject_key.is_empty() {
        if inject_key.len() != 64 {
            panic!("inject-key must be 32 bytes hex");
        } else {
            info!("Inject key {}", inject_key);
        }
        debug_set_key = Some(hex::decode(inject_key.to_string()).expect("Invalid dev key"));
    } else if use_dev_key {
        info!("Inject key {}", DEV_KEY);
        debug_set_key = Some(hex::decode(DEV_KEY).expect("Invalid dev key"));
    }

    let resp = pr
        .init_runtime(prpc::InitRuntimeRequest::new(
            skip_ra,
            genesis_info,
            debug_set_key,
            genesis_state,
            operator,
            is_parachain,
        ))
        .await?;
    Ok(resp)
}

async fn register_worker(
    para_api: &ParachainApi,
    encoded_runtime_info: Vec<u8>,
    attestation: prpc::Attestation,
    signer: &mut SrSigner,
) -> Result<()> {
    let pruntime_info = Decode::decode(&mut &encoded_runtime_info[..])
        .map_err(|_| anyhow!("Decode pruntime info failed"))?;
    let attestation: phala_types::AttestationReport = Decode::decode(&mut &attestation.payload[..])
        .map_err(|_| anyhow!("Decode attestation payload failed"))?;
    let attestation =
        match attestation {
            phala_types::AttestationReport::SgxIas { ra_report, signature, raw_signing_cert } => {
                phaxt::khala::runtime_types::phala_pallets::utils::attestation::Attestation::SgxIas {
                    ra_report,
                    signature,
                    raw_signing_cert,
                }
            }
            phala_types::AttestationReport::OptOut => {
                phaxt::khala::runtime_types::phala_pallets::utils::attestation::Attestation::OptOut
            }
        };
    chain_client::update_signer_nonce(para_api, signer).await?;
    let ret = para_api
        .tx()
        .phala_registry()
        .register_worker(pruntime_info, attestation.into())
        .sign_and_submit_then_watch(signer)
        .await;
    if ret.is_err() {
        error!("FailedToCallRegisterWorker: {:?}", ret);
        return Err(anyhow!(Error::FailedToCallRegisterWorker));
    }
    signer.increment_nonce();
    Ok(())
}

async fn try_register_worker(
    pr: &PrClient,
    paraclient: &ParachainApi,
    signer: &mut SrSigner,
) -> Result<()> {
    let info = pr.get_runtime_info(()).await?;
    if let Some(attestation) = info.attestation {
        info!("Registering worker...");
        register_worker(&paraclient, info.encoded_runtime_info, attestation, signer).await?;
    }
    Ok(())
}

const DEV_KEY: &str = "0000000000000000000000000000000000000000000000000000000000000001";

async fn wait_until_synced<T: subxt::Config>(client: &subxt::Client<T>) -> Result<()> {
    loop {
        let state = client.extra_rpc().system_sync_state().await?;
        info!(
            "Checking synced: current={} highest={:?}",
            state.current_block, state.highest_block
        );
        if let Some(highest) = state.highest_block {
            if highest - state.current_block <= 8 {
                return Ok(());
            }
        }
        sleep(Duration::from_secs(5)).await;
    }
}

pub async fn subxt_connect<T: subxt::Config>(uri: &str) -> Result<subxt::Client<T>> {
    subxt::ClientBuilder::new()
        .set_url(uri)
        .build()
        .await
        .context("Failed to connect to substrate")
}

async fn bridge(
    args: &Args,
    flags: &mut RunningFlags,
    err_report: Sender<MsgSyncError>,
) -> Result<()> {
    // Connect to substrate

    let api: RelaychainApi = subxt_connect(&args.substrate_ws_endpoint).await?.into();
    info!("Connected to relaychain at: {}", args.substrate_ws_endpoint);

    let para_uri: &str = if args.parachain {
        &args.collator_ws_endpoint
    } else {
        &args.substrate_ws_endpoint
    };
    let para_api: ParachainApi = subxt_connect(para_uri).await?.into();
    info!(
        "Connected to parachain node at: {}",
        args.collator_ws_endpoint
    );

    if !args.no_wait {
        // Don't start our worker until the substrate node is synced
        info!("Waiting for substrate to sync blocks...");
        wait_until_synced(&api.client).await?;
        wait_until_synced(&para_api.client).await?;
        info!("Substrate sync blocks done");
    }

    // Other initialization
    let pr = pruntime_client::new_pruntime_client(args.pruntime_endpoint.clone());
    let pair = <sr25519::Pair as Pair>::from_string(&args.mnemonic, None)
        .expect("Bad privkey derive path");
    let mut signer: SrSigner = subxt::PairSigner::new(pair);
    let nc = NotifyClient::new(&args.notify_endpoint);
    let mut pruntime_initialized = false;
    let mut pruntime_new_init = false;
    let mut initial_sync_finished = false;

    // Try to initialize pRuntime and register on-chain
    let info = pr.get_info(()).await?;
    if !args.no_init {
        if !info.initialized {
            warn!("pRuntime not initialized. Requesting init...");
            let operator = match args.operator.clone() {
                None => None,
                Some(operator) => {
                    let parsed_operator = AccountId32::from_str(&operator)
                        .map_err(|e| anyhow!("Failed to parse operator address: {}", e))?;
                    Some(parsed_operator)
                }
            };
            let start_header =
                resolve_start_header(&para_api, args.parachain, args.start_header).await?;
            info!("Resolved start header at {}", start_header);
            let runtime_info = init_runtime(
                &api,
                &para_api,
                &pr,
                !args.ra,
                args.use_dev_key,
                &args.inject_key,
                operator,
                args.parachain,
                start_header,
            )
            .await?;
            // STATUS: pruntime_initialized = true
            // STATUS: pruntime_new_init = true
            pruntime_initialized = true;
            pruntime_new_init = true;
            nc.notify(&NotifyReq {
                headernum: info.headernum,
                blocknum: info.blocknum,
                pruntime_initialized,
                pruntime_new_init,
                initial_sync_finished,
            })
            .await
            .ok();
            info!("runtime_info: {:?}", runtime_info);
        } else {
            info!("pRuntime already initialized.");
            // STATUS: pruntime_initialized = true
            // STATUS: pruntime_new_init = false
            pruntime_initialized = true;
            pruntime_new_init = false;
            nc.notify(&NotifyReq {
                headernum: info.headernum,
                blocknum: info.blocknum,
                pruntime_initialized,
                pruntime_new_init,
                initial_sync_finished,
            })
            .await
            .ok();
        }
    }

    if args.no_sync {
        if !args.no_register {
            try_register_worker(&pr, &para_api, &mut signer).await?;
            flags.worker_registered = true;
        }
        warn!("Block sync disabled.");
        return Ok(());
    }

    // Don't just sync message if we want to wait for some block
    let mut sync_state = BlockSyncState {
        blocks: Vec::new(),
        authory_set_state: None,
    };

    loop {
        // update the latest pRuntime state
        let info = pr.get_info(()).await?;
        info!("pRuntime get_info response: {:#?}", info);

        // STATUS: header_synced = info.headernum
        // STATUS: block_synced = info.blocknum
        nc.notify(&NotifyReq {
            headernum: info.headernum,
            blocknum: info.blocknum,
            pruntime_initialized,
            pruntime_new_init,
            initial_sync_finished,
        })
        .await
        .ok();

        let latest_block = get_block_at(&api.client, None).await?.0.block;
        // remove the blocks not needed in the buffer. info.blocknum is the next required block
        while let Some(ref b) = sync_state.blocks.first() {
            if b.block.block.header.number >= info.blocknum {
                break;
            }
            sync_state.blocks.remove(0);
        }

        if args.parachain {
            info!(
                "try to sync blocks. next required: (relay_header={}, para_header={}, body={}), relay finalized tip: {}, buffered: {}",
                info.headernum, info.para_headernum, info.blocknum, latest_block.header.number, sync_state.blocks.len());
        } else {
            info!(
                "try to sync blocks. next required: (body={}, header={}), finalized tip: {}, buffered: {}",
                info.blocknum, info.headernum, latest_block.header.number, sync_state.blocks.len());
        }

        // fill the sync buffer to catch up the chain tip
        let next_block = match sync_state.blocks.last() {
            Some(b) => b.block.block.header.number + 1,
            None => {
                if args.parachain {
                    info.headernum
                } else {
                    info.blocknum
                }
            }
        };

        let (batch_end, more_blocks) = {
            let latest = latest_block.header.number;
            let fetch_limit = next_block + args.fetch_blocks - 1;
            if fetch_limit < latest {
                (fetch_limit, true)
            } else {
                (latest, false)
            }
        };

        // TODO.kevin: batch request blocks and changes.
        for b in next_block..=batch_end {
            let block = if args.parachain {
                get_block_without_storage_changes(&api, Some(b)).await?
            } else {
                // api and para_api are connected to the same node in solochain mode
                get_block_with_storage_changes(&para_api, Some(b)).await?
            };

            if block.block.justifications.is_some() {
                debug!(
                    "block with justification at: {}",
                    block.block.block.header.number
                );
            }
            sync_state.blocks.push(block.clone());
        }

        let next_headernum = info.para_headernum;

        // if the header syncs faster than the event, let the events to catch up
        if next_headernum > info.blocknum {
            sync_events_only(&pr, &mut sync_state, next_headernum - 1, args.sync_blocks).await?;
        }

        // send the blocks to pRuntime in batch
        let synced_blocks = batch_sync_block(
            &api,
            &para_api,
            &pr,
            &mut sync_state,
            args.sync_blocks,
            &info,
            args.parachain,
        )
        .await?;

        // check if pRuntime has already reached the chain tip.
        if synced_blocks == 0 && !more_blocks {
            if !initial_sync_finished && !args.no_register {
                if !flags.worker_registered {
                    try_register_worker(&pr, &para_api, &mut signer).await?;
                    flags.worker_registered = true;
                }
            }
            // STATUS: initial_sync_finished = true
            initial_sync_finished = true;
            nc.notify(&NotifyReq {
                headernum: info.headernum,
                blocknum: info.blocknum,
                pruntime_initialized,
                pruntime_new_init,
                initial_sync_finished,
            })
            .await
            .ok();

            // Now we are idle. Let's try to sync the egress messages.
            if !args.no_msg_submit {
                msg_sync::maybe_sync_mq_egress(
                    &para_api,
                    &pr,
                    &mut signer,
                    args.tip,
                    args.longevity,
                    args.max_sync_msgs_per_round,
                    err_report.clone(),
                )
                .await?;
            }
            flags.restart_failure_count = 0;
            info!("Waiting for new blocks");
            sleep(Duration::from_millis(args.dev_wait_block_ms)).await;
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
    if args.longevity > 0 {
        assert!(args.longevity >= 4, "Option --longevity must be 0 or >= 4.");
        assert_eq!(
            args.longevity.count_ones(),
            1,
            "Option --longevity must be power of two."
        );
    }
}

async fn collect_async_errors(
    mut threshold: Option<u64>,
    mut err_receiver: Receiver<MsgSyncError>,
) {
    let threshold_bak = threshold.unwrap_or_default();
    loop {
        match err_receiver.recv().await {
            Some(error) => match error {
                MsgSyncError::BadSignature => {
                    warn!("tx received bad signature, restarting...");
                    return;
                }
                MsgSyncError::OtherRpcError => {
                    if let Some(threshold) = &mut threshold {
                        if *threshold == 0 {
                            warn!("{} tx errors reported, restarting...", threshold_bak);
                            return;
                        }
                        *threshold -= 1;
                    }
                }
            },
            None => {
                warn!("All senders gone, this should never happen!");
                return;
            }
        }
    }
}

pub async fn pherry_main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    let mut args = Args::from_args();
    preprocess_args(&mut args);

    let mut flags = RunningFlags {
        worker_registered: false,
        restart_failure_count: 0,
    };

    loop {
        let (sender, receiver) = msg_sync::create_report_channel();
        let threshold = args.restart_on_rpc_error_threshold;
        tokio::select! {
            res = bridge(&args, &mut flags, sender) => {
                if let Err(err) = res {
                    info!("bridge() exited with error: {:?}", err);
                } else {
                    break;
                }
            }
            () = collect_async_errors(threshold, receiver) => ()
        };
        if !args.auto_restart || flags.restart_failure_count > args.max_restart_retries {
            std::process::exit(if flags.worker_registered { 1 } else { 2 });
        }
        flags.restart_failure_count += 1;
        sleep(Duration::from_secs(2)).await;
        info!("Restarting...");
    }
}

use anyhow::{anyhow, Context, Result};
use log::{debug, error, info, warn};
use phala_node_rpc_ext::MakeInto;
use phala_trie_storage::ser::StorageChanges;
use sp_core::{crypto::AccountId32, H256};
use std::cmp;
use std::convert::TryFrom;
use std::str::FromStr;
use std::time::Duration;
use tokio::time::sleep;

use codec::{Decode, Encode};
use phala_pallets::pallet_registry::Attestation;
use phaxt::{
    dynamic::storage_key,
    rpc::ExtraRpcExt as _,
    sp_core::{crypto::Pair, sr25519},
    subxt::{self, tx::TxPayload},
    RpcClient,
};
use sp_consensus_grandpa::{AuthorityList, SetId, VersionedAuthorityList, GRANDPA_AUTHORITIES_KEY};
use subxt::config::{substrate::Era, Header as _};

mod endpoint;
mod error;
mod msg_sync;
mod notify_client;
mod prefetcher;

pub mod chain_client;
pub mod headers_cache;
pub mod types;

use crate::error::Error;
use crate::types::{
    Block, BlockNumber, ConvertTo, Hash, Header, NotifyReq, NumberOrHex, ParachainApi, PrClient,
    RelaychainApi, SrSigner,
};
use phactory_api::blocks::{
    self, AuthoritySet, AuthoritySetChange, BlockHeader, BlockHeaderWithChanges, HeaderToSync,
    StorageProof,
};
use phactory_api::prpc::{self, InitRuntimeResponse, PhactoryInfo};
use phactory_api::pruntime_client;

use clap::Parser;
use headers_cache::Client as CacheClient;
use msg_sync::{Error as MsgSyncError, Receiver, Sender};
use notify_client::NotifyClient;
use phala_types::AttestationProvider;

pub use phaxt::connect as subxt_connect;

#[derive(Parser, Debug)]
#[clap(
    about = "Sync messages between pruntime and the blockchain.",
    version,
    author
)]
pub struct Args {
    #[arg(
        long,
        help = "Dev mode (equivalent to `--use-dev-key --mnemonic='//Alice'`)"
    )]
    dev: bool,

    #[arg(short = 'n', long = "no-init", help = "Should init pRuntime?")]
    no_init: bool,

    #[arg(
        long = "no-sync",
        help = "Don't sync pRuntime. Quit right after initialization."
    )]
    no_sync: bool,

    #[arg(long, help = "Don't write pRuntime egress data back to Substarte.")]
    no_msg_submit: bool,

    #[arg(long, help = "Skip registering the worker.")]
    no_register: bool,

    #[arg(long, help = "Skip binding the worker endpoint.")]
    no_bind: bool,

    #[arg(
        long,
        help = "Inject dev key (0x1) to pRuntime. Cannot be used with remote attestation enabled."
    )]
    use_dev_key: bool,

    #[arg(
        default_value = "",
        long = "inject-key",
        help = "Inject key to pRuntime."
    )]
    inject_key: String,

    #[arg(
        default_value = "ws://localhost:9944",
        long,
        visible_alias = "substrate-ws-endpoint",
        help = "Substrate (relaychain for --parachain mode) rpc websocket endpoint"
    )]
    relaychain_ws_endpoint: String,

    #[arg(
        default_value = "ws://localhost:9977",
        long,
        alias = "collator-ws-endpoint",
        help = "Parachain rpc websocket endpoint"
    )]
    parachain_ws_endpoint: String,

    #[arg(
        default_value = "http://localhost:8000",
        long,
        help = "pRuntime http endpoint"
    )]
    pruntime_endpoint: String,

    #[arg(
        long,
        help = "pRuntime http endpoint to handover the key. The handover will only happen when the old pRuntime is synced."
    )]
    next_pruntime_endpoint: Option<String>,

    #[arg(default_value = "", long, help = "notify endpoint")]
    notify_endpoint: String,

    #[arg(
        default_value = "//Alice",
        short = 'm',
        long = "mnemonic",
        help = "Controller SR25519 private key mnemonic, private key seed, or derive path"
    )]
    mnemonic: String,

    #[arg(
        default_value = "1000",
        long = "fetch-blocks",
        help = "The batch size to fetch blocks from Substrate."
    )]
    fetch_blocks: u32,

    #[arg(
        default_value = "4",
        long = "sync-blocks",
        help = "The batch size to sync blocks to pRuntime."
    )]
    sync_blocks: BlockNumber,

    #[arg(
        long = "operator",
        help = "The operator account to set the miner for the worker."
    )]
    operator: Option<String>,

    #[arg(long = "parachain", help = "Parachain mode")]
    parachain: bool,

    #[arg(
        long,
        help = "The first parent header to be synced, default to auto-determine"
    )]
    start_header: Option<BlockNumber>,

    #[arg(long, help = "Don't wait the substrate nodes to sync blocks")]
    no_wait: bool,

    #[arg(
        default_value = "5000",
        long,
        help = "(Debug only) Set the wait block duration in ms"
    )]
    dev_wait_block_ms: u64,

    #[arg(
        default_value = "0",
        long,
        help = "The charge transaction payment, unit: balance"
    )]
    tip: u128,

    #[arg(
        default_value = "4",
        long,
        help = "The transaction longevity, should be a power of two between 4 and 65536. unit: block"
    )]
    longevity: u64,

    #[arg(
        default_value = "200",
        long,
        help = "Max number of messages to be submitted per-round"
    )]
    max_sync_msgs_per_round: u64,

    #[arg(long, help = "Auto restart self after an error occurred")]
    auto_restart: bool,

    #[arg(
        default_value = "10",
        long,
        help = "Max auto restart retries if it continiously failing. Only used with --auto-restart"
    )]
    max_restart_retries: u32,

    #[arg(long, help = "Restart if number of rpc errors reaches the threshold")]
    restart_on_rpc_error_threshold: Option<u64>,

    #[arg(long, help = "URI to fetch cached headers from")]
    #[arg(default_value = "")]
    headers_cache_uri: String,

    #[arg(long, help = "Stop when synced to given parachain block")]
    #[arg(default_value_t = BlockNumber::MAX)]
    to_block: BlockNumber,

    #[arg(
        long,
        help = "Disable syncing waiting parachain blocks in the beginning of each round"
    )]
    disable_sync_waiting_paraheaders: bool,

    /// Attestation provider
    #[arg(long, value_enum, default_value_t = RaOption::Ias)]
    attestation_provider: RaOption,

    /// Use IAS RA method, this is compatible with Pherry 1.x
    #[arg(
        short = 'r',
        help = "Use IAS as RA method, enable this will override attestation-provider"
    )]
    use_ias: bool,

    /// Try to load chain state from the latest block that the worker haven't registered at.
    #[arg(long)]
    fast_sync: bool,

    /// The prefered block to load the genesis state from.
    #[arg(long)]
    prefer_genesis_at_block: Option<BlockNumber>,

    /// Load handover proof after blocks synced.
    #[arg(long)]
    load_handover_proof: bool,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
enum RaOption {
    None,
    Ias,
}

impl From<RaOption> for Option<AttestationProvider> {
    fn from(other: RaOption) -> Self {
        match other {
            RaOption::None => None,
            RaOption::Ias => Some(AttestationProvider::Ias),
        }
    }
}

struct RunningFlags {
    worker_registered: bool,
    endpoint_registered: bool,
    restart_failure_count: u32,
}

pub struct BlockSyncState {
    pub blocks: Vec<Block>,
    /// Tracks the latest known authority set id at a certain block.
    pub authory_set_state: Option<(BlockNumber, SetId)>,
}

pub async fn get_header_hash(client: &phaxt::RpcClient, h: Option<u32>) -> Result<Hash> {
    let pos = h.map(|h| subxt::rpc::types::BlockNumber::from(NumberOrHex::Number(h.into())));
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

pub async fn get_block_at(client: &phaxt::RpcClient, h: Option<u32>) -> Result<(Block, Hash)> {
    let hash = get_header_hash(client, h).await?;
    let block = client
        .rpc()
        .block(Some(hash))
        .await?
        .ok_or(Error::BlockNotFound)?;

    Ok((block.convert_to(), hash))
}

pub async fn get_header_at(client: &phaxt::RpcClient, h: Option<u32>) -> Result<(Header, Hash)> {
    let hash = get_header_hash(client, h).await?;
    let header = client
        .rpc()
        .header(Some(hash))
        .await?
        .ok_or(Error::BlockNotFound)?;

    Ok((header.convert_to(), hash))
}

pub async fn get_block_without_storage_changes(
    api: &RelaychainApi,
    h: Option<u32>,
) -> Result<Block> {
    let (block, hash) = get_block_at(api, h).await?;
    info!("get_block: Got block {:?} hash {}", h, hash.to_string());
    Ok(block)
}

pub async fn fetch_storage_changes(
    client: &RpcClient,
    cache: Option<&CacheClient>,
    from: BlockNumber,
    to: BlockNumber,
) -> Result<Vec<BlockHeaderWithChanges>> {
    fetch_storage_changes_with_root_or_not(client, cache, from, to, false).await
}

pub async fn fetch_storage_changes_with_root_or_not(
    client: &RpcClient,
    cache: Option<&CacheClient>,
    from: BlockNumber,
    to: BlockNumber,
    with_root: bool,
) -> Result<Vec<BlockHeaderWithChanges>> {
    log::info!("fetch_storage_changes with_root={with_root}, ({from}-{to})");
    if to < from {
        return Ok(vec![]);
    }
    if let Some(cache) = cache {
        let count = to + 1 - from;
        if let Ok(changes) = cache.get_storage_changes(from, count).await {
            log::info!(
                "Got {} storage changes from cache server ({from}-{to})",
                changes.len()
            );
            return Ok(changes);
        }
    }
    let from_hash = get_header_hash(client, Some(from)).await?;
    let to_hash = get_header_hash(client, Some(to)).await?;

    let changes = if with_root {
        client
            .extra_rpc()
            .get_storage_changes_with_root(&from_hash, &to_hash)
            .await?
            .into_iter()
            .map(|changes| {
                Ok((changes.changes, {
                    let raw: [u8; 32] = TryFrom::try_from(&changes.state_root[..])
                        .or(Err(anyhow!("Invalid state root")))?;
                    H256::from(raw)
                }))
            })
            .collect::<Result<Vec<_>>>()?
    } else {
        client
            .extra_rpc()
            .get_storage_changes(&from_hash, &to_hash)
            .await?
            .into_iter()
            .map(|changes| (changes, Default::default()))
            .collect::<Vec<_>>()
    };
    let storage_changes = changes
        .into_iter()
        .enumerate()
        .map(|(offset, (storage_changes, state_root))| {
            BlockHeaderWithChanges {
                // Headers are synced separately. Only the `number` is used in pRuntime while syncing blocks.
                block_header: BlockHeader {
                    number: from + offset as BlockNumber,
                    parent_hash: Default::default(),
                    state_root,
                    extrinsics_root: Default::default(),
                    digest: Default::default(),
                },
                storage_changes: StorageChanges {
                    main_storage_changes: storage_changes.main_storage_changes.into_(),
                    child_storage_changes: storage_changes.child_storage_changes.into_(),
                },
            }
        })
        .collect();
    Ok(storage_changes)
}

pub async fn batch_sync_storage_changes(
    pr: &PrClient,
    api: &ParachainApi,
    cache: Option<&CacheClient>,
    from: BlockNumber,
    to: BlockNumber,
    batch_size: BlockNumber,
) -> Result<()> {
    info!(
        "batch syncing from {from} to {to} ({} blocks)",
        to as i64 - from as i64 + 1
    );

    let mut fetcher = prefetcher::PrefetchClient::new();

    for from in (from..=to).step_by(batch_size as _) {
        let to = to.min(from.saturating_add(batch_size - 1));
        let storage_changes = fetcher.fetch_storage_changes(api, cache, from, to).await?;
        let r = req_dispatch_block(pr, storage_changes).await?;
        log::debug!("  ..dispatch_block: {:?}", r);
    }
    Ok(())
}

pub async fn get_authority_with_proof_at(
    api: &RelaychainApi,
    hash: Hash,
) -> Result<AuthoritySetChange> {
    // Storage
    let id_key = phaxt::dynamic::storage_key("Grandpa", "CurrentSetId");
    // Authority set
    let value = api
        .rpc()
        .storage(GRANDPA_AUTHORITIES_KEY, Some(hash))
        .await?
        .expect("No authority key found")
        .0;
    let list: AuthorityList = VersionedAuthorityList::decode(&mut value.as_slice())
        .expect("Failed to decode VersionedAuthorityList")
        .into();

    // Set id
    let id = api.current_set_id(Some(hash)).await?;
    // Proof
    let proof =
        chain_client::read_proofs(api, Some(hash), vec![GRANDPA_AUTHORITIES_KEY, &id_key]).await?;
    Ok(AuthoritySetChange {
        authority_set: AuthoritySet { list, id },
        authority_proof: proof,
    })
}

async fn try_load_handover_proof(pr: &PrClient, api: &ParachainApi) -> Result<()> {
    let info = pr.get_info(()).await?;
    if info.safe_mode_level < 2 {
        return Ok(());
    }
    if info.blocknum == 0 {
        return Ok(());
    }
    let current_block = info.blocknum - 1;
    let hash = get_header_hash(api, Some(current_block)).await?;
    let proof = chain_client::read_proofs(
        api,
        Some(hash),
        vec![
            &storage_key("PhalaRegistry", "PRuntimeAddedAt")[..],
            &storage_key("PhalaRegistry", "PRuntimeAllowList")[..],
            &storage_key("Timestamp", "Now")[..],
        ],
    )
    .await
    .context("Failed to get handover proof")?;
    info!("Loading handover proof at {current_block}");
    for p in &proof {
        info!("key=0x{}", hex::encode(sp_core::blake2_256(p)));
    }
    pr.load_storage_proof(prpc::StorageProof { proof }).await?;
    Ok(())
}

/// Returns the next set_id change by a binary search on the known blocks
///
/// `known_blocks` must have at least one block with block justification, otherwise raise an error
/// `NoJustificationInRange`. If there's no set_id change in the given blocks, it returns None.
async fn bisec_setid_change(
    api: &RelaychainApi,
    last_set: (BlockNumber, SetId),
    known_blocks: &Vec<Block>,
) -> Result<Option<BlockNumber>> {
    debug!("bisec_setid_change(last_set: {:?})", last_set);
    if known_blocks.is_empty() {
        return Err(anyhow!(Error::SearchSetIdChangeInEmptyRange));
    }
    let (last_block, last_id) = last_set;
    // Run binary search only on blocks with justification
    let headers: Vec<&Header> = known_blocks
        .iter()
        .filter(|b| b.block.header.number > last_block && b.justifications.is_some())
        .map(|b| &b.block.header)
        .collect();
    let mut l = 0i64;
    let mut r = (headers.len() as i64) - 1;
    while l <= r {
        let mid = (l + r) / 2;
        let hash = headers[mid as usize].hash();
        let set_id = api.current_set_id(Some(hash)).await?;
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

const GRANDPA_ENGINE_ID: sp_runtime::ConsensusEngineId = *b"FRNK";

#[allow(clippy::too_many_arguments)]
async fn batch_sync_block(
    api: &RelaychainApi,
    paraclient: &ParachainApi,
    cache: Option<&CacheClient>,
    pr: &PrClient,
    sync_state: &mut BlockSyncState,
    batch_window: BlockNumber,
    info: &prpc::PhactoryInfo,
    parachain: bool,
) -> Result<BlockNumber> {
    let block_buf = &mut sync_state.blocks;
    if block_buf.is_empty() {
        return Ok(0);
    }
    let mut next_headernum = info.headernum;
    let mut next_blocknum = info.blocknum;
    let mut next_para_headernum = info.para_headernum;

    let mut synced_blocks: BlockNumber = 0;

    let hdr_synced_to = if parachain {
        next_para_headernum
    } else {
        next_headernum
    }
    .saturating_sub(1);
    macro_rules! sync_blocks_to {
        ($to: expr) => {
            if next_blocknum <= $to {
                batch_sync_storage_changes(pr, paraclient, cache, next_blocknum, $to, batch_window)
                    .await?;
                synced_blocks += $to - next_blocknum + 1;
                next_blocknum = $to + 1;
            };
        };
    }
    sync_blocks_to!(hdr_synced_to);

    while !block_buf.is_empty() {
        // Current authority set id
        let last_set = if let Some(set) = sync_state.authory_set_state {
            set
        } else {
            // Construct the authority set from the last block we have synced (the genesis)
            let number = block_buf
                .first()
                .unwrap()
                .block
                .header
                .number
                .saturating_sub(1);
            let hash = api
                .rpc()
                .block_hash(Some(number.into()))
                .await?
                .ok_or_else(|| anyhow!("Failed to get block hash for block number {}", number))?;
            let set_id = api.current_set_id(Some(hash)).await?;
            let set = (number, set_id);
            sync_state.authory_set_state = Some(set);
            set
        };
        // Find the next set id change
        let set_id_change_at = bisec_setid_change(api, last_set, block_buf).await?;
        let last_number_in_buff = block_buf.last().unwrap().block.header.number;
        // Search
        // Find the longest batch within the window
        let first_block_number = block_buf.first().unwrap().block.header.number;
        // TODO: fix the potential overflow here
        let end_buffer = block_buf.len() as isize - 1;
        let end_set_id_change = match set_id_change_at {
            Some(change_at) => change_at as isize - first_block_number as isize,
            None => block_buf.len() as isize,
        };
        let header_end = cmp::min(end_buffer, end_set_id_change);
        let mut header_idx = header_end;
        while header_idx >= 0 {
            if block_buf[header_idx as usize]
                .justifications
                .as_ref()
                .and_then(|v| v.get(GRANDPA_ENGINE_ID))
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
                block_buf.last().unwrap().block.header.number,
            );
            break;
        }
        // send out the longest batch and remove it from the input buffer
        let block_batch: Vec<Block> = block_buf.drain(..=(header_idx as usize)).collect();
        let header_batch: Vec<HeaderToSync> = block_batch
            .iter()
            .map(|b| HeaderToSync {
                header: b.block.header.clone(),
                justification: b
                    .justifications
                    .clone()
                    .and_then(|v| v.into_justification(GRANDPA_ENGINE_ID)),
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
                authrotiy_change = Some(get_authority_with_proof_at(api, last_header_hash).await?);
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
        // Remove justifications for all headers except the last one to reduce data overhead
        // and improve sync performance.
        if let Some((_last, rest_headers)) = header_batch.split_last_mut() {
            for header in rest_headers {
                header.justification = None;
            }
        }
        let r = req_sync_header(pr, header_batch, authrotiy_change).await?;
        info!("  ..sync_header: {:?}", r);
        next_headernum = r.synced_to + 1;

        let hdr_synced_to = if parachain {
            let hdr_synced_to =
                match get_finalized_header(api, paraclient, last_header_hash).await? {
                    Some((fin_header, proof)) => {
                        sync_parachain_header(
                            pr,
                            paraclient,
                            cache,
                            fin_header.number,
                            next_para_headernum,
                            proof,
                        )
                        .await?
                    }
                    None => 0,
                };
            next_para_headernum = hdr_synced_to + 1;
            hdr_synced_to
        } else {
            r.synced_to
        };

        sync_blocks_to!(hdr_synced_to);

        sync_state.authory_set_state = Some(match set_id_change_at {
            // set_id changed at next block
            Some(change_at) => (change_at + 1, last_set.1 + 1),
            // not changed
            None => (last_number_in_buff, last_set.1),
        });
    }
    Ok(synced_blocks)
}

pub async fn get_finalized_header(
    api: &RelaychainApi,
    para_api: &ParachainApi,
    last_header_hash: Hash,
) -> Result<Option<(Header, Vec<Vec<u8>> /*proof*/)>> {
    let para_id = para_api.get_paraid(None).await?;
    get_finalized_header_with_paraid(api, para_id, last_header_hash).await
}

async fn get_finalized_header_with_paraid(
    api: &RelaychainApi,
    para_id: u32,
    last_header_hash: Hash,
) -> Result<Option<(Header, Vec<Vec<u8>> /*proof*/)>> {
    let para_head_storage_key = api.paras_heads_key(para_id)?;

    let raw_header = api
        .rpc()
        .storage(&para_head_storage_key, Some(last_header_hash))
        .await?;

    let raw_header = if let Some(hdr) = raw_header {
        hdr.0
    } else {
        return Ok(None);
    };

    let para_fin_header_data = chain_client::decode_parachain_heads(raw_header.clone())?;

    let para_fin_header =
        sp_runtime::generic::Header::<BlockNumber, sp_runtime::traits::BlakeTwo256>::decode(
            &mut para_fin_header_data.as_slice(),
        )
        .or(Err(Error::FailedToDecode))?;

    let header_proof =
        chain_client::read_proof(api, Some(last_header_hash), &para_head_storage_key).await?;
    Ok(Some((para_fin_header, header_proof)))
}

async fn maybe_sync_waiting_parablocks(
    pr: &PrClient,
    api: &RelaychainApi,
    para_api: &ParachainApi,
    cache_client: &Option<CacheClient>,
    info: &PhactoryInfo,
    batch_window: BlockNumber,
) -> Result<()> {
    info!("Syncing waiting parablocks...");
    let mut fin_header = None;
    if let Some(cache) = &cache_client {
        let mut cached_headers = cache
            .get_headers(info.headernum - 1)
            .await
            .unwrap_or_default();
        if cached_headers.len() > 1 {
            info!("The relaychain header is not staying at a justification checkpoint, skipping to sync paraheaders...");
            return Ok(());
        }
        if cached_headers.len() == 1 {
            fin_header = cached_headers
                .remove(0)
                .para_header
                .map(|h| (h.fin_header_num, h.proof));
        }
    }
    if fin_header.is_none() {
        let last_header_hash = get_header_hash(api, Some(info.headernum - 1)).await?;
        fin_header = get_finalized_header(api, para_api, last_header_hash)
            .await?
            .map(|(h, proof)| (h.number, proof));
    }
    let (fin_header_num, proof) = match fin_header {
        Some(num) => num,
        None => {
            info!("No finalized paraheader found, skipping to sync paraheaders...");
            return Ok(());
        }
    };

    if fin_header_num > info.para_headernum - 1 {
        info!(
            "Syncing waiting para blocks from {} to {}",
            info.para_headernum, fin_header_num
        );
        let hdr_synced_to = sync_parachain_header(
            pr,
            para_api,
            cache_client.as_ref(),
            fin_header_num,
            info.para_headernum,
            proof,
        )
        .await?;
        if info.blocknum <= hdr_synced_to {
            batch_sync_storage_changes(
                pr,
                para_api,
                cache_client.as_ref(),
                info.blocknum,
                hdr_synced_to,
                batch_window,
            )
            .await?;
        }
    } else {
        info!(
            "No more finalized para headers, fin_header_num={}, next_para_headernum={}",
            fin_header_num, info.para_headernum
        );
    }
    Ok(())
}

async fn sync_parachain_header(
    pr: &PrClient,
    para_api: &ParachainApi,
    cache: Option<&CacheClient>,
    para_fin_block_number: BlockNumber,
    next_headernum: BlockNumber,
    header_proof: Vec<Vec<u8>>,
) -> Result<BlockNumber> {
    info!(
        "relaychain finalized paraheader number: {}",
        para_fin_block_number
    );
    if next_headernum > para_fin_block_number {
        return Ok(next_headernum - 1);
    }
    let mut para_headers = if let Some(cache) = cache {
        let count = para_fin_block_number - next_headernum + 1;
        cache
            .get_parachain_headers(next_headernum, count)
            .await
            .unwrap_or_default()
    } else {
        vec![]
    };
    if para_headers.is_empty() {
        info!("parachain headers not found in cache");
        for b in next_headernum..=para_fin_block_number {
            info!("fetching parachain header {}", b);
            let num = subxt::rpc::types::BlockNumber::from(NumberOrHex::Number(b.into()));
            let hash = para_api.rpc().block_hash(Some(num)).await?;
            let hash = match hash {
                Some(hash) => hash,
                None => {
                    info!("Hash not found for block {}, fetch it next turn", b);
                    return Ok(next_headernum - 1);
                }
            };
            let header = para_api
                .rpc()
                .header(Some(hash))
                .await?
                .ok_or(Error::BlockNotFound)?;
            para_headers.push(header.convert_to());
        }
    } else {
        info!("Got {} parachain headers from cache", para_headers.len());
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
    let number = para_api.relay_parent_number().await?;
    Ok((number - 1) as BlockNumber)
}

#[allow(clippy::too_many_arguments)]
async fn init_runtime(
    cache: &Option<CacheClient>,
    api: &RelaychainApi,
    para_api: &ParachainApi,
    pr: &PrClient,
    attestation_provider: Option<AttestationProvider>,
    use_dev_key: bool,
    inject_key: &str,
    operator: Option<AccountId32>,
    is_parachain: bool,
    start_header: BlockNumber,
) -> Result<InitRuntimeResponse> {
    let genesis_info = if let Some(cache) = cache {
        cache.get_genesis(start_header).await.ok()
    } else {
        None
    };
    let genesis_info = match genesis_info {
        Some(genesis_info) => genesis_info,
        None => {
            let genesis_block = get_block_at(api, Some(start_header)).await?.0.block;
            let hash = api
                .rpc()
                .block_hash(Some(subxt::rpc::types::BlockNumber::from(
                    NumberOrHex::Number(start_header as _),
                )))
                .await?
                .expect("No genesis block?");
            let set_proof = get_authority_with_proof_at(api, hash).await?;
            blocks::GenesisBlockInfo {
                block_header: genesis_block.header.clone(),
                authority_set: set_proof.authority_set,
                proof: set_proof.authority_proof,
            }
        }
    };
    let genesis_state = chain_client::fetch_genesis_storage(para_api).await?;
    let mut debug_set_key = None;
    if !inject_key.is_empty() {
        if inject_key.len() != 64 {
            panic!("inject-key must be 32 bytes hex");
        } else {
            info!("Inject key {}", inject_key);
        }
        debug_set_key = Some(hex::decode(inject_key).expect("Invalid dev key"));
    } else if use_dev_key {
        info!("Inject key {}", DEV_KEY);
        debug_set_key = Some(hex::decode(DEV_KEY).expect("Invalid dev key"));
    }

    let resp = pr
        .init_runtime(prpc::InitRuntimeRequest::new(
            attestation_provider.is_none(),
            genesis_info,
            debug_set_key,
            genesis_state,
            operator,
            is_parachain,
            attestation_provider,
        ))
        .await?;
    Ok(resp)
}

async fn register_worker(
    para_api: &ParachainApi,
    encoded_runtime_info: Vec<u8>,
    attestation: prpc::Attestation,
    signer: &mut SrSigner,
    args: &Args,
) -> Result<()> {
    chain_client::update_signer_nonce(para_api, signer).await?;
    let params = mk_params(para_api, args.longevity, args.tip).await?;
    let v2 = attestation.payload.is_none();
    let attestation = match attestation.payload {
        Some(payload) => Attestation::SgxIas {
            ra_report: payload.report.as_bytes().to_vec(),
            signature: payload.signature,
            raw_signing_cert: payload.signing_cert,
        }
        .encode(),
        None => attestation.encoded_report,
    };
    let tx = phaxt::dynamic::tx::register_worker(encoded_runtime_info, attestation, v2);

    let encoded_call_data = tx
        .encode_call_data(&para_api.metadata())
        .expect("should encoded");
    debug!("register_worker call: 0x{}", hex::encode(encoded_call_data));

    let ret = para_api
        .tx()
        .create_signed_with_nonce(&tx, &signer.signer, signer.nonce(), params)?
        .submit_and_watch()
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
    operator: Option<AccountId32>,
    args: &Args,
) -> Result<bool> {
    let info = pr
        .get_runtime_info(prpc::GetRuntimeInfoRequest::new(false, operator))
        .await?;
    if let Some(attestation) = info.attestation {
        info!("Registering worker...");
        register_worker(
            paraclient,
            info.encoded_runtime_info,
            attestation,
            signer,
            args,
        )
        .await?;
        Ok(true)
    } else {
        Ok(false)
    }
}

async fn try_load_chain_state(pr: &PrClient, para_api: &ParachainApi, args: &Args) -> Result<()> {
    let info = pr.get_info(()).await?;
    info!("info: {info:#?}");
    if !info.can_load_chain_state {
        return Ok(());
    }
    let Some(pubkey) = &info.public_key else {
        return Err(anyhow!("No public key found for worker"));
    };
    let Ok(pubkey) = hex::decode(pubkey) else {
        return Err(anyhow!("pRuntime returned an invalid pubkey"));
    };
    let (block_number, state) = chain_client::search_suitable_genesis_for_worker(
        para_api,
        &pubkey,
        args.prefer_genesis_at_block,
    )
    .await
    .context("Failed to search suitable genesis state for worker")?;
    pr.load_chain_state(prpc::ChainState::new(block_number, state))
        .await?;
    Ok(())
}

const DEV_KEY: &str = "0000000000000000000000000000000000000000000000000000000000000001";

async fn wait_until_synced(client: &phaxt::RpcClient) -> Result<()> {
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

async fn bridge(
    args: &Args,
    flags: &mut RunningFlags,
    err_report: Sender<MsgSyncError>,
) -> Result<()> {
    // Connect to substrate

    let api: RelaychainApi = subxt_connect(&args.relaychain_ws_endpoint).await?;
    info!(
        "Connected to relaychain at: {}",
        args.relaychain_ws_endpoint
    );

    let para_uri: &str = if args.parachain {
        &args.parachain_ws_endpoint
    } else {
        &args.relaychain_ws_endpoint
    };
    let para_api: ParachainApi = subxt_connect(para_uri).await?;
    info!("Connected to parachain node at: {para_uri}");

    if !args.no_wait {
        // Don't start our worker until the substrate node is synced
        info!("Waiting for relaychain node to sync blocks...");
        wait_until_synced(&api).await?;
        info!("Waiting for parachain node to sync blocks...");
        wait_until_synced(&para_api).await?;
        info!("Substrate sync blocks done");
    }

    let cache_client = if !args.headers_cache_uri.is_empty() {
        Some(CacheClient::new(&args.headers_cache_uri))
    } else {
        None
    };

    // Other initialization
    let pr = pruntime_client::new_pruntime_client(args.pruntime_endpoint.clone());
    let pair = <sr25519::Pair as Pair>::from_string(&args.mnemonic, None)
        .expect("Bad privkey derive path");
    let mut signer = SrSigner::new(pair);
    let nc = NotifyClient::new(&args.notify_endpoint);
    let mut pruntime_initialized = false;
    let mut pruntime_new_init = false;
    let mut initial_sync_finished = false;

    // Try to initialize pRuntime and register on-chain
    let info = pr.get_info(()).await?;
    let operator = match args.operator.clone() {
        None => None,
        Some(operator) => {
            let parsed_operator = AccountId32::from_str(&operator)
                .map_err(|e| anyhow!("Failed to parse operator address: {}", e))?;
            Some(parsed_operator)
        }
    };
    if !args.no_init {
        if !info.initialized {
            info!("pRuntime not initialized. Requesting init...");
            let start_header =
                resolve_start_header(&para_api, args.parachain, args.start_header).await?;
            info!("Resolved start header at {}", start_header);
            let runtime_info = init_runtime(
                &cache_client,
                &api,
                &para_api,
                &pr,
                args.attestation_provider.into(),
                args.use_dev_key,
                &args.inject_key,
                operator.clone(),
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

        if args.fast_sync {
            try_load_chain_state(&pr, &para_api, args).await?;
        }
    }

    if args.no_sync {
        if !args.no_register {
            let registered =
                try_register_worker(&pr, &para_api, &mut signer, operator, args).await?;
            flags.worker_registered = registered;
        }
        // Try bind worker endpoint
        if !args.no_bind && info.public_key.is_some() {
            // Here the reason we dont directly report errors when `try_update_worker_endpoint` fails is that we want the endpoint can be registered anytime (e.g. days after the pherry initialization)
            match endpoint::try_update_worker_endpoint(&pr, &para_api, &mut signer, args).await {
                Ok(registered) => {
                    flags.endpoint_registered = registered;
                }
                Err(e) => {
                    error!("FailedToCallBindWorkerEndpoint: {:?}", e);
                }
            }
        }
        warn!("Block sync disabled.");
        return Ok(());
    }

    // Don't just sync message if we want to wait for some block
    let mut sync_state = BlockSyncState {
        blocks: Vec::new(),
        authory_set_state: None,
    };

    for round in 0u64.. {
        // update the latest pRuntime state
        let info = pr.get_info(()).await?;
        info!("pRuntime get_info response: {:#?}", info);
        if info.blocknum >= args.to_block {
            info!("Reached target block: {}", args.to_block);
            return Ok(());
        }

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

        let next_headernum = if args.parachain {
            info.para_headernum
        } else {
            info.headernum
        };
        if info.blocknum < next_headernum {
            info!("blocks fall behind");
            batch_sync_storage_changes(
                &pr,
                &para_api,
                cache_client.as_ref(),
                info.blocknum,
                next_headernum - 1,
                args.sync_blocks,
            )
            .await?;
        }
        if args.parachain
            && !args.disable_sync_waiting_paraheaders
            // `round == 0` is for old pruntimes which don't return `waiting_for_paraheaders`
            && (info.waiting_for_paraheaders || round == 0)
        {
            maybe_sync_waiting_parablocks(
                &pr,
                &api,
                &para_api,
                &cache_client,
                &info,
                args.sync_blocks,
            )
            .await?;
        }

        // Sync the relaychain and parachain data from the cache service as much as possible
        if let (true, Some(cache)) = (args.parachain, &cache_client) {
            info!("Fetching headers at {} from cache...", info.headernum);
            let cached_headers = cache.get_headers(info.headernum).await.unwrap_or_default();
            if cached_headers.is_empty() {
                info!("Header cache missing at {}", info.headernum);
            } else {
                info!(
                    "Syncing {} cached headers start from {}",
                    cached_headers.len(),
                    cached_headers[0].header.number
                );
                sync_state.authory_set_state = None;
                sync_state.blocks.clear();
                sync_with_cached_headers(
                    &pr,
                    &para_api,
                    cache_client.as_ref(),
                    info.blocknum,
                    info.para_headernum,
                    cached_headers,
                    args.sync_blocks,
                )
                .await?;
                continue;
            }
        }

        let latest_block = get_block_at(&api, None).await?.0.block;
        // remove the blocks not needed in the buffer. info.blocknum is the next required block
        while let Some(b) = sync_state.blocks.first() {
            if b.block.header.number >= info.headernum {
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
            Some(b) => b.block.header.number + 1,
            None => info.headernum,
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

        for b in next_block..=batch_end {
            let block = get_block_without_storage_changes(&api, Some(b)).await?;

            if block.justifications.is_some() {
                debug!("block with justification at: {}", block.block.header.number);
            }
            sync_state.blocks.push(block);
        }

        // send the blocks to pRuntime in batch
        let synced_blocks = batch_sync_block(
            &api,
            &para_api,
            cache_client.as_ref(),
            &pr,
            &mut sync_state,
            args.sync_blocks,
            &info,
            args.parachain,
        )
        .await?;

        // check if pRuntime has already reached the chain tip.
        if synced_blocks == 0 && !more_blocks {
            if args.load_handover_proof {
                try_load_handover_proof(&pr, &para_api)
                    .await
                    .context("Failed to load handover proof")?;
            }
            if !args.no_register && !flags.worker_registered {
                flags.worker_registered =
                    try_register_worker(&pr, &para_api, &mut signer, operator.clone(), args)
                        .await?;
            }

            if !args.no_bind && !flags.endpoint_registered && info.public_key.is_some() {
                // Here the reason we dont directly report errors when `try_update_worker_endpoint` fails is that we want the endpoint can be registered anytime (e.g. days after the pherry initialization)
                match endpoint::try_update_worker_endpoint(&pr, &para_api, &mut signer, args).await
                {
                    Ok(registered) => {
                        flags.endpoint_registered = registered;
                    }
                    Err(e) => {
                        error!("FailedToCallBindWorkerEndpoint: {:?}", e);
                    }
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

            // Launch key handover if required only when the old pRuntime is up-to-date
            if args.next_pruntime_endpoint.is_some() {
                let next_pr = pruntime_client::new_pruntime_client(
                    args.next_pruntime_endpoint.clone().unwrap(),
                );
                handover_worker_key(&pr, &next_pr).await?;
            }

            sleep(Duration::from_millis(args.dev_wait_block_ms)).await;
            continue;
        }
    }
    Ok(())
}

fn preprocess_args(args: &mut Args) {
    if args.use_ias {
        args.attestation_provider = RaOption::Ias;
    }
    if args.dev {
        args.use_dev_key = true;
        args.mnemonic = String::from("//Alice");
        args.attestation_provider = RaOption::None;
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

pub async fn mk_params(
    api: &ParachainApi,
    longevity: u64,
    tip: u128,
) -> Result<phaxt::ExtrinsicParamsBuilder> {
    let era = if longevity > 0 {
        let header = api
            .rpc()
            .header(<Option<Hash>>::None)
            .await?
            .ok_or_else(|| anyhow!("No header"))?;
        let number = header.number as u64;
        let period = longevity;
        let phase = number % period;
        let era = Era::Mortal(period, phase);
        info!(
            "update era: block={}, period={}, phase={}",
            number, period, phase
        );
        Some((era, header.hash()))
    } else {
        None
    };
    // gua: encoding era crashes when period.trailing_zeros() === 0

    let params = if let Some((era, checkpoint)) = era {
        phaxt::ExtrinsicParamsBuilder::new()
            .tip(tip)
            .era(era, checkpoint)
    } else {
        phaxt::ExtrinsicParamsBuilder::new().tip(tip)
    };

    Ok(params)
}

pub async fn pherry_main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp_micros()
        .parse_default_env()
        .init();

    let mut args = Args::parse();
    preprocess_args(&mut args);

    let mut flags = RunningFlags {
        worker_registered: false,
        endpoint_registered: false,
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

async fn sync_with_cached_headers(
    pr: &PrClient,
    para_api: &ParachainApi,
    cache: Option<&CacheClient>,
    next_blocknum: BlockNumber,
    next_para_headernum: BlockNumber,
    mut headers: Vec<headers_cache::BlockInfo>,
    batch_window: BlockNumber,
) -> Result<()> {
    let last_header = match headers.last_mut() {
        Some(header) => header,
        None => return Ok(()),
    };
    let authority_set_change = last_header.authority_set_change.take();
    let para_header = last_header.para_header.take();

    let headers = headers
        .into_iter()
        .map(|info| blocks::HeaderToSync {
            header: info.header,
            justification: info.justification,
        })
        .collect();
    let r = req_sync_header(pr, headers, authority_set_change).await?;
    info!("  ..sync_header: {:?}", r);

    if let Some(para_header) = para_header {
        let hdr_synced_to = sync_parachain_header(
            pr,
            para_api,
            cache,
            para_header.fin_header_num,
            next_para_headernum,
            para_header.proof,
        )
        .await?;
        if next_blocknum <= hdr_synced_to {
            batch_sync_storage_changes(
                pr,
                para_api,
                cache,
                next_blocknum,
                hdr_synced_to,
                batch_window,
            )
            .await?;
        }
    }
    Ok(())
}

/// This function panics intentionally after the worker key handover finishes
async fn handover_worker_key(server: &PrClient, client: &PrClient) -> Result<()> {
    let challenge = server.handover_create_challenge(()).await?;
    let response = client.handover_accept_challenge(challenge).await?;
    let encrypted_key = server.handover_start(response).await?;
    client.handover_receive(encrypted_key).await?;
    panic!("Worker key handover done, the new pRuntime is ready to go");
}

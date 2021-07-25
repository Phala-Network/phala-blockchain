use anyhow::{anyhow, Result};
use log::{debug, error, info, warn};
use phala_pallets::registry::Attestation;
use sp_core::crypto::AccountId32;
use std::cmp;
use std::str::FromStr;
use std::time::Duration;
use structopt::StructOpt;
use tokio::time::sleep;

use codec::{Decode, Encode};
use core::marker::PhantomData;
use sp_core::{crypto::Pair, sr25519, storage::StorageKey};
use sp_finality_grandpa::{AuthorityList, SetId, VersionedAuthorityList, GRANDPA_AUTHORITIES_KEY};
use sp_rpc::number::NumberOrHex;
use subxt::{system::AccountStoreExt, Signer};

mod chain_client;
mod error;
mod msg_sync;
mod notify_client;
mod pruntime_client;
mod runtimes;
mod types;

use crate::error::Error;
use crate::types::{
    AuthoritySet, AuthoritySetChange, BlockHeaderWithChanges, BlockNumber, BlockWithChanges, Hash,
    Header, HeaderToSync, NotifyReq, OpaqueSignedBlock, Runtime,
};
use enclave_api::blocks::{self, StorageProof};
use enclave_api::prpc::{self, InitRuntimeResponse};

use notify_client::NotifyClient;
type XtClient = subxt::Client<Runtime>;
type PrClient = pruntime_client::PRuntimeClient;
type SrSigner = subxt::PairSigner<Runtime, sr25519::Pair>;

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

    #[structopt(
        long = "no-write-back",
        help = "Don't write pRuntime egress data back to Substarte."
    )]
    no_write_back: bool,

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
        default_value = "0",
        long,
        help = "The first parent header to be synced"
    )]
    start_header: BlockNumber,
}

struct BlockSyncState {
    blocks: Vec<BlockWithChanges>,
    authory_set_state: Option<(BlockNumber, SetId)>,
}

async fn get_header_hash(client: &XtClient, h: Option<u32>) -> Result<Hash> {
    let pos = h.map(|h| subxt::BlockNumber::from(NumberOrHex::Number(h.into())));
    let hash = match pos {
        Some(_) => client
            .block_hash(pos)
            .await?
            .ok_or(Error::BlockHashNotFound)?,
        None => client.finalized_head().await?,
    };
    Ok(hash)
}

async fn get_block_at(client: &XtClient, h: Option<u32>) -> Result<OpaqueSignedBlock> {
    let hash = get_header_hash(client, h).await?;

    info!("get_block_at: Got block {:?} hash {}", h, hash.to_string());

    let block = client
        .block(Some(hash.clone()))
        .await?
        .ok_or(Error::BlockNotFound)?;

    Ok(block)
}

async fn get_block_without_storage_changes(
    client: &XtClient,
    h: Option<u32>,
) -> Result<BlockWithChanges> {
    let block = get_block_at(&client, h).await?;
    return Ok(BlockWithChanges {
        block,
        storage_changes: Default::default(),
    });
}

async fn get_block_with_storage_changes(
    client: &XtClient,
    h: Option<u32>,
) -> Result<BlockWithChanges> {
    let block = get_block_at(&client, h).await?;
    let hash = block.block.header.hash();
    let storage_changes = chain_client::fetch_storage_changes(&client, &hash).await?;
    return Ok(BlockWithChanges {
        block,
        storage_changes,
    });
}

async fn get_authority_with_proof_at(client: &XtClient, hash: Hash) -> Result<AuthoritySetChange> {
    // Storage
    let storage_key = StorageKey(GRANDPA_AUTHORITIES_KEY.to_vec());
    let value = chain_client::get_storage(&client, Some(hash), storage_key.clone())
        .await?
        .expect("No authority key found");
    let authority_set: AuthorityList = VersionedAuthorityList::decode(&mut value.as_slice())
        .expect("Failed to decode VersionedAuthorityList")
        .into();
    // Proof
    let proof = chain_client::read_proof(&client, Some(hash), storage_key).await?;
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
        authority_proof: proof,
    })
}

async fn get_paraid(
    client: &XtClient,
    hash: Option<Hash>,
) -> Result<runtimes::parachain_info::ParachainId, Error> {
    client
        .fetch_or_default(&runtimes::parachain_info::ParachainIdStore::new(), hash)
        .await
        .or(Err(Error::ParachainIdNotFound))
}

/// Returns the next set_id change by a binary search on the known blocks
///
/// `known_blocks` must have at least one block with block justification, otherwise raise an error
/// `NoJustificationInRange`. If there's no set_id change in the given blocks, it returns None.
async fn bisec_setid_change(
    client: &XtClient,
    last_set: (BlockNumber, SetId),
    known_blocks: &Vec<BlockWithChanges>,
) -> Result<Option<BlockNumber>> {
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

async fn req_sync_header(
    pr: &PrClient,
    headers: Vec<HeaderToSync>,
    authority_set_change: Option<AuthoritySetChange>,
) -> Result<prpc::SyncedTo> {
    let resp = pr
        .prpc
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
        .prpc
        .sync_para_header(prpc::ParaHeadersToSync::new(headers, proof))
        .await?;
    Ok(resp)
}

async fn req_dispatch_block(
    pr: &PrClient,
    blocks: Vec<BlockHeaderWithChanges>,
) -> Result<prpc::SyncedTo> {
    let resp = pr.prpc.dispatch_blocks(prpc::Blocks::new(blocks)).await?;
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
    client: &XtClient,
    paraclient: &XtClient,
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
                authrotiy_change =
                    Some(get_authority_with_proof_at(&client, last_header_hash).await?);
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
            let hdr_synced_to = sync_parachain_header(
                pr,
                client,
                paraclient,
                last_header_hash,
                next_para_headernum,
            )
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
    client: &XtClient,
    paraclient: &XtClient,
    last_header_hash: Hash,
    next_headernum: BlockNumber,
) -> Result<BlockNumber> {
    let para_id = get_paraid(paraclient, None).await?;
    let para_head_storage_key = chain_client::paras_heads_key(&para_id);

    let raw_header = chain_client::get_storage(
        &client,
        Some(last_header_hash),
        para_head_storage_key.clone(),
    )
    .await?;

    let raw_header = if let Some(hdr) = raw_header {
        hdr
    } else {
        return Ok(0);
    };

    let para_fin_header_data = chain_client::get_parachain_heads(raw_header.clone())?;

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

    let header_proof = chain_client::read_proof(
        &client,
        Some(last_header_hash),
        para_head_storage_key.clone(),
    )
    .await?;

    if next_headernum > para_fin_block_number {
        return Ok(next_headernum - 1);
    }
    let mut para_headers = Vec::new();
    for b in next_headernum..=para_fin_block_number {
        let num = subxt::BlockNumber::from(NumberOrHex::Number(b.into()));
        let hash = paraclient
            .block_hash(Some(num))
            .await?
            .ok_or(Error::BlockHashNotFound)?;
        let header = paraclient
            .header(Some(hash))
            .await?
            .ok_or(Error::BlockNotFound)?;
        para_headers.push(header);
    }
    let r = req_sync_para_header(pr, para_headers, header_proof).await?;
    info!("..req_sync_para_header: {:?}", r);
    Ok(r.synced_to)
}

/// Updates the nonce from the blockchain (system.account)
async fn update_signer_nonce(client: &XtClient, signer: &mut SrSigner) -> Result<()> {
    // TODO: try to fetch the pending txs from mempool for a more accurate nonce
    let account_id = signer.account_id();
    let nonce = client.account(account_id, None).await?.nonce;
    let local_nonce = signer.nonce();
    signer.set_nonce(cmp::max(nonce, local_nonce.unwrap_or(0)));
    Ok(())
}

async fn init_runtime(
    client: &XtClient,
    paraclient: &XtClient,
    pr: &PrClient,
    skip_ra: bool,
    use_dev_key: bool,
    inject_key: &str,
    operator: Option<AccountId32>,
    is_parachain: bool,
    start_header: BlockNumber,
) -> Result<InitRuntimeResponse> {
    let genesis_block = get_block_at(client, Some(start_header)).await?.block;
    let hash = client
        .block_hash(Some(subxt::BlockNumber::from(NumberOrHex::Number(
            start_header as _,
        ))))
        .await?
        .expect("No genesis block?");
    let set_proof = get_authority_with_proof_at(client, hash).await?;
    let genesis_state = chain_client::fetch_genesis_storage(paraclient).await?;
    let genesis_info = blocks::GenesisBlockInfo {
        block_header: genesis_block.header,
        validator_set: set_proof.authority_set.authority_set,
        validator_set_proof: set_proof.authority_proof,
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
        .prpc
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
    paraclient: &XtClient,
    encoded_runtime_info: Vec<u8>,
    attestation: prpc::Attestation,
    signer: &mut SrSigner,
) -> Result<()> {
    let payload = attestation
        .payload
        .ok_or(anyhow!("Missing attestation payload"))?;
    let signature = base64::decode(&payload.signature).expect("Failed to decode signature");
    let raw_signing_cert = base64::decode_config(&payload.signing_cert, base64::STANDARD)
        .expect("Failed to decode certificate");
    let call = runtimes::phala_registry::RegisterWorkerCall {
        _runtime: PhantomData,
        pruntime_info: Decode::decode(&mut &encoded_runtime_info[..])
            .map_err(|_| anyhow!("Decode pruntime info failed"))?,
        attestation: Attestation::SgxIas {
            ra_report: payload.report.as_bytes().to_vec(),
            signature: signature,
            raw_signing_cert: raw_signing_cert,
        },
    };
    update_signer_nonce(paraclient, signer).await?;
    let ret = paraclient.watch(call, signer).await;
    if ret.is_err() {
        error!("FailedToCallRegisterWorker: {:?}", ret);
        return Err(anyhow!(Error::FailedToCallRegisterWorker));
    }
    signer.increment_nonce();
    Ok(())
}

const DEV_KEY: &str = "0000000000000000000000000000000000000000000000000000000000000001";

async fn bridge(args: Args) -> Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    // Connect to substrate
    let client = subxt::ClientBuilder::<Runtime>::new()
        .set_url(args.substrate_ws_endpoint.clone())
        .skip_type_sizes_check()
        .build()
        .await?;
    info!("Connected to substrate at: {}", args.substrate_ws_endpoint);

    let paraclient = if args.parachain {
        let paraclient = subxt::ClientBuilder::<Runtime>::new()
            .skip_type_sizes_check()
            .set_url(args.collator_ws_endpoint.clone())
            .build()
            .await?;
        info!(
            "Connected to parachain node at: {}",
            args.collator_ws_endpoint
        );
        paraclient
    } else {
        client.clone()
    };

    // Other initialization
    let pr = PrClient::new(&args.pruntime_endpoint);
    let pair = <sr25519::Pair as Pair>::from_string(&args.mnemonic, None)
        .expect("Bad privkey derive path");
    let mut signer: SrSigner = subxt::PairSigner::new(pair);
    let nc = NotifyClient::new(&args.notify_endpoint);
    let mut pruntime_initialized = false;
    let mut pruntime_new_init = false;
    let mut initial_sync_finished = false;
    let mut pending_register_info: Option<(prpc::Attestation, Vec<u8>)> = None;

    // Try to initialize pRuntime and register on-chain
    let info = pr.prpc.get_info(()).await?;
    if !args.no_init {
        let runtime_info;
        if !info.initialized {
            warn!("pRuntime not initialized. Requesting init...");
            let operator = match args.operator {
                None => None,
                Some(operator) => {
                    let parsed_operator = AccountId32::from_str(&operator)
                        .map_err(|e| anyhow!("Failed to parse operator address: {}", e))?;
                    Some(parsed_operator)
                }
            };
            runtime_info = init_runtime(
                &client,
                &paraclient,
                &pr,
                !args.ra,
                args.use_dev_key,
                &args.inject_key,
                operator,
                args.parachain,
                args.start_header,
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
        } else {
            info!("pRuntime already initialized. Fetching runtime info...");
            // TODO.kevin
            // runtime_info = pr
            //     .req_decode("get_runtime_info", GetRuntimeInfoReq {})
            //     .await?;
            runtime_info = todo!("TODO.kevin");

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
        info!("runtime_info: {:?}", runtime_info);
        if let Some(attestation) = runtime_info.attestation {
            pending_register_info = Some((attestation, runtime_info.runtime_info));
        }
    }

    if args.no_sync {
        if let Some((attestation, encoded_runtime_info)) = pending_register_info {
            register_worker(&paraclient, encoded_runtime_info, attestation, &mut signer).await?;
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
        let info = pr.prpc.get_info(()).await?;
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

        let latest_block = get_block_at(&client, None).await?.block;
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
        let batch_end = std::cmp::min(
            latest_block.header.number,
            next_block + args.fetch_blocks - 1,
        );

        // TODO.kevin: batch request blocks and changes.
        for b in next_block..=batch_end {
            let block = if args.parachain {
                get_block_without_storage_changes(&client, Some(b)).await?
            } else {
                get_block_with_storage_changes(&client, Some(b)).await?
            };
            if block.block.justifications.is_some() {
                debug!(
                    "block with justification at: {}",
                    block.block.block.header.number
                );
            }
            sync_state.blocks.push(block.clone());
        }

        let next_headernum = if args.parachain {
            info.para_headernum
        } else {
            info.headernum
        };

        // if the header syncs faster than the event, let the events to catch up
        if next_headernum > info.blocknum {
            sync_events_only(&pr, &mut sync_state, next_headernum - 1, args.sync_blocks).await?;
        }

        // send the blocks to pRuntime in batch
        let synced_blocks = batch_sync_block(
            &client,
            &paraclient,
            &pr,
            &mut sync_state,
            args.sync_blocks,
            &info,
            args.parachain,
        )
        .await?;

        // check if pRuntime has already reached the chain tip.
        if synced_blocks == 0 {
            if let Some((attestation, encoded_runtime_info)) = pending_register_info.take() {
                info!("Registering worker");
                register_worker(&paraclient, encoded_runtime_info, attestation, &mut signer)
                    .await?;
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
            if !args.no_write_back {
                let mut msg_sync = msg_sync::MsgSync::new(&paraclient, &pr, &mut signer);
                msg_sync.maybe_sync_mq_egress().await?;
            }
        }
        if synced_blocks == 0 {
            info!("Waiting for new blocks");
            sleep(Duration::from_millis(5000)).await;
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
    info!("bridge() exited with result: {:?}", r);
    // TODO: when got any error, we should wait and retry until it works just like a daemon.
}

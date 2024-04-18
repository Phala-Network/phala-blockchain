use anyhow::{anyhow, Result};
use rayon::iter::IntoParallelIterator;
use sp_consensus_grandpa::AuthorityList;
use core::time::Duration;
use futures::StreamExt;
use log::{debug, error, info, trace, warn};
use phaxt::ChainApi;
use sp_core::sr25519::Public as Sr25519Public;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::sleep;

use crate::bus::Bus;
use crate::datasource::DataSourceManager;
use crate::headers_db::*;
use crate::processor::{PRuntimeRequest, ProcessorEvent};
use crate::pool_operator::DB;
use crate::{use_parachain_api, use_relaychain_api};

use phactory_api::prpc::{Blocks, ChainState, CombinedHeadersToSync, HeadersToSync, ParaHeadersToSync};
use pherry::headers_cache::Client as CacheClient;

pub struct ChaintipInfo {
    pub relaychain: u32,
    pub parachain: u32,
}

#[derive(Clone, Debug, Default)]
pub struct SyncRequest {
    pub headers: Option<HeadersToSync>,
    pub para_headers: Option<ParaHeadersToSync>,
    pub combined_headers: Option<CombinedHeadersToSync>,
    pub blocks: Option<Blocks>,
    pub manifest: SyncRequestManifest,
}

#[derive(Clone, Debug, Default)]
pub struct SyncRequestManifest {
    pub headers: Option<(u32, u32)>,
    pub para_headers: Option<(u32, u32)>,
    pub blocks: Option<(u32, u32)>,
}

impl SyncRequest {
    pub fn create_from_headers(
        headers: HeadersToSync,
        from: u32,
        to: u32,
    ) -> Self {
        Self {
            headers: Some(headers),
            manifest: SyncRequestManifest {
                headers: Some((from, to)),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn create_from_para_headers(
        para_headers: ParaHeadersToSync,
        from: u32,
        to: u32,
        relay_at: u32,
    ) -> Self {
        Self {
            para_headers: Some(para_headers),
            manifest: SyncRequestManifest {
                // Multiple relaychain blocks may have same parachain head.
                // 
                // Even the parachain froms are same, if the relaychain height are different,
                // the proof cannot be validated. So we need to also pass the relaychain height.
                headers: Some((relay_at + 1, relay_at)),
                para_headers: Some((from, to)),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn create_from_combine_headers(
        headers: CombinedHeadersToSync,
        relay_from: u32,
        relay_to: u32,
        para_from: u32,
        para_to: u32,
    ) -> Self {
        Self {
            combined_headers: Some(headers),
            manifest: SyncRequestManifest {
                headers: Some((relay_from, relay_to)),
                para_headers: Some((para_from, para_to)),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn create_from_blocks(
        blocks: Blocks,
        from: u32,
        to: u32
    ) -> Self {
        Self {
            blocks: Some(blocks),
            manifest: SyncRequestManifest {
                blocks: Some((from, to)),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.headers.is_none()
            && self.para_headers.is_none()
            && self.combined_headers.is_none()
            && self.blocks.is_none()
    }
}

#[derive(Clone, Debug)]
pub struct WorkerSyncInfo {
    pub worker_id: String,
    pub headernum: u32,
    pub para_headernum: u32,
    pub blocknum: u32,
}

pub struct Repository {
    pub bus: Arc<Bus>,
    pub dsm: Arc<DataSourceManager>,
    pub headers_db: Arc<DB>,
    pub para_id: u32,
    pub next_number: u32,
    pub current_set_id: u64,
    pub current_authorities: Option<AuthorityList>,
}

impl Repository {
    // Polkadot: Number: 9688654, SetId: 891, Hash: 0x5fdbc952b059d7c26b8b7e6432bb2b40981c602ded8cf2be7d629a4ead96f156
    // Kusama: Number: 8325311, SetId: 3228, Hash: 0xff93a4a903207ad45af110a3e15f8b66c903a0045f886c528c23fe7064532b08
    pub async fn create(
        bus: Arc<Bus>,
        dsm: Arc<DataSourceManager>,
        headers_db: Arc<DB>,
    ) -> Result<Self> {

        let para_api = use_parachain_api!(dsm, false).unwrap();
        let para_id = para_api.get_paraid(None).await?;
        //let para_head_storage_key = para_api.paras_heads_key(para_id)?;
        info!("para id: {}", para_id);

        let relaychain_start_at = para_api.relay_parent_number().await? - 1;
        debug!("relaychain_start_at: {}", relaychain_start_at);

        let start_authority_set_id = if relaychain_start_at == 9688654 {
            891
        } else if relaychain_start_at == 8325311 {
            3228
        } else {
            0
        };

        Ok(Self {
            bus,
            dsm,
            headers_db,
            para_id,
            next_number: relaychain_start_at,
            current_set_id: start_authority_set_id,
            current_authorities: None,
        })
    }

    pub async fn background(&mut self, init_run: bool, need_init_verify: bool) -> Result<()> {
        if init_run {
            delete_tail_headers(&self.headers_db);
        }
        (self.next_number, self.current_set_id, self.current_authorities) = find_valid_num(
            &self.headers_db,
            self.next_number,
            self.current_set_id,
            need_init_verify,
        );
        info!("next number: {}", self.next_number);
        info!("current authority set id: {}", self.current_set_id);

        'forever: loop {
            let relay_api = match use_relaychain_api!(self.dsm, false) {
                Some(instance) => instance,
                None => {
                    error!("No valid data source, wait 10 seconds");
                    sleep(Duration::from_secs(10)).await;
                    continue;
                },
            };

            let blocks_sub = relay_api.blocks().subscribe_finalized().await;
            let mut blocks_sub = match blocks_sub {
                Ok(blocks_sub) => blocks_sub,
                Err(e) => {
                    error!("Subscribe finalized blocks failed, wait 10 seconds. {e}");
                    sleep(Duration::from_secs(10)).await;
                    continue;
                },
            };

            while let Some(block) = blocks_sub.next().await {
                let block = match block {
                    Ok(block) => block,
                    Err(e) => {
                        error!("Got error for next block. {e}");
                        continue;
                    },
                };

                if block.number() < self.next_number {
                    continue;
                }

                let prev_finalized_at = self.next_number - 1;
                match self.fill_gap(&relay_api, block.number()).await {
                    Ok(_) => {
                        if init_run {
                            info!("Filled until #{}", self.next_number - 1);
                            break 'forever
                        }
                    },
                    Err(err) => {
                        if init_run {
                            return Err(err);
                        }
                        error!("Got error when filling gap. {err}");
                        continue;
                    },
                }

                let broadcast_result = prepare_and_broadcast(
                    self.bus.clone(),
                    self.dsm.clone(),
                    self.headers_db.clone(),
                    self.para_id,
                    prev_finalized_at,
                    self.next_number - 1,
                ).await;
                if let Err(err) = broadcast_result {
                    error!("Met error when try to prepare and broadcast headers. {err}");
                }
            }
        }

        let _ = self.headers_db.flush();
        self.headers_db.compact_range(None::<&[u8]>, None::<&[u8]>);

        Ok(())
    }

    async fn fill_gap(
        &mut self,
        relay_api: &ChainApi,
        potential_chaintip: u32,
    ) -> Result<()> {
        loop {
            if self.next_number > potential_chaintip {
                break Ok(())
            }

            let mut try_count = 0 as usize;
            let headers = loop {
                let headers = pherry::get_headers(&relay_api, self.next_number).await?;
                let last_header = headers.last().unwrap();
                let justifications = last_header.justification.as_ref().expect("last header from proof api should has justification");
                match &self.current_authorities {
                    Some(authorities) => {
                        let result = pherry::verify_with_prev_authority_set(
                            self.current_set_id,
                            authorities,
                            &last_header.header,
                            justifications,
                        );
                        if let Ok(_) = result {
                            break headers
                        }
                    }
                    _ => break headers,
                }
                try_count += 1;
                if try_count >= 3 {
                    return Err(anyhow!("Tried three times, cannot retrieve a valid justification."))
                }
            };

            let last_header = headers.last().unwrap().header.clone();
            let last_number = put_headers_to_db(
                self.headers_db.clone(),
                headers,
                potential_chaintip
            )?;
            self.next_number = last_number + 1;

            if let Some(next_authorities) = phactory_api::blocks::find_scheduled_change(&last_header) {
                self.current_authorities = Some(next_authorities.next_authorities);
                self.current_set_id += 1;
            }
        }
    }
}

pub async fn get_load_state_request(
    bus: Arc<Bus>,
    dsm: Arc<DataSourceManager>,
    worker_id: String,
    public_key: Sr25519Public,
    prefer_number: u32,
) {
    let para_api = use_parachain_api!(dsm, true).unwrap();
    match pherry::chain_client::search_suitable_genesis_for_worker(&para_api, &public_key, Some(prefer_number)).await {
        Ok((block_number, state)) => {
            let request = ChainState::new(block_number, state);
            let _ = bus.send_pruntime_request(worker_id, PRuntimeRequest::LoadChainState(request));
        },
        Err(err) => {
            let _ = bus.send_worker_mark_error(worker_id, err.to_string());
        },
    }
}

pub async fn do_request_next_sync(
    bus: Arc<Bus>,
    dsm: Arc<DataSourceManager>,
    headers_db: Arc<DB>,
    info: WorkerSyncInfo,
) {
    let mut try_count = 0;
    let request = loop {
        match generate_sync_request(dsm.clone(), headers_db.clone(), info.clone()).await {
            Ok(request) => break request,
            Err(err) => {
                try_count += 1;
                let err_msg = format!("Fail to generate_sync_request for {} times. Retrying... Last Error: {}", try_count, err);
                error!("[{}] {}", info.worker_id, err_msg);
                if try_count >= 3 {
                    bus.send_worker_mark_error(info.worker_id.clone(), err_msg);
                }
                tokio::time::sleep(std::time::Duration::from_secs(6)).await;
            },
        }
    };
    if request.headers.is_some() || request.para_headers.is_some() || request.combined_headers.is_some() || request.blocks.is_some() {
        trace!("[{}] sending sync request. {:?}", info.worker_id, info);
    } else {
        trace!("[{}] sending empty sync request.", info.worker_id);
    }
    let manifest = request.manifest.clone();
    let _ = bus.send_pruntime_request(info.worker_id.clone(), PRuntimeRequest::Sync(request));
    preload_next_sync(dsm, headers_db, info, &manifest).await;
}

async fn preload_next_sync(
    dsm: Arc<DataSourceManager>,
    headers_db: Arc<DB>,
    info: WorkerSyncInfo,
    manifest: &SyncRequestManifest,
) {
    let mut next_info = info.clone();
    if let Some((_, to)) = &manifest.headers {
        next_info.headernum = to + 1;
    }
    if let Some((_, to)) = &manifest.para_headers {
        next_info.para_headernum = to + 1;
    }
    if let Some((_, to)) = &manifest.blocks {
        next_info.blocknum = to + 1;
    }
    let _ = generate_sync_request(dsm, headers_db.clone(), next_info).await;
}

async fn generate_sync_request(
    dsm: Arc<DataSourceManager>,
    headers_db: Arc<DB>,
    info: WorkerSyncInfo,
) -> Result<SyncRequest> {
    if info.blocknum < info.para_headernum {
        let to = std::cmp::min((info.blocknum + 3) / 4 * 4, info.para_headernum - 1);
        return dsm
            .fetch_storage_changes(info.blocknum, to)
            .await
            .map(|blocks| SyncRequest::create_from_blocks(blocks, info.blocknum, to));
    }

    if let Some((para_headernum, proof)) = get_para_headernum(dsm.clone(), info.headernum - 1).await.unwrap_or(None) {
        if para_headernum > 0 && info.para_headernum <= para_headernum {
            return dsm
                .get_para_headers(info.para_headernum, para_headernum)
                .await
                .map(|mut headers| {
                    headers.proof = proof;
                    SyncRequest::create_from_para_headers(
                        headers,
                        info.para_headernum,
                        para_headernum,
                        info.headernum - 1,
                    )
                });
        }
    } else {
        warn!("Failed to get para headernum at {}", info.headernum - 1);
    }

    trace!("[{}] Getting from headers_db: {}", info.worker_id, info.headernum);
    if let Some(headers) = get_current_point(headers_db, info.headernum) {
        let headers = headers
            .into_iter()
            .filter(|header| header.header.number >= info.headernum)
            .collect::<Vec<_>>();
        if let Some(last_header) = headers.last() {
            let to = last_header.header.number;
            let headers = phactory_api::prpc::HeadersToSync::new(headers, None);
            return Ok(SyncRequest::create_from_headers(headers, info.headernum, to));
        }
    }

    trace!("[{}] Got nothing to sync", info.worker_id);
    Ok(SyncRequest { ..Default::default() })
}

async fn get_para_headernum(
    dsm: Arc<DataSourceManager>,
    relay_headernum: u32,
) -> Result<Option<(u32, Vec<Vec<u8>>)>> {
    dsm.get_para_header_by_relay_header(relay_headernum).await
}

async fn prepare_and_broadcast(
    bus: Arc<Bus>,
    dsm: Arc<DataSourceManager>,
    headers_db: Arc<DB>,
    para_id: u32,
    prev_relaychain_finalized_at: u32,
    curr_relaychain_finalized_at: u32,
) -> Result<()> {
    let relay_api = use_relaychain_api!(dsm, false).expect("should have relaychain api");
    let para_api = use_parachain_api!(dsm, false).expect("should have parachain api");

    let relay_from = prev_relaychain_finalized_at + 1;
    let headers = get_current_point(headers_db.clone(), relay_from)
        .expect("should already put headers into db")
        .into_iter()
        .filter(|h| prev_relaychain_finalized_at < h.header.number)
        .collect::<Vec<_>>();

    let relay_to = curr_relaychain_finalized_at;
    if headers.first().expect("should have headers").header.number != relay_from {
        return Err(anyhow!("first header from DB is not match the prev one"));
    }
    if headers.last().expect("should have headers").header.number != relay_to {
        return Err(anyhow!("first header from DB is not match the prev one"));
    }
    let relay_to_hash = (&headers.last().unwrap().header).hash();

    let (para_prev, _) = get_para_headernum(dsm.clone(), prev_relaychain_finalized_at).await?
        .expect(&format!("Unknown para header for relay #{prev_relaychain_finalized_at}"));
    let (para_header, proof) = pherry::get_finalized_header_with_paraid(&relay_api, para_id, relay_to_hash.clone())
        .await?
        .expect(&format!("Unknown para header for relay #{relay_to} {relay_to_hash}"));
    let para_to = para_header.number;

    if para_to < para_prev {
        return Err(anyhow!("Para number {} from relaychain is smaller than the newest in prb {}", para_to, para_prev));
    }
    let para_from = para_prev + 1;

    let sync_request = if para_from > para_to {
        info!("Broadcasting header: relaychain from {} to {}.", relay_from, relay_to);
        SyncRequest::create_from_headers(
            HeadersToSync::new(headers.clone(), None),
            relay_from,
            relay_to
        )
    } else {
        let _ = bus.send_processor_event(ProcessorEvent::RequestUpdateSessionInfo);
        let para_headers = pherry::get_parachain_headers(&para_api, None, para_from, para_to).await?;
        info!("Broadcasting header: relaychain from {} to {}, parachain from {} to {}.",
            relay_from, relay_to, para_from, para_to);
        let headers = CombinedHeadersToSync::new(
            headers.clone(),
            None,
            para_headers,
            proof
        );
        SyncRequest::create_from_combine_headers(headers, relay_from, relay_to, para_from, para_to)
        /*
        SyncRequest {
            headers: Some(HeadersToSync::new(headers.clone(), None)),
            //para_headers: Some(ParaHeadersToSync::new(para_headers, proof)),
            para_headers: None,
            combined_headers: None,
            blocks: None,
            manifest: SyncRequestManifest {
                headers: Some((relay_from, relay_to)),
                //para_headers: Some((para_from, para_to)),
                para_headers: None,
                blocks: None,
            },
        } */
    };
    let _ = bus.send_processor_event(ProcessorEvent::BroadcastSync((
        sync_request,
        ChaintipInfo {
            relaychain: relay_to,
            parachain: para_to,
        },
    )));

    Ok(())
}
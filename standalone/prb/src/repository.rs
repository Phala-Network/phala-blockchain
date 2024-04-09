use anyhow::Result;
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
use crate::{headers_db::*, use_parachain_api};
use crate::processor::{PRuntimeRequest, ProcessorEvent};
use crate::pool_operator::DB;

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

pub fn encode_u32(val: u32) -> [u8; 4] {
    use byteorder::{ByteOrder, BigEndian};
    let mut buf = [0; 4];
    BigEndian::write_u32(&mut buf, val);
    return buf;
}

#[derive(Clone, Debug)]
pub struct WorkerSyncInfo {
    pub worker_id: String,
    pub headernum: u32,
    pub para_headernum: u32,
    pub blocknum: u32,
}

pub enum RepositoryEvent {
    PreloadWorkerSyncInfo(WorkerSyncInfo),
    UpdateWorkerSyncInfo(WorkerSyncInfo),
}

pub type RepositoryRx = mpsc::UnboundedReceiver<RepositoryEvent>;
pub type RepositoryTx = mpsc::UnboundedSender<RepositoryEvent>;

pub struct Repository {
    pub rx: RepositoryRx,
    pub bus: Arc<Bus>,
    pub dsm: Arc<DataSourceManager>,
    pub headers_db: Arc<DB>,
}

impl Repository {
    // Polkadot: Number: 9688654, SetId: 891, Hash: 0x5fdbc952b059d7c26b8b7e6432bb2b40981c602ded8cf2be7d629a4ead96f156
    // Kusama: Number: 8325311, SetId: 3228, Hash: 0xff93a4a903207ad45af110a3e15f8b66c903a0045f886c528c23fe7064532b08
    pub async fn create(
        rx: RepositoryRx,
        bus: Arc<Bus>,
        dsm: Arc<DataSourceManager>,
        headers_db: Arc<DB>,
    ) -> Result<Self> {

        //db.iterator(rocksdb::IteratorMode::From((), ()))

        let relay_api = dsm.clone().current_relaychain_rpc_client(false).await.unwrap().client.clone();
        let para_api = dsm.clone().current_parachain_rpc_client(true).await.unwrap().client.clone();

        let para_id = para_api.get_paraid(None).await?;
        //let para_head_storage_key = para_api.paras_heads_key(para_id)?;
        info!("para id: {}", para_id);

        let relaychain_start_at = para_api.relay_parent_number().await? - 1;
        debug!("relaychain_start_at: {}", relaychain_start_at);

        let mut current_num = match get_previous_authority_set_change_number(headers_db.clone(), std::u32::MAX) {
            Some(num) => num + 1,
            None => relaychain_start_at
        };
        info!("current number: {}", current_num);

        loop {
            let relay_chaintip = relay_api.latest_finalized_block_number().await?;
            if current_num > relay_chaintip {
                break
            }

            let headers = pherry::get_headers(&relay_api, current_num).await?;
            let last_number = put_headers_to_db(headers_db.clone(), headers, relay_chaintip)?;
            current_num = last_number + 1;
        }

        headers_db.compact_range(None::<&[u8]>, None::<&[u8]>);
        let _ = headers_db.flush();

        Ok(Self {
            rx,
            bus,
            dsm,
            headers_db,
        })
    }

    pub async fn master_loop(
        &mut self,
    ) -> Result<()> {
        loop {
            let event = self.rx.recv().await;
            if event.is_none() {
                break;
            }

            match event.unwrap() {
                RepositoryEvent::PreloadWorkerSyncInfo(info) => {
                    tokio::spawn(get_sync_request(
                        self.dsm.clone(),
                        self.headers_db.clone(),
                        info,
                    ));
                },
                RepositoryEvent::UpdateWorkerSyncInfo(info) => {
                    trace!("[{}] Received UpdateWorkerSyncInfo. header({}) para_header({}) block({})",
                        info.worker_id, info.headernum, info.para_headernum, info.blocknum);
                    let bus = self.bus.clone();
                    let dsm = self.dsm.clone();
                    let headers_db = self.headers_db.clone();
                    tokio::spawn(async move {
                        let request = loop {
                            match get_sync_request(dsm.clone(), headers_db.clone(), info.clone()).await {
                                Ok(request) => break request,
                                Err(err) => {
                                    error!("[{}] fail to get_sync_request, {}", info.worker_id, err);
                                    // TODO: Send some error message
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
                        let _ = bus.send_repository_event(RepositoryEvent::PreloadWorkerSyncInfo(next_info));
                    });
                },
            }
        }

        Ok(())
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

async fn get_sync_request(
    dsm: Arc<DataSourceManager>,
    headers_db: Arc<DB>,
    info: WorkerSyncInfo,
) -> Result<SyncRequest> {
    if info.blocknum < info.para_headernum {
        let to = std::cmp::min((info.blocknum + 3) / 4 * 4, info.para_headernum - 1);
        return dsm.clone()
            .fetch_storage_changes(info.blocknum, to)
            .await
            .map(|blocks| SyncRequest::create_from_blocks(blocks, info.blocknum, to));
    }

    if let Some((para_headernum, proof)) = get_para_headernum(dsm.clone(), info.headernum - 1).await? {
        if info.para_headernum <= para_headernum {
            return dsm.clone()
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
    dsm.clone().get_para_header_by_relay_header(relay_headernum).await
}

pub async fn relaychain_api(dsm: Arc<DataSourceManager>, full: bool) -> ChainApi {
    dsm.clone().current_relaychain_rpc_client(full).await.unwrap().client.clone()
}

pub async fn parachain_api(dsm: Arc<DataSourceManager>, full: bool) -> ChainApi {
    dsm.clone().current_parachain_rpc_client(full).await.unwrap().client.clone()
}

pub async fn relaychain_cache(dsm: Arc<DataSourceManager>) -> CacheClient {
    dsm.clone().current_relaychain_headers_cache().await.unwrap().client.clone()
}

pub async fn keep_syncing_headers(
    bus: Arc<Bus>,
    dsm: Arc<DataSourceManager>,
    headers_db: Arc<DB>,
) -> Result<()> {
    // TODO: Handle Error
    let para_api= parachain_api(dsm.clone(), false).await;
    let para_id = para_api.get_paraid(None).await?;

    let relay_api = relaychain_api(dsm.clone(), false).await;
    //let para_head_storage_key = relay_api.paras_heads_key(para_id)?;

    let relaychain_start_block = para_api.relay_parent_number().await? - 1;
    info!("relaychain_start_block: {relaychain_start_block}");

    let mut current_relay_number = get_previous_authority_set_change_number(headers_db.clone(), std::u32::MAX).unwrap();
    let (mut current_para_number, _) = pherry::get_parachain_header_from_relaychain_at(
        &relay_api,
        &para_api,
        &None,
        current_relay_number,
    ).await?;

    loop {
        let relay_api = match dsm.clone().current_relaychain_rpc_client(false).await {
            Some(instance) => instance.client.clone(),
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

            if block.number() <= current_relay_number {
                continue;
            }

            let relay_from = current_relay_number + 1;
            let headers = match pherry::get_headers(&relay_api, relay_from).await {
                Ok(headers) => headers,
                Err(e) => {
                    error!("Failed to get headers with justification. {e}");
                    continue;
                }
            };
            let relay_to = headers.last().unwrap().header.number;
            let relay_to_hash = (&headers.last().unwrap().header).hash();

            match pherry::get_finalized_header_with_paraid(&relay_api, para_id, relay_to_hash.clone()).await {
                Ok(Some((para_header, proof))) => {
                    let para_to = para_header.number;
                    if para_to < current_para_number {
                        error!("Para number {} from relaychain is smaller than the newest in prb {}", para_to, current_para_number);
                        continue;
                    }
                    let para_from = current_para_number + 1;

                    let sync_request = if para_from > para_to {
                        info!("Broadcasting header: relaychain from {} to {}.", relay_from, relay_to);
                        SyncRequest::create_from_headers(
                            HeadersToSync::new(headers.clone(), None),
                            relay_from,
                            relay_to
                        )
                    } else {
                        let _ = bus.send_processor_event(ProcessorEvent::RequestUpdateSessionInfo);
                        match pherry::get_parachain_headers(&para_api, None, current_para_number + 1, para_to).await {
                            Ok(para_headers) => {
                                info!("Broadcasting header: relaychain from {} to {}, parachain from {} to {}.",
                                    relay_from, relay_to, para_from, para_to);
                                let headers = CombinedHeadersToSync::new(
                                    headers.clone(),
                                    None,
                                    para_headers,
                                    proof
                                );
                                SyncRequest::create_from_combine_headers(headers, relay_from, relay_to, para_from, para_to)
                            },
                            Err(e) => {
                                error!("Failed to get para headers. {e}");
                                continue;
                            },
                        }
                    };
                    current_relay_number = relay_to;
                    current_para_number = para_to;
                    let _ = bus.send_processor_event(ProcessorEvent::BroadcastSync((
                        sync_request,
                        ChaintipInfo {
                            relaychain: current_relay_number,
                            parachain: current_para_number,
                        },
                    )));
                },
                Ok(None) => {
                    error!("Unknown para header for relay #{relay_to} {relay_to_hash}");
                    continue;
                },
                Err(err) => {
                    error!("Fail to get para number with proof {err}");
                    continue;
                },
            }

            if let Err(err) = put_headers_to_db(headers_db.clone(), headers, relay_to) {
                error!("Failed to put headers to DB, {err}");
            }

            info!("relaychain_chaintip: {}, parachain_chaintip: {}", current_relay_number, current_para_number);
        }
    }
}
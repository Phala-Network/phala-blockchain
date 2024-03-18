use std::sync::Arc;
use anyhow::Result;
use core::time::Duration;
use futures::StreamExt;
use log::{debug, error, info, warn};
use phaxt::ChainApi;
use tokio::sync::mpsc;
use tokio::time::sleep;

use crate::datasource::DataSourceManager;
use crate::processor::{send_processor_event, BroadcastInfo, PRuntimeRequest, ProcessorEvent, ProcessorEventTx, SyncInfo, SyncRequest};
use crate::tx::DB;

use phactory_api::blocks::{find_scheduled_change, HeadersToSync};
use pherry::headers_cache::Client as CacheClient;
use prpc::codec::scale::{Decode, Encode};

pub fn encode_u32(val: u32) -> [u8; 4] {
    use byteorder::{ByteOrder, BigEndian};
    let mut buf = [0; 4];
    BigEndian::write_u32(&mut buf, val);
    return buf;
}

#[derive(Clone, Debug)]
pub struct WorkerSyncInfo {
    pub worker_id: usize,
    pub headernum: u32,
    pub para_headernum: u32,
    pub blocknum: u32,
}

pub enum DataProviderEvent {
    PreloadWorkerSyncInfo(WorkerSyncInfo),
    UpdateWorkerSyncInfo(WorkerSyncInfo),
}

pub type DataProviderEventRx = mpsc::UnboundedReceiver<DataProviderEvent>;
pub type DataProviderEventTx = mpsc::UnboundedSender<DataProviderEvent>;

pub struct DataProvider {
    pub dsm: Arc<DataSourceManager>,
    pub headers_db: Arc<DB>,
    pub rx: DataProviderEventRx,
    pub tx: Arc<DataProviderEventTx>,
    pub processor_event_tx: Arc<ProcessorEventTx>,
}

impl DataProvider {
    // Polkadot: Number: 9688654, SetId: 891, Hash: 0x5fdbc952b059d7c26b8b7e6432bb2b40981c602ded8cf2be7d629a4ead96f156
    // Kusama: Number: 8325311, SetId: 3228, Hash: 0xff93a4a903207ad45af110a3e15f8b66c903a0045f886c528c23fe7064532b08
    pub async fn init(
        &self,
    ) -> Result<()> {

        //db.iterator(rocksdb::IteratorMode::From((), ()))

        let relay_api = self.dsm.clone().current_relaychain_rpc_client(false).await.unwrap().client.clone();
        let para_api = self.dsm.clone().current_parachain_rpc_client(true).await.unwrap().client.clone();

        let para_id = para_api.get_paraid(None).await?;
        //let para_head_storage_key = para_api.paras_heads_key(para_id)?;
        info!("para id: {}", para_id);

        let relaychain_start_at = para_api.relay_parent_number().await? - 1;
        debug!("relaychain_start_at: {}", relaychain_start_at);

        let mut current_num = match get_previous_authority_set_change_number(self.headers_db.clone(), std::u32::MAX) {
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
            let last_number = put_headers_to_db(self.headers_db.clone(), headers, relay_chaintip)?;
            current_num = last_number + 1;
        }

        self.headers_db.compact_range(None::<&[u8]>, None::<&[u8]>);
        let _ = self.headers_db.flush();

        /*
        for result in self.headers_db.iterator(rocksdb::IteratorMode::Start) {
            match result {
                Ok((_, value)) => {
                    let authority_set_change_at = match HeadersToSync::decode(&mut &value[..]) {
                        Ok(headers) => headers.last().unwrap().header.number,
                        Err(_) => break,
                    };
                    let block_info = relaychain_cache(self.dsm.clone()).await.get_header(authority_set_change_at).await;
                    match block_info {
                        Ok(block_info) => {
                            info!("{}: {}", authority_set_change_at, block_info.para_header.is_some());

                        },
                        Err(_) => break,
                    }
                },
                Err(_) => break,
            }
        }

        //let mut last_known_block_num = 8325311 as u32 - 1;
        //let relaychain_authority_set_start_block_num = Vec::<u32>::new();
        //relaychain_authority_set_start_block_num.as_mut().push(0);

        for i in 3229..8605 {
            let headers = pherry::get_headers(&relay_api, last_known_block_num + 1).await?;
            last_known_block_num = headers.last().unwrap().header.number;
            let just = headers.last().unwrap().justification.as_ref().unwrap().len();
            info!("{i}\t{last_known_block_num}\t{just}");
        }
    */
        Ok(())
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
                DataProviderEvent::PreloadWorkerSyncInfo(info) => {
                    let dsm = self.dsm.clone();
                    let headers_db = self.headers_db.clone();
                    tokio::spawn(async move {
                        let _ = get_sync_request(dsm, headers_db, &info).await;
                    });
                },
                DataProviderEvent::UpdateWorkerSyncInfo(info) => {
                    let dsm = self.dsm.clone();
                    let headers_db = self.headers_db.clone();
                    let processor_event_tx = self.processor_event_tx.clone();
                    tokio::spawn(async move {
                        let request = loop {
                            match get_sync_request(dsm.clone(), headers_db.clone(), &info).await {
                                Ok(request) => break request,
                                Err(err) => {
                                    error!("fail to get_sync_request, {}", err);
                                },
                            }
                        };
                        if request.headers.is_some() || request.para_headers.is_some() || request.combined_headers.is_some() || request.blocks.is_some() {
                        } else {
                            info!("empty response: {:?}", info);
                        }
                        let send_result = processor_event_tx.clone().send(
                            ProcessorEvent::PRuntimeRequest((info.worker_id, PRuntimeRequest::Sync(request)))
                        );
                        if let Err(send_error) = send_result {
                            error!("{:?}", send_error);
                            std::process::exit(255);
                        }
                    });
                },
            }
        }

        Ok(())
    }
}

fn get_current_point(db: Arc<DB>, num: u32) -> Option<HeadersToSync> {
    let mut iter = db.iterator(rocksdb::IteratorMode::From(&encode_u32(num), rocksdb::Direction::Forward));
    if let Some(Ok((_, value))) = iter.next() {
        match HeadersToSync::decode(&mut &value[..]) {
            Ok(headers) => return Some(headers),
            Err(_) => {},
        };
    }
    None
}

fn get_previous_authority_set_change_number(db: Arc<DB>, num:u32) -> Option<u32> {
    let mut iter = db.iterator(rocksdb::IteratorMode::From(&encode_u32(num), rocksdb::Direction::Reverse));
    if let Some(Ok((_, value))) = iter.next() {
        match HeadersToSync::decode(&mut &value[..]) {
            Ok(headers) => return Some(headers.last().unwrap().header.number),
            Err(_) => {},
        };
    }
    None
}

async fn get_sync_request(
    dsm: Arc<DataSourceManager>,
    headers_db: Arc<DB>,
    info: &WorkerSyncInfo,
) -> Result<SyncRequest> {
    if info.blocknum < info.para_headernum {
        let to = std::cmp::min((info.blocknum + 3) / 4 * 4, info.para_headernum - 1);
        return dsm.clone()
            .fetch_storage_changes(info.blocknum, to)
            .await
            .map(|blocks| SyncRequest { blocks: Some(blocks), ..Default::default() });
    }

    if let Some((para_headernum, proof)) = get_para_headernum(dsm.clone(), info.headernum - 1).await? {
        if info.para_headernum <= para_headernum {
            return dsm.clone()
                .get_para_headers(info.para_headernum, para_headernum)
                .await
                .map(|mut headers| {
                    headers.proof = proof;
                    SyncRequest {
                        para_headers: Some(headers),
                        ..Default::default()
                    }
                });
        }
    } else {
        warn!("Failed to get para headernum at {}", info.headernum - 1);
    }

    if let Some(headers) = get_current_point(headers_db, info.headernum) {
        let headers = headers
            .into_iter()
            .filter(|header| header.header.number >= info.headernum)
            .collect::<Vec<_>>();
        return Ok(SyncRequest {
            headers: Some(phactory_api::prpc::HeadersToSync::new(headers, None)),
            ..Default::default()
        });
    }

    info!("nothing can get");
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
    dsm: Arc<DataSourceManager>,
    headers_db: Arc<DB>,
    processor_event_tx: Arc<ProcessorEventTx>
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
            let mut broadcast_info = BroadcastInfo {
                sync_info: SyncInfo {
                    headernum: Some(relay_from),
                    ..Default::default()
                },
                ..Default::default()
            };

            //let header = block.header();

            let headers = match pherry::get_headers(&relay_api, relay_from).await {
                Ok(headers) => headers,
                Err(e) => {
                    error!("Failed to get headers with justification. {e}");
                    continue;
                }
            };
            let relay_to = headers.last().unwrap().header.number;
            let relay_to_hash = (&headers.last().unwrap().header).hash();
            broadcast_info.relay_chaintip = relay_to;

            match pherry::get_finalized_header_with_paraid(&relay_api, para_id, relay_to_hash.clone()).await {
                Ok(Some((para_header, proof))) => {
                    let para_to = para_header.number;
                    if para_to < current_para_number {
                        error!("Para number {} from relaychain is smaller than the newest in prb {}", para_to, current_para_number);
                    }
                    broadcast_info.para_chaintip = para_to;

                    let para_from = current_para_number + 1;
                    broadcast_info.sync_info.para_headernum = Some(para_from);
                    broadcast_info.sync_info.blocknum = Some(para_from);
                    let sync_request = if para_from > para_to {
                        info!("Broadcasting header: relaychain from {} to {}.", relay_from, relay_to);
                        SyncRequest {
                            headers: Some(phactory_api::prpc::HeadersToSync::new(
                                headers.clone(),
                                None
                            )),
                            ..Default::default()
                        }
                    } else {
                        match pherry::get_parachain_headers(&para_api, None, current_para_number + 1, para_to).await {
                            Ok(para_headers) => {
                                info!("Broadcasting header: relaychain from {} to {}, parachain from {} to {}.",
                                    relay_from,
                                    relay_to,
                                    para_from,
                                    para_to,
                                );
                                SyncRequest {
                                    combined_headers: Some(phactory_api::prpc::CombinedHeadersToSync::new(
                                        headers.clone(),
                                        None,
                                        para_headers,
                                        proof
                                    )),
                                    ..Default::default()
                                }
                            },
                            Err(e) => {
                                error!("Failed to get para headers. {e}");
                                continue;
                            },
                        }
                    };
                    send_processor_event(processor_event_tx.clone(), ProcessorEvent::BroadcastSyncRequest((sync_request, broadcast_info)));
                    current_relay_number = relay_to;
                    current_para_number = para_to;
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

    Ok(())
}

fn put_headers_to_db(
    headers_db: Arc<DB>,
    new_headers: HeadersToSync,
    known_chaintip: u32,
) -> Result<u32> {
    let first_new_number = new_headers.first().unwrap().header.number;
    let mut headers = match get_current_point(headers_db.clone(), first_new_number) {
        Some(headers) => {
            let _ = headers_db.delete(encode_u32(std::u32::MAX));
            headers
        },
        None => vec![], 
    };
    for header in &mut headers {
        header.justification = None;
    }
    headers.extend(new_headers);
    let headers = headers;

    let mut last_num: Option<u32> = None;
    for header in &headers {
        if let Some(num) = last_num {
            assert!(num + 1 == header.header.number, "prev {}, current {}, not match", num, header.header.number);
        }
        last_num = Some(header.header.number);
    }

    let from = headers.first().unwrap().header.number;
    let to = headers.last().unwrap().header.number;
    let with_authority_change = find_scheduled_change(&headers.last().unwrap().header).is_some();

    let key = if with_authority_change {
        encode_u32(to)
    } else if to >= known_chaintip {
        encode_u32(std::u32::MAX)
    } else {
        error!("Should not happen: prove_finality API returns a non-chaintip block without authority set change");
        encode_u32(to)
    };

    let encoded_val = headers.encode();
    let headers_size = encoded_val.len();

    if let Err(err) = headers_db.put(key, encoded_val) {
        anyhow::bail!("Failed to write DB {err}");
    }

    let justification = headers.last().unwrap().justification.as_ref().unwrap();
    debug!(
        "put into headers_db: from {} to {}, count {}, justification size: {}, headers size: {}",
        from,
        to,
        to - from + 1,
        justification.len(),
        headers_size,
    );

    Ok(to)
}
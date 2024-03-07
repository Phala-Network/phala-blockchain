use std::sync::Arc;
use anyhow::Result;
use core::time::Duration;
use futures::StreamExt;
use log::{error, debug, info};
use phaxt::ChainApi;
use tokio::sync::mpsc;
use tokio::time::sleep;

use crate::datasource::DataSourceManager;
use crate::processor::{ProcessorEvent, ProcessorEventTx, PRuntimeRequest, SyncRequest};
use crate::tx::DB;

use phactory_api::blocks::HeadersToSync;
//use phactory_api::prpc::ParaHeadersToSync;
use prpc::codec::scale::{Decode, Encode};
use pherry::headers_cache::Client as CacheClient;

pub fn encode_u32(val: u32) -> [u8; 4] {
    use byteorder::{ByteOrder, BigEndian};
    let mut buf = [0; 4];
    BigEndian::write_u32(&mut buf, val);
    return buf;
}

#[derive(Clone)]
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

        let mut current_num = match get_previous_authority_set_change_number(self.headers_db.clone(), std::u32::MAX).await {
            Some(num) => num + 1,
            None => relaychain_start_at
        };
        info!("current number: {}", current_num);

        let relay_chaintip_num = relay_api.latest_finalized_block_number().await?;

        let mut total_size: usize = 0;

        while current_num <= relay_chaintip_num {
            let headers = pherry::get_headers(&relay_api, current_num).await?;

            let from = headers.first().unwrap().header.number;
            let to = headers.last().unwrap().header.number;

            let key = encode_u32(to);
            let encoded = headers.encode();
            let headers_size = encoded.len();

            self.headers_db.put(key, encoded);

            total_size += headers_size;
            let justification = headers.last().unwrap().justification.as_ref().unwrap();

            info!("from {} to {}, count {}, justification size: {}, all size: {}, total size: {}", from, to, to - from + 1, justification.len(), headers_size, total_size);

            let last_header = headers.last().unwrap();
            current_num = last_header.header.number + 1;
        }

        self.headers_db.flush();
        self.headers_db.compact_range(None::<&[u8]>, None::<&[u8]>);

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
 */
        //let mut last_known_block_num = 8325311 as u32 - 1;
        //let relaychain_authority_set_start_block_num = Vec::<u32>::new();
        //relaychain_authority_set_start_block_num.as_mut().push(0);

        /*
        for i in 3229..8605 {
            let headers = pherry::get_headers(&relay_api, last_known_block_num + 1).await?;
            last_known_block_num = headers.last().unwrap().header.number;
            let just = headers.last().unwrap().justification.as_ref().unwrap().len();
            info!("{i}\t{last_known_block_num}\t{just}");
        }
    */
        /*
        for set_id in self.relaychain_start_authority_set_id.. {
            
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
                            let send_result = processor_event_tx.clone().send(
                                ProcessorEvent::PRuntimeRequest((info.worker_id, PRuntimeRequest::Sync(request)))
                            );
                            if let Err(send_error) = send_result {
                                error!("{:?}", send_error);
                                std::process::exit(255);
                            }
                        }
                    });
                },
            }
        }

        Ok(())
    }
}

async fn get_current_point(db: Arc<DB>, num: u32) -> Option<HeadersToSync> {
    let mut iter = db.iterator(rocksdb::IteratorMode::From(&encode_u32(num), rocksdb::Direction::Forward));
    if let Some(Ok((_, value))) = iter.next() {
        match HeadersToSync::decode(&mut &value[..]) {
            Ok(headers) => return Some(headers),
            Err(_) => {},
        };
    }
    None
}

async fn get_previous_authority_set_change_number(db: Arc<DB>, num:u32) -> Option<u32> {
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
        let mut to = std::cmp::min((info.blocknum + 3) / 4 * 4, info.para_headernum - 1);
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
    }

    if let Some(headers) = get_current_point(headers_db, info.headernum).await {
        return Ok(SyncRequest {
            headers: Some(phactory_api::prpc::HeadersToSync::new(headers, None)),
            ..Default::default()
        });
    }

    Ok(SyncRequest {..Default::default()})
}

async fn get_para_headernum(
    dsm: Arc<DataSourceManager>,
    relay_headernum: u32,
) -> Result<Option<(u32, Vec<Vec<u8>>)>> {
    relaychain_cache(dsm.clone())
        .await
        .get_header(relay_headernum)
        .await
        .map(|info| info.para_header.map(
            |para_header| (para_header.fin_header_num, para_header.proof)
        ))
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
    processor_event_tx: Arc<ProcessorEventTx>
) -> Result<()> {
    // TODO: Handle Error
    let para_api= parachain_api(dsm.clone(), false).await;
    let para_id = para_api.get_paraid(None).await?;

    let relay_api = relaychain_api(dsm.clone(), false).await;
    let para_head_storage_key = relay_api.paras_heads_key(para_id)?;

    let relaychain_start_block = para_api.relay_parent_number().await? - 1;
    info!("relaychain_start_block: {relaychain_start_block}");

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

            let hash = block.hash();
            let number = block.number();
            let header = block.header();

            let para_header_raw_storage = block.storage().fetch_raw(&para_head_storage_key).await;
            let para_header_raw_storage = match para_header_raw_storage {
                Ok(storage) => {
                    match storage {
                        Some(raw_storage) => raw_storage,
                        None => {
                            error!("Got None for raw para header storage. Block: {hash} {number}");
                            continue;
                        },
                    }
                },
                Err(e) => {
                    error!("Got error for raw para header storage. Block: {hash} {number}, {e}");
                    continue;
                },
            };

            let para_header = pherry::chain_client::decode_parachain_header(para_header_raw_storage);
            let para_header = match para_header {
                Ok(para_header) => para_header,
                Err(e) => {
                    error!("Failed to decode header. Block: {hash}, {e}");
                    continue;
                },
            };
            let proof = pherry::chain_client::read_proof(&relay_api, Some(hash), &para_head_storage_key).await;
            let proof = match proof {
                Ok(proof) => proof,
                Err(e) => {
                    error!("Failed to decode proof. Block: {hash}, {e}");
                    continue;
                },
            };

            info!("relaychain: {}, parachain: {}", number, para_header.number);
        }
    }

    Ok(())
}
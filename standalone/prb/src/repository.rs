use anyhow::{anyhow, Result};
use sp_consensus_grandpa::AuthorityList;
use core::time::Duration;
use futures::StreamExt;
use log::{debug, error, info, trace, warn};
use phaxt::ChainApi;
use sp_core::sr25519::Public as Sr25519Public;
use std::sync::Arc;
use tokio::time::sleep;

use crate::bus::Bus;
use crate::datasource::DataSourceManager;
use crate::headers_db::*;
use crate::processor::{PRuntimeRequest, ProcessorEvent};
use crate::pool_operator::DB;
use crate::{use_parachain_api, use_relaychain_api};

use phactory_api::prpc::{Blocks, ChainState, CombinedHeadersToSync, HeadersToSync, ParaHeadersToSync};

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
        if init_run {
            let count = delete_from(&self.headers_db, self.next_number);
            warn!("{} keys was deleted.", count);
        }

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

                trace!("Got #{} from subscription. Desired: {}", block.number(), self.next_number);
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
                        error!("Got error when filling gap. {err}");
                        if init_run {
                            return Err(err);
                        }
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
            trace!("Filling Gap Round. Next: {}, Potential Tip: {}", self.next_number, potential_chaintip);
            if self.next_number > potential_chaintip {
                break Ok(())
            }

            let mut try_count = 0 as usize;
            let headers = loop {
                let headers = pherry::get_headers(&relay_api, self.next_number).await?;
                let last_header = headers.last().unwrap();
                debug!("Got {} headers from node. Last one: #{}", headers.len(), last_header.header.number);
                let justifications = last_header.justification.as_ref().expect("last header from proof api should has justification");
                match &self.current_authorities {
                    Some(authorities) => {
                        let result = pherry::verify_with_prev_authority_set(
                            self.current_set_id,
                            authorities,
                            &last_header.header,
                            justifications,
                        );
                        match result {
                            Ok(_) => break headers,
                            Err(err) => {
                                error!("Fail to verify justification. {}", err);
                            },
                        }
                    }
                    _ => break headers,
                }
                try_count += 1;
                if try_count >= 3 {
                    error!("Tried 3 times but still no valid headers with justification received.");
                    return Err(anyhow!("Tried three times, cannot retrieve a valid justification."))
                }
            };

            let last_header = headers.last().unwrap().header.clone();
            debug!("Putting headers with last #{} into DB.", last_header.number);
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
            // let _ = bus.send_pruntime_request(worker_id, PRuntimeRequest::LoadChainState(request));
        },
        Err(err) => {
            let _ = bus.send_worker_mark_error(worker_id, err.to_string());
        },
    }
}

pub async fn do_request_chain_state(
    bus: Arc<Bus>,
    dsm: Arc<DataSourceManager>,
    number: u32,
) {
    todo!()
}

pub async fn do_request_headers(
    bus: Arc<Bus>,
    headers_db: Arc<DB>,
    next_headernum: u32,
) {
    if let Some(mut headers) = get_current_point(headers_db, next_headernum) {
        headers.retain(|h| h.header.number >= next_headernum);
        if let Some(last_header) = headers.last() {
            // let to = last_header.header.number;
            let _ = bus.send_processor_event(ProcessorEvent::ReceivedHeaders((next_headernum, headers.into())));
        }
    }
    let _ = bus.send_processor_event(ProcessorEvent::ReceivedHeaders((next_headernum, vec![].into())));
}

pub async fn do_request_para_headers(
    bus: Arc<Bus>,
    dsm: Arc<DataSourceManager>,
    para_from: u32,
    relay_num: u32,
) {
    let response = dsm.clone().get_para_header_by_relay_header(relay_num).await;
    if let Err(err) = response {
        let _ = bus.send_processor_event(ProcessorEvent::ReceivedParaHeaders((
            para_from,
            relay_num,
            Err(err)
        )));
        return;
    }

    if let Some((para_to, proof)) = response.unwrap() {
        if para_from <= para_to {
            trace!("Requesting para headers, # {} to {}", para_from, para_to);
            let result = dsm
                .get_para_headers(para_from, para_to)
                .await
                .map(|headers| (Arc::new(headers), Arc::new(proof)));
            let _  = bus.send_processor_event(ProcessorEvent::ReceivedParaHeaders((para_from, relay_num, result)));
        } else {
            todo!()
        }
    } else {
        warn!("Failed to get para headernum at {}", relay_num);
        let _ = bus.send_processor_event(ProcessorEvent::ReceivedParaHeaders((
            para_from,
            relay_num,
            Err(anyhow!("Failed to get para headernum at #{}", relay_num)),
        )));
    }
}

pub async fn do_request_blocks(
    bus: Arc<Bus>,
    dsm: Arc<DataSourceManager>,
    from: u32,
    desired_to: u32,
) {
    let to = std::cmp::min(from + 3, desired_to);
    let result = dsm
        .fetch_storage_changes(from, to)
        .await;
    let _  = bus.send_processor_event(ProcessorEvent::ReceivedBlocks((from, to, result)));
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

    let (para_prev, _) = dsm.get_para_header_by_relay_header(prev_relaychain_finalized_at).await?
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
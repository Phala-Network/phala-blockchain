use crate::datasource::WrappedDataSourceManager;
use crate::db::{get_all_workers, Worker, WrappedDb};
use crate::wm::{
    send_to_main_channel, send_to_main_channel_and_wait_for_response, WorkerManagerCommandTx,
    WorkerManagerMessage, WrappedWorkerManagerContext,
};
use crate::worker::{WorkerContext, WrappedWorkerContext};
use anyhow::Result;
use log::{debug, info};
use phactory_api::blocks::GenesisBlockInfo;
use phactory_api::prpc::InitRuntimeRequest;
use phala_types::AttestationProvider;
use phaxt::subxt::rpc::types as subxt_types;
use pherry::{get_authority_with_proof_at, get_block_at};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::task::JoinSet;
use tokio::time::sleep;

pub struct WorkerLifecycleManager {
    pub main_tx: WorkerManagerCommandTx,
    pub main_ctx: WrappedWorkerManagerContext,
    pub dsm: WrappedDataSourceManager,
    pub should_stop: bool,
    pub inv_db: WrappedDb,
    pub workers: Vec<Worker>,
    pub worker_context_vec: Vec<WrappedWorkerContext>,
    pub worker_context_map: WorkerContextMap,
    pub genesis_info: GenesisBlockInfo,
    pub genesis_state: Vec<(Vec<u8>, Vec<u8>)>,
    pub default_pruntime_init_request: InitRuntimeRequest,
    pub fast_sync_enabled: bool,
}
pub type WrappedWorkerLifecycleManager = Arc<WorkerLifecycleManager>;

pub type WorkerContextMap = HashMap<String, WrappedWorkerContext>; // HashMap<UuidString, WrappedWorkerContext>

impl WorkerLifecycleManager {
    pub async fn create(
        main_tx: WorkerManagerCommandTx,
        main_ctx: WrappedWorkerManagerContext,
        dsm: WrappedDataSourceManager,
        inv_db: WrappedDb,
        fast_sync_enabled: bool,
    ) -> WrappedWorkerLifecycleManager {
        let workers =
            get_all_workers(inv_db.clone()).expect("Failed to load workers from local database");
        let count = workers.len();
        let workers = workers
            .into_iter()
            .filter(|w| w.enabled)
            .collect::<Vec<_>>();
        let count_enabled = workers.len();
        if count_enabled == 0 {
            info!("Got {} worker(s) from local database.", count);
            panic!("There are no worker enabled!");
        }
        debug!(
            "Got workers:\n{}",
            serde_json::to_string_pretty(&workers).unwrap()
        );
        info!(
            "Starting lifecycle for {} of {} worker(s).",
            count_enabled, count
        );

        let mut join_set = JoinSet::new();
        for w in workers.clone() {
            join_set.spawn(WorkerContext::create(w, main_ctx.clone()));
        }
        let mut worker_context_vec = Vec::new();
        let mut worker_context_map: WorkerContextMap = HashMap::new();
        while let Some(c) = join_set.join_next().await {
            match c {
                Ok(c) => match c {
                    Ok(c) => {
                        let cc = Arc::new(RwLock::new(c));
                        let c = cc.clone();
                        let mut c = c.write().await;
                        c.self_ref = Some(cc.clone());
                        worker_context_map.insert(c.id.clone(), cc.clone());
                        worker_context_vec.push(cc.clone());
                        drop(c)
                    }
                    Err(e) => panic!("create_worker_contexts: {}", e),
                },
                Err(e) => panic!("create_worker_contexts: {}", e),
            }
        }

        let dd = dsm.clone();
        dd.clone().wait_until_rpc_avail(false).await;

        info!("Preparing genesis...");
        let relay_api = dd
            .clone()
            .current_relaychain_rpc_client(true)
            .await
            .unwrap()
            .client
            .clone();
        let para_api = dd
            .clone()
            .current_parachain_rpc_client(true)
            .await
            .unwrap()
            .client
            .clone();

        let relaychain_start_block = para_api
            .clone()
            .relay_parent_number()
            .await
            .expect("Relaychain start block not found")
            - 1;
        let genesis_block = get_block_at(&relay_api, Some(relaychain_start_block))
            .await
            .expect("Genesis block not found")
            .0
            .block;
        let genesis_hash = relay_api
            .rpc()
            .block_hash(Some(subxt_types::BlockNumber::from(
                subxt_types::NumberOrHex::Number(relaychain_start_block as u64),
            )))
            .await
            .expect("Genesis hash not found")
            .unwrap();
        let set_proof = get_authority_with_proof_at(&relay_api, genesis_hash)
            .await
            .expect("get_authority_with_proof_at Failed");
        let genesis_info = GenesisBlockInfo {
            block_header: genesis_block.header.clone(),
            authority_set: set_proof.authority_set,
            proof: set_proof.authority_proof,
        };
        let genesis_state = pherry::chain_client::fetch_genesis_storage(&para_api)
            .await
            .expect("fetch_genesis_storage failed");
        let default_pruntime_init_request = InitRuntimeRequest::new(
            false,
            genesis_info.clone(),
            None,
            genesis_state.clone(),
            None,
            true,
            Some(AttestationProvider::Ias),
        );

        let lm = Self {
            main_tx,
            main_ctx,
            dsm,
            should_stop: false,
            inv_db: inv_db.clone(),
            workers,
            worker_context_map,
            worker_context_vec,
            genesis_info,
            genesis_state,
            default_pruntime_init_request,
            fast_sync_enabled,
        };
        Arc::new(lm)
    }

    pub async fn spawn_lifecycle_tasks(self: Arc<Self>) {
        debug!("spawn_lifecycle_tasks start");
        let v = &self.clone().worker_context_map;
        let v = v.iter().map(|(_, c)| c.clone()).collect::<Vec<_>>();
        let mut join_set = JoinSet::new();
        for c in v {
            join_set.spawn(WorkerContext::start(c));
        }
        while let Some(_) = join_set.join_next().await {} // wait tasks to be done
        self.clone()
            .send_to_main_channel(WorkerManagerMessage::ShouldBreakMessageLoop)
            .await
            .expect("spawn_lifecycle_tasks -> ShouldBreakMessageLoop");
    }

    pub async fn acquire_fast_sync_lock(self: Arc<Self>, id: String) {
        debug!("acquire_fast_sync_lock: {}", &id);
        loop {
            let res = self
                .clone()
                .send_to_main_channel_and_wait_for_response(
                    WorkerManagerMessage::AcquireFastSyncLock(id.clone()),
                )
                .await
                .expect("acquire_fast_sync_lock send_to_main_channel_and_wait_for_response failed");
            match res {
                WorkerManagerMessage::ResponseOk => {
                    debug!("Got lock for {}!", &id);
                    break;
                }
                WorkerManagerMessage::ResponseErr(e) => {
                    panic!("Error while acquire_fast_sync_lock: {}", e)
                }
                WorkerManagerMessage::FastSyncLockBusy => {}
                _ => {
                    panic!("Error while acquire_fast_sync_lock: unexcepted message")
                }
            }
            sleep(Duration::from_millis(100)).await;
        }
    }

    pub async fn release_fast_sync_lock(self: Arc<Self>, id: String) {
        let _ = self
            .clone()
            .send_to_main_channel(WorkerManagerMessage::ReleaseFastSyncLock((id)))
            .await;
    }

    pub async fn send_to_main_channel(
        self: Arc<Self>,
        message: WorkerManagerMessage,
    ) -> Result<()> {
        send_to_main_channel(self.main_tx.clone(), message).await
    }

    pub async fn send_to_main_channel_and_wait_for_response(
        self: Arc<Self>,
        message: WorkerManagerMessage,
    ) -> Result<WorkerManagerMessage> {
        send_to_main_channel_and_wait_for_response(self.main_tx.clone(), message).await
    }
}

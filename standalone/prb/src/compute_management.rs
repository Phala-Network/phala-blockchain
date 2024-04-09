use crate::bus::Bus;
use crate::datasource::DataSourceManager;
use crate::processor::*;
use crate::tx::TxManager;
use crate::use_parachain_api;
use crate::worker::WorkerLifecycleState;
use anyhow::{anyhow, Result};
use chrono::Utc;
use futures::future::try_join_all;
use log::{error, info, trace};
use parity_scale_codec::Decode;
use phactory_api::prpc::InitRuntimeResponse;
use phala_pallets::pallet_computation::SessionInfo;
use phala_pallets::registry::WorkerInfoV2;
use sp_core::crypto::AccountId32;
use sp_core::sr25519::Public as Sr25519Public;
use std::fmt;
use std::sync::Arc;

#[derive(Debug, PartialEq)]
pub enum ComputeManagementStage {
    NotStarted,
    Register,
    InitialScore,
    AddToPool,
    StartComputing,
    Completed,
}

impl fmt::Display for ComputeManagementStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct ComputeManagementContext {
    pub stage: ComputeManagementStage,
}

impl WorkerContext {
    pub fn is_compute_management_needed(&self) -> bool {
        !self.is_compute_started() && !self.is_sync_only()
    }

    fn is_compute_started(&self) -> bool {
        matches!(
            self.compute_management_context.as_ref().map(|c| &c.stage),
            Some(ComputeManagementStage::Completed)
        )
    }

    fn determinate_next_stage(
        &mut self,
    ) -> ComputeManagementStage {
        if !self.is_registered() {
            return ComputeManagementStage::Register;
        }

        if self.worker_status.worker.gatekeeper {
            info!("[{}] Worker is in Gatekeeper mode. Completing compute management.", self.uuid);
            return ComputeManagementStage::Completed;
        }

        let initial_score = self.worker_info.as_ref()
            .and_then(|info| info.initial_score);
        if initial_score.is_none() {
            return ComputeManagementStage::InitialScore;
        }

        if self.session_id.is_none() {
            return ComputeManagementStage::AddToPool;
        }

        if !self.is_computing() {
            return ComputeManagementStage::StartComputing;
        }

        return ComputeManagementStage::Completed;
    }
}

impl Processor {
    pub fn request_compute_management(
        &mut self,
        worker: &mut WorkerContext,
    ) {
        if worker.compute_management_context.is_none() {
            worker.compute_management_context = Some(ComputeManagementContext {
                stage: ComputeManagementStage::NotStarted,
            });
        }

        if matches!(&worker.compute_management_context.as_ref().unwrap().stage, ComputeManagementStage::Completed) {
            return;
        }

        if worker.is_sync_only() {
            trace!("[{}] Worker or Pool is sync only mode, skip compute management.", worker.uuid);
            return;
        }

        let next_stage = worker.determinate_next_stage();
        match &next_stage {
            ComputeManagementStage::NotStarted => {
                error!("[{}] NotStarted is not a valid next_stage.", worker.uuid);
                return;
            },
            ComputeManagementStage::Register => {
                self.request_register(worker);
            },
            ComputeManagementStage::InitialScore => {
                trace!("[{}] Waiting for initial_score.", worker.uuid);
            },
            ComputeManagementStage::AddToPool => {
                self.request_add_to_pool(worker);
            },
            ComputeManagementStage::StartComputing => {
                self.request_start_computing(worker);
            },
            ComputeManagementStage::Completed => {
                self.update_worker_state_and_message(
                    worker,
                    WorkerLifecycleState::Working,
                    "Computing Now!",
                    None,
                );
            },
        }

        if &worker.compute_management_context.as_ref().unwrap().stage != &next_stage {
            info!("[{}] Updating ComputeManagementStage from {} to {}",
                worker.uuid, worker.compute_management_context.as_ref().unwrap().stage, next_stage);
        }
        worker.compute_management_context.as_mut().unwrap().stage = next_stage;
    }

    fn request_register(
        &mut self,
        worker: &mut WorkerContext,
    ) {
        if matches!(&worker.compute_management_context.as_ref().unwrap().stage, ComputeManagementStage::Register) {
            trace!("[{}] Register has been requested, waiting for register result", worker.uuid);
            return;
        }

        info!("[{}] Requesting PRuntime InitRuntimeResponse for register", worker.uuid);
        self.add_pruntime_request(
            worker,
            PRuntimeRequest::PrepareRegister((true, worker.operator.clone(), false))
        );
    }

    fn request_add_to_pool(
        &mut self,
        worker: &mut WorkerContext,
    ) {
        if worker.update_session_id_count == 0 {
            trace!("[{}] Never try updating SessionId. Skip continuing compute management.", worker.uuid);
            return;
        }

        if matches!(&worker.compute_management_context.as_ref().unwrap().stage, ComputeManagementStage::AddToPool) {
            trace!("[{}] AddToPool has been requested, waiting for session_id", worker.uuid);
            return;
        }

        info!("[{}] Requesting add to pool", worker.uuid);
        tokio::spawn(do_add_worker_to_pool(
            self.bus.clone(),
            self.txm.clone(),
            worker.uuid.clone(),
            worker.pool_id.clone(),
            worker.public_key().unwrap().clone(),
        ));
    }

    fn request_start_computing(
        &mut self,
        worker: &mut WorkerContext,
    ) {
        if worker.update_session_info_count == 0 {
            trace!("[{}] Never try updating SessionInfo. Skip continuing computation determination.", worker.uuid);
            return;
        }

        if matches!(worker.compute_management_context.as_ref().unwrap().stage, ComputeManagementStage::StartComputing) {
            trace!("[{}] StartComputing has been requested, waiting for session_info", worker.uuid);
            return;
        }

        info!("[{}] Requesting start computing", worker.uuid);
        tokio::spawn(do_start_computing(
            self.bus.clone(),
            self.txm.clone(),
            worker.uuid.clone(),
            worker.pool_id.clone(),
            worker.public_key().unwrap().clone(),
            worker.worker_status.worker.stake.clone()
        ));
    }
}

pub async fn do_register(
    bus: Arc<Bus>,
    txm: Arc<TxManager>,
    worker_id: String,
    pool_id: u64,
    response: InitRuntimeResponse,
    pccs_url: String,
    pccs_timeout_secs: u64,
) {
    let attestation = match response.attestation {
        Some(attestation) => attestation,
        None => {
            error!("[{}] Worker has no attestation.", worker_id);
            return;
        },
    };
    let v2 = attestation.payload.is_none();
    let attestation = pherry::attestation_to_report(
        attestation,
        &pccs_url,
        pccs_timeout_secs,
    )
    .await;
    let attestation = match attestation {
        Ok(attestation) => attestation,
        Err(err) => {
            error!("[{}] Failed to get attestation report. {}", worker_id, err);
            let _ = bus.send_worker_mark_error(
                worker_id,
                err.to_string(),
            );
            return;
        },
    };

    let result = txm.register_worker(pool_id, response.encoded_runtime_info, attestation, v2).await;
    match result {
        Ok(_) => {
            info!("[{}] Worker Register Completed.", worker_id);
            let _ = bus.send_worker_event(
                worker_id.clone(),
                WorkerEvent::UpdateMessage((
                    Utc::now(),
                    "Worker Register Completed.".to_string(),
                ))
            );
            let _ = bus.send_pruntime_request(worker_id.clone(), PRuntimeRequest::RegularGetInfo);
        },
        Err(err) => {
            error!("[{}] Worker Register Failed: {}", worker_id, err);
            let _ = bus.send_worker_mark_error(
                worker_id,
                err.to_string(),
            );
            return;
        },
    }
}

async fn do_add_worker_to_pool(
    bus: Arc<Bus>,
    txm: Arc<TxManager>,
    worker_id: String,
    pool_id: u64,
    worker_public_key: Sr25519Public,
) {
    let result = txm.add_worker(pool_id, worker_public_key).await;
    if let Err(err) = result {
        let err_msg = format!("Failed to add_worker_to_pool. {}", err);
        error!("[{}] {}", worker_id, err_msg);
        let _ = bus.send_worker_mark_error(worker_id, err_msg);
    }
}

async fn do_start_computing(
    bus: Arc<Bus>,
    txm: Arc<TxManager>,
    worker_id: String,
    pool_id: u64,
    worker_public_key: Sr25519Public,
    stake: String,
) {
    let result = txm.start_computing(pool_id, worker_public_key, stake).await;
    if let Err(err) = result {
        let err_msg = format!("Failed to start computing. {}", err);
        error!("[{}] {}", worker_id, err_msg);
        let _ = bus.send_worker_mark_error(worker_id, err_msg);
    }
}

pub async fn do_update_worker_info(
    bus: Arc<Bus>,
    dsm: Arc<DataSourceManager>,
    requests: Vec<(String, Sr25519Public)>,
) -> Result<()> {
    let para_api = use_parachain_api!(dsm, false).ok_or(anyhow!("parachain_api not found"))?;
    let metadata = para_api.metadata();
    let hash = para_api.rpc().finalized_head().await?;
    let storage = para_api.storage().at(hash);

    let futures = requests
        .iter()
        .map(|(_, public_key)| {
            let address = subxt::dynamic::storage(
                "PhalaRegistry",
                "Workers",
                vec![subxt::dynamic::Value::from_bytes(public_key)],
            );
            let storage = storage.clone();
            let lookup_bytes = crate::utils::storage_address_bytes(&address, &metadata).unwrap();
            tokio::spawn(async move {
                storage.fetch_raw(&lookup_bytes).await
            })
        })
        .collect::<Vec<_>>();

    let results = match try_join_all(futures).await {
        Ok(results) => results,
        Err(err) => {
            error!("Met error during update worker info: {}", err);
            return Err(err.into());
        },
    };

    let mut session_ids_requests = Vec::<(String, Sr25519Public)>::new();
    for (idx, (worker_id, public_key)) in requests.iter().enumerate() {
        let result = match results.get(idx) {
            Some(res) => res,
            None => {
                error!("WorkerInfo: #{} result not exists.", idx);
                continue;
            },
        };
        match result {
            Ok(result) => {
                if let Some(bytes) = result {
                    let worker_info = match WorkerInfoV2::<AccountId32>::decode(&mut bytes.as_slice()) {
                        Ok(info) => info,
                        Err(err) => {
                            error!("Failed to decode WorkerInvoV2: {:?}. {:?}", err, bytes);
                            continue;
                        },
                    };
                    let _ = bus.send_worker_event(
                        worker_id.clone(),
                        WorkerEvent::UpdateWorkerInfo(worker_info.clone()),
                    );
                    session_ids_requests.push((worker_id.clone(), public_key.clone()))
                }
            },
            Err(err) => {
                error!("{}", err);
            },
        }
    }

    if session_ids_requests.is_empty() {
        Ok(())
    } else {
        do_update_session_id(bus, dsm, session_ids_requests).await
    }
}

pub async fn do_update_session_id(
    bus: Arc<Bus>,
    dsm: Arc<DataSourceManager>,
    requests: Vec<(String, Sr25519Public)>,
) -> Result<()> {
    let para_api = use_parachain_api!(dsm, false).ok_or(anyhow!("parachain_api not found"))?;
    let metadata = para_api.metadata();
    let hash = para_api.rpc().finalized_head().await?;
    let storage = para_api.storage().at(hash);

    let futures = requests
        .iter()
        .map(|(_, public_key)| {
            let address = subxt::dynamic::storage(
                "PhalaComputation",
                "WorkerBindings",
                vec![subxt::dynamic::Value::from_bytes(public_key)],
            );
            let storage = storage.clone();
            let lookup_bytes = crate::utils::storage_address_bytes(&address, &metadata).unwrap();
            tokio::spawn(async move {
                storage.fetch_raw(&lookup_bytes).await
            })
        })
        .collect::<Vec<_>>();

    let results = match try_join_all(futures).await {
        Ok(results) => results,
        Err(err) => {
            error!("Met error during update session id: {}", err);
            return Err(err.into());
        },
    };

    let mut session_info_requests = Vec::<(String, AccountId32)>::new();
    for (idx, (worker_id, _)) in requests.iter().enumerate() {
        let result = match results.get(idx) {
            Some(res) => res,
            None => {
                error!("SessionId: #{} result not exists.", idx);
                continue;
            },
        };
        match result {
            Ok(result) => {
                let session_id = match result {
                    Some(bytes) => match AccountId32::decode(&mut bytes.as_slice()) {
                        Ok(id) => Some(id),
                        Err(err) => {
                            error!("Failed to decode AccountId32: {:?}. {:?}", err, bytes);
                            continue;
                        },
                    },
                    None => None,
                };
                let _ = bus.send_worker_event(
                    worker_id.clone(),
                    WorkerEvent::UpdateSessionId(session_id.clone()),
                );
                if let Some(session_id) = session_id {
                    session_info_requests.push((worker_id.clone(), session_id))
                }
            },
            Err(err) => {
                error!("{}", err);
            },
        }
    }

    if session_info_requests.is_empty() {
        Ok(())
    } else {
        do_update_session_info(bus, dsm, session_info_requests).await
    }
}

pub async fn do_update_session_info(
    bus: Arc<Bus>,
    dsm: Arc<DataSourceManager>,
    requests: Vec<(String, AccountId32)>,
) -> Result<()> {
    let para_api = use_parachain_api!(dsm, false).ok_or(anyhow!("parachain_api not found"))?;
    let metadata = para_api.metadata();
    let hash = para_api.rpc().finalized_head().await?;
    let storage = para_api.storage().at(hash);

    let futures = requests
        .iter()
        .map(|(_, session_id)| {
            let address = subxt::dynamic::storage(
                "PhalaComputation",
                "Sessions",
                vec![subxt::dynamic::Value::from_bytes(session_id)],
            );
            let storage = storage.clone();
            let lookup_bytes = crate::utils::storage_address_bytes(&address, &metadata).unwrap();
            tokio::spawn(async move {
                storage.fetch_raw(&lookup_bytes).await
            })
        })
        .collect::<Vec<_>>();

    let results = match try_join_all(futures).await {
        Ok(results) => results,
        Err(err) => {
            error!("Met error during update session id: {}", err);
            return Err(err.into());
        },
    };

    for (idx, (worker_id, _)) in requests.iter().enumerate() {
        let result = match results.get(idx) {
            Some(res) => res,
            None => {
                error!("SessionInfo: #{} result not exists.", idx);
                continue;
            },
        };
        match result {
            Ok(result) => {
                let session_info = match result {
                    Some(bytes) => {
                        let session_info = match SessionInfo::decode(&mut bytes.as_slice()) {
                            Ok(id) => id,
                            Err(err) => {
                                error!("Failed to decode SessionInfo: {:?}. {:?}", err, bytes);
                                continue;
                            },
                        };
                        Some(session_info)
                    }
                    _ => None
                };
                let _ = bus.send_worker_event(
                    worker_id.clone(),
                    WorkerEvent::UpdateSessionInfo(session_info),
                );
            },
            Err(err) => {
                error!("{}", err);
            },
        }
    }
    Ok(())
}
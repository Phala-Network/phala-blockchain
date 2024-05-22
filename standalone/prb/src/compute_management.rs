use crate::bus::Bus;
use crate::processor::*;
use crate::tx::TxManager;
use crate::worker::WorkerLifecycleState;
use chrono::Utc;
use log::{error, info, trace};
use phactory_api::prpc::InitRuntimeResponse;
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

        ComputeManagementStage::Completed
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

        if worker.compute_management_context.as_ref().unwrap().stage != next_stage {
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
        if !worker.session_updated {
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
            worker.pool_id,
            worker.public_key().unwrap(),
        ));
    }

    fn request_start_computing(
        &mut self,
        worker: &mut WorkerContext,
    ) {
        if matches!(worker.compute_management_context.as_ref().unwrap().stage, ComputeManagementStage::StartComputing) {
            trace!("[{}] StartComputing has been requested, waiting for session_info", worker.uuid);
            return;
        }

        info!("[{}] Requesting start computing", worker.uuid);
        tokio::spawn(do_start_computing(
            self.bus.clone(),
            self.txm.clone(),
            worker.uuid.clone(),
            worker.pool_id,
            worker.public_key().unwrap(),
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
            let _ = bus.send_worker_mark_error(
                worker_id,
                "Failed to register: worker has no attestation".to_string(),
            );
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
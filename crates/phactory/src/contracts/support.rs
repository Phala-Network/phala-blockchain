use anyhow::{anyhow, bail, Context, Result};
use core::time::Duration;
use pink::types::{AccountId, ExecutionMode, TransactionArguments};
use pink_extension::{chain_extension::JsValue, SidevmConfig};
use serde::{Deserialize, Serialize};
use std::sync::{mpsc, Arc, Mutex};

use parity_scale_codec::Decode;
use phala_mq::SignedMessageChannel;
use phala_scheduler::RequestScheduler;
use runtime::BlockNumber;
use sidevm::{
    service::{Command as SidevmCommand, CommandSender, ExitReason},
    OcallAborted, OutgoingRequestChannel, ShortId, VmId, WasmInstanceConfig, WasmModule,
};

use super::pink::Cluster;
use crate::{
    hex,
    secret_channel::{KeyPair, SecretMessageChannel, SecretReceiver},
    system::{TransactionError, TransactionResult, WorkerIdentityKey},
    types::BlockInfo,
    ChainStorage, H256,
};
use phactory_api::prpc as pb;
use tokio::sync::watch::Receiver as WatchReceiver;
use tracing::{error, info, instrument, Instrument};

pub struct ExecuteEnv<'a, 'b> {
    pub block: &'a mut BlockInfo<'b>,
    pub contract_cluster: &'a mut Cluster,
    pub log_handler: Option<CommandSender>,
}

pub struct TransactionContext<'a, 'b> {
    pub block: &'a mut BlockInfo<'b>,
    pub mq: &'a SignedMessageChannel,
    pub secret_mq: SecretMessageChannel<'a, SignedMessageChannel>,
    pub log_handler: Option<CommandSender>,
}

pub struct QueryContext {
    pub block_number: BlockNumber,
    pub now_ms: u64,
    pub sidevm_handle: Option<SidevmHandle>,
    pub log_handler: Option<CommandSender>,
    pub query_scheduler: RequestScheduler<AccountId>,
    pub weight: u32,
    pub worker_identity_key: WorkerIdentityKey,
    pub chain_storage: ChainStorage,
    pub req_id: u64,
    pub sidevm_event_tx: OutgoingRequestChannel,
}

pub(crate) struct RawData(Vec<u8>);

impl Decode for RawData {
    fn decode<I: parity_scale_codec::Input>(
        input: &mut I,
    ) -> Result<Self, parity_scale_codec::Error> {
        // The remaining_len is not guaranteed to be correct by the trait Input definition. We only
        // decode the RawData with <&[u8] as Input>, which obviously impl the correct remaining_len.
        let mut remaining_len = input
            .remaining_len()?
            .ok_or("Can not decode RawData without length")?;
        let mut decoded = Vec::with_capacity(remaining_len);
        let mut buf = [0u8; 256];
        loop {
            let chunk = remaining_len.min(buf.len());
            input.read(&mut buf[..chunk])?;
            decoded.extend_from_slice(&buf[..chunk]);
            remaining_len -= chunk;
            if remaining_len == 0 {
                break;
            }
        }
        Ok(RawData(decoded))
    }
}

#[derive(Clone)]
pub enum SidevmHandle {
    Running {
        cmd_sender: CommandSender,
        stopped: WatchReceiver<bool>,
    },
    Stopped(ExitReason),
}

impl SidevmHandle {
    pub fn cmd_sender(&self) -> Option<CommandSender> {
        match self {
            SidevmHandle::Running { cmd_sender, .. } => Some(cmd_sender.clone()),
            SidevmHandle::Stopped(_) => None,
        }
    }
}

impl Serialize for SidevmHandle {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            SidevmHandle::Running { .. } => ExitReason::Restore.serialize(serializer),
            SidevmHandle::Stopped(r) => r.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for SidevmHandle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let reason = ExitReason::deserialize(deserializer)?;
        Ok(SidevmHandle::Stopped(reason))
    }
}

#[derive(Serialize, Deserialize, Clone, ::scale_info::TypeInfo)]
pub struct SidevmInfo {
    code: Vec<u8>,
    code_hash: H256,
    start_time: String,
    auto_restart: bool,
    #[codec(skip)]
    handle: Arc<Mutex<SidevmHandle>>,
    #[serde(default)]
    pub config: SidevmConfig,
}

pub(crate) enum SidevmCode {
    Hash(H256),
    Code(Vec<u8>),
}

#[derive(Serialize, Deserialize, Clone, ::scale_info::TypeInfo)]
pub struct Contract {
    send_mq: SignedMessageChannel,
    #[codec(skip)]
    cmd_rcv_mq: SecretReceiver<RawData>,
    #[codec(skip)]
    #[serde(with = "crate::secret_channel::ecdh_serde")]
    ecdh_key: KeyPair,
    cluster_id: phala_mq::ContractClusterId,
    address: AccountId,
    pub sidevm_info: Option<SidevmInfo>,
    weight: u32,
    code_hash: Option<H256>,
    on_block_end: Option<OnBlockEnd>,
}

#[derive(Copy, Clone, Serialize, Deserialize, ::scale_info::TypeInfo)]
struct OnBlockEnd {
    selector: u32,
    gas_limit: u64,
}

impl Contract {
    pub(crate) fn new(
        send_mq: SignedMessageChannel,
        cmd_rcv_mq: SecretReceiver<RawData>,
        ecdh_key: KeyPair,
        cluster_id: phala_mq::ContractClusterId,
        address: AccountId,
        code_hash: Option<H256>,
    ) -> Self {
        Contract {
            send_mq,
            cmd_rcv_mq,
            ecdh_key,
            cluster_id,
            address,
            sidevm_info: None,
            weight: 0,
            code_hash,
            on_block_end: None,
        }
    }

    pub(crate) fn address(&self) -> &AccountId {
        &self.address
    }

    pub(crate) fn sidevm_handle(&self) -> Option<SidevmHandle> {
        self.sidevm_info
            .as_ref()
            .map(|info| info.handle.lock().unwrap().clone())
    }

    pub(crate) fn process_next_message(
        &mut self,
        env: &mut ExecuteEnv,
    ) -> Option<TransactionResult> {
        let secret_mq = SecretMessageChannel::new(&self.ecdh_key, &self.send_mq);
        let mut context = TransactionContext {
            block: env.block,
            mq: &self.send_mq,
            secret_mq,
            log_handler: env.log_handler.clone(),
        };

        phala_mq::select! {
            next_cmd = self.cmd_rcv_mq => match next_cmd {
                Ok((_, cmd, origin)) => {
                    info!("Contract {:?} handling tx call", self.address());
                    let Ok(command) = Decode::decode(&mut &cmd.0[..]) else {
                        error!("Failed to decode tx input");
                        return Some(Err(TransactionError::BadInput));
                    };
                    env.contract_cluster.handle_command(self.address(), origin, command, &mut context)
                }
                Err(_e) => {
                    Err(TransactionError::ChannelError)
                }
            },
        }
    }

    pub(crate) fn on_block_end(&mut self, env: &mut ExecuteEnv) -> TransactionResult {
        let Some(OnBlockEnd {
            selector,
            gas_limit,
        }) = self.on_block_end
        else {
            return Ok(None);
        };

        let input_data = selector.to_be_bytes();
        let tx_args = TransactionArguments {
            origin: self.address.clone(),
            transfer: 0,
            gas_free: false,
            storage_deposit_limit: None,
            gas_limit,
            deposit: 0,
        };
        let mut handle = env.contract_cluster.runtime_mut(env.log_handler.clone());
        _ = handle.call(
            self.address().clone(),
            input_data.to_vec(),
            ExecutionMode::Transaction,
            tx_args,
        );
        Ok(handle.effects)
    }

    pub(crate) fn set_on_block_end_selector(&mut self, selector: u32, gas_limit: u64) {
        self.on_block_end = Some(OnBlockEnd {
            selector,
            gas_limit,
        });
    }

    pub(crate) fn start_sidevm(
        &mut self,
        spawner: &sidevm::service::Spawner,
        code: SidevmCode,
        ensure_waiting_code: bool,
        config: SidevmConfig,
    ) -> Result<()> {
        let handle = self.sidevm_handle();
        let mut prev = None;
        if let Some(SidevmHandle::Running {
            cmd_sender,
            stopped,
        }) = &handle
        {
            if let Err(err) = cmd_sender.try_send(SidevmCommand::Stop) {
                error!("Failed to send stop command to sidevm: {err}");
            }
            prev = Some(stopped.clone());
        }

        let (code, code_hash) = match code {
            SidevmCode::Hash(hash) => (vec![], hash),
            SidevmCode::Code(code) => {
                let actual_hash = sp_core::blake2_256(&code).into();
                if ensure_waiting_code {
                    if !matches!(
                        &handle,
                        Some(SidevmHandle::Stopped(ExitReason::WaitingForCode))
                    ) {
                        bail!("The sidevm isn't waiting for code");
                    }
                    let expected_hash = self
                        .sidevm_info
                        .as_ref()
                        .ok_or_else(|| anyhow!("No sidevm info"))?
                        .code_hash;
                    if actual_hash != expected_hash {
                        bail!(
                            "Code hash mismatch, expected: {expected_hash:?}, actual: {actual_hash:?}"
                        );
                    }
                }
                (code, actual_hash)
            }
        };

        let handle = if code.is_empty() {
            info!("Sidevm code {code_hash:?} not found, waiting to be uploaded");
            Arc::new(Mutex::new(SidevmHandle::Stopped(
                ExitReason::WaitingForCode,
            )))
        } else if code.len() > config.max_code_size as usize {
            warn!(
                "Sidevm code size {} exceeds max_code_size {}",
                code.len(),
                config.max_code_size
            );
            Arc::new(Mutex::new(SidevmHandle::Stopped(ExitReason::CodeTooLarge)))
        } else {
            do_start_sidevm(
                spawner,
                &code,
                *self.address.as_ref(),
                self.weight,
                &config,
                prev,
            )?
        };

        let start_time = chrono::Utc::now().to_rfc3339();
        self.sidevm_info = Some(SidevmInfo {
            code,
            code_hash,
            start_time,
            handle,
            auto_restart: true,
            config,
        });
        Ok(())
    }

    pub(crate) fn restart_sidevm_if_needed(
        &mut self,
        spawner: &sidevm::service::Spawner,
        current_block: BlockNumber,
    ) -> Result<()> {
        if let Some(sidevm_info) = &mut self.sidevm_info {
            let handle = sidevm_info.handle.lock().unwrap().clone();
            if let SidevmHandle::Stopped(reason) = &handle {
                let need_restart = match reason {
                    ExitReason::Exited(_) => false,
                    ExitReason::Stopped => false,
                    ExitReason::InputClosed => false,
                    ExitReason::Panicked => true,
                    ExitReason::Cancelled => false,
                    // TODO.kevin: Allow to charge new gas? How to charge gas or weather the gas
                    // system works or not is not clear ATM.
                    ExitReason::OcallAborted(OcallAborted::GasExhausted) => false,
                    ExitReason::OcallAborted(OcallAborted::Stifled) => true,
                    ExitReason::Restore => true,
                    ExitReason::WaitingForCode => false,
                    ExitReason::CodeTooLarge => false,
                    ExitReason::FailedToStart => false,
                };
                if !need_restart {
                    return Ok(());
                }
                sidevm_info.start_time = chrono::Utc::now().to_rfc3339();
                let handle = do_start_sidevm(
                    spawner,
                    &sidevm_info.code,
                    *self.address.as_ref(),
                    self.weight,
                    &sidevm_info.config,
                    None,
                )?;
                sidevm_info.handle = handle;
            } else {
                if current_block > sidevm_info.config.deadline {
                    let id = sidevm::ShortId(&self.address);
                    info!(target: "sidevm", id=%id, "Sidevm deadline reached, stopping");
                    return self.push_message_to_sidevm(SidevmCommand::Stop);
                }
                return Ok(());
            };
        }
        Ok(())
    }

    pub(crate) fn push_message_to_sidevm(&self, message: SidevmCommand) -> Result<()> {
        let handle = self
            .sidevm_info
            .as_ref()
            .ok_or_else(|| anyhow!("Push message to sidevm failed, no sidevm instance"))?
            .handle
            .clone();

        let vmid = sidevm::ShortId(&self.address);
        let span = tracing::info_span!("sidevm:push", %vmid);
        let _enter = span.enter();

        let tx = match &*handle.lock().unwrap() {
            SidevmHandle::Stopped(_) => {
                error!(target: "sidevm", "PM to sidevm failed, instance terminated");
                return Err(anyhow!(
                    "Push message to sidevm failed, instance terminated"
                ));
            }
            SidevmHandle::Running { cmd_sender: tx, .. } => tx.clone(),
        };
        let result = tx.try_send(message);
        if let Err(err) = result {
            use tokio::sync::mpsc::error::TrySendError;
            match err {
                TrySendError::Full(_) => {
                    error!(target: "sidevm", "PM to sidevm failed (channel full), the guest program may be stucked");
                }
                TrySendError::Closed(_) => {
                    error!(target: "sidevm", "PM to sidevm failed (channel closed), the VM might be already stopped");
                }
            }
        }
        Ok(())
    }

    pub(crate) fn get_system_message_handler(&self) -> Option<CommandSender> {
        let guard = self.sidevm_info.as_ref()?.handle.lock().unwrap();
        match &*guard {
            SidevmHandle::Stopped(_) => None,
            SidevmHandle::Running { cmd_sender: tx, .. } => Some(tx.clone()),
        }
    }

    pub(crate) fn destroy(self, spawner: &sidevm::service::Spawner) {
        if let Some(sidevm_info) = &self.sidevm_info {
            match sidevm_info.handle.lock().unwrap().clone() {
                SidevmHandle::Stopped(_) => {}
                SidevmHandle::Running { cmd_sender: tx, .. } => {
                    spawner.spawn(
                        async move {
                            if let Err(err) = tx.send(SidevmCommand::Stop).await {
                                error!("Failed to send stop command to sidevm: {}", err);
                            }
                        }
                        .in_current_span(),
                    );
                }
            }
        }
    }

    pub fn set_weight(&mut self, weight: u32) {
        self.weight = weight;
        info!(
            "Updated weight for contarct {:?} to {}",
            self.address(),
            weight
        );
        if let Some(SidevmHandle::Running { cmd_sender: tx, .. }) = self.sidevm_handle() {
            if tx.try_send(SidevmCommand::UpdateWeight(weight)).is_err() {
                error!("Failed to update weight for sidevm, maybe it has crashed");
            }
        }
    }
    pub fn weight(&self) -> u32 {
        self.weight
    }

    pub fn info(&self) -> pb::ContractInfo {
        pb::ContractInfo {
            id: hex(&self.address),
            weight: self.weight,
            code_hash: self.code_hash.as_ref().map(hex).unwrap_or_default(),
            sidevm: self.sidevm_info.as_ref().map(|info| {
                let handle = info.handle.lock().unwrap().clone();
                let start_time = info.start_time.clone();
                let code_hash = hex(info.code_hash);
                let max_code_size = info.config.max_code_size;
                let max_memory_pages = info.config.max_memory_pages;
                let vital_capacity = info.config.vital_capacity;
                let deadline = info.config.deadline;
                match handle {
                    SidevmHandle::Running { .. } => pb::SidevmInfo {
                        state: "running".into(),
                        code_hash,
                        start_time,
                        max_memory_pages,
                        vital_capacity,
                        deadline,
                        max_code_size,
                        ..Default::default()
                    },
                    SidevmHandle::Stopped(reason) => pb::SidevmInfo {
                        state: "stopped".into(),
                        code_hash,
                        start_time,
                        stop_reason: format!("{reason}"),
                        max_memory_pages,
                        vital_capacity,
                        deadline,
                        max_code_size,
                    },
                }
            }),
        }
    }
}

#[instrument(name="sidevm", skip_all, fields(id=%sidevm::ShortId(&id)))]
fn do_start_sidevm(
    spawner: &sidevm::service::Spawner,
    code: &[u8],
    id: VmId,
    weight: u32,
    config: &SidevmConfig,
    prev: Option<WatchReceiver<bool>>,
) -> Result<Arc<Mutex<SidevmHandle>>> {
    info!(target: "sidevm", ?config, "Starting sidevm...");
    let (stopped_tx, stopped) = tokio::sync::watch::channel(false);
    let (cmd_sender, join_handle) = spawner.start(
        code,
        config.max_memory_pages,
        id,
        config.vital_capacity,
        local_cache_ops(),
        weight,
        prev,
    )?;
    let handle = Arc::new(Mutex::new(SidevmHandle::Running {
        cmd_sender,
        stopped,
    }));
    let cloned_handle = handle.clone();
    spawner.spawn(
        async move {
            let reason = join_handle.await.unwrap_or(ExitReason::Cancelled);
            error!(target: "sidevm", ?reason, "Sidevm process terminated");
            *cloned_handle.lock().unwrap() = SidevmHandle::Stopped(reason);
            let _ = stopped_tx.send(true);
        }
        .in_current_span(),
    );
    Ok(handle)
}

fn local_cache_ops() -> sidevm::DynCacheOps {
    use ::pink::local_cache as cache;
    type OpResult<T> = Result<T, sidevm::OcallError>;

    struct CacheOps;
    impl sidevm::CacheOps for CacheOps {
        fn get(&self, contract: &[u8], key: &[u8]) -> OpResult<Option<Vec<u8>>> {
            Ok(cache::get(contract, key))
        }

        fn set(&self, contract: &[u8], key: &[u8], value: &[u8]) -> OpResult<()> {
            cache::set(contract, key, value).map_err(|_| sidevm::OcallError::ResourceLimited)
        }

        fn set_expiration(
            &self,
            contract: &[u8],
            key: &[u8],
            expire_after_secs: u64,
        ) -> OpResult<()> {
            cache::set_expiration(contract, key, expire_after_secs);
            Ok(())
        }

        fn remove(&self, contract: &[u8], key: &[u8]) -> OpResult<Option<Vec<u8>>> {
            Ok(cache::remove(contract, key))
        }
    }
    &CacheOps
}

#[instrument(skip_all, fields(id=%ShortId(id)), name = "run")]
pub fn block_on_run_module(
    id: VmId,
    module: &WasmModule,
    args: Vec<String>,
    timeout: Duration,
    sidevm_event_tx: OutgoingRequestChannel,
    log_handler: impl Fn(VmId, u8, String),
) -> Result<JsValue> {
    info!("Run wasm module timeout={}ms", timeout.as_millis());
    enum OutMsg {
        Log(u8, String),
        Output(Vec<u8>),
        Terminated,
        Timeout,
        Error(anyhow::Error),
    }
    let (_tx, cancel_rx) = tokio::sync::oneshot::channel::<()>();
    // Channel to bridge between the sync and async world
    let (output_tx, output_rx) = mpsc::channel();
    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel(1);
    let tx_for_logging = output_tx.clone();
    let config = WasmInstanceConfig {
        max_memory_pages: 256,           // 16MB
        gas_per_breath: 100_000_000_000, // About 1 second tested with md5 calculation
        cache_ops: local_cache_ops(),
        scheduler: None,
        weight: 0,
        id,
        event_tx,
        log_handler: Some(Box::new(move |_vmid, level, msg| {
            if let Err(err) = tx_for_logging.send(OutMsg::Log(level, msg.into())) {
                error!("Failed to send log message to response channel: {}", err);
            }
        })),
    };
    let (mut wasm_run, _env) = module
        .run(args, config)
        .context("Failed to start sidevm instance")?;
    tokio::spawn(
        async move {
            /// Returns true if the sidevm should be terminated
            async fn forward_event(
                vmid: VmId,
                event: sidevm::OutgoingRequest,
                response_tx: &mpsc::Sender<OutMsg>,
                event_tx: &OutgoingRequestChannel,
            ) -> bool {
                match event {
                    event @ sidevm::OutgoingRequest::Query { .. } => {
                        match event_tx.send((vmid, event)).await {
                            Ok(_) => false,
                            Err(err) => {
                                error!("Failed to send query request: {err}");
                                true
                            }
                        }
                    }
                    sidevm::OutgoingRequest::Output(output) => {
                        if let Err(err) = response_tx.send(OutMsg::Output(output)) {
                            error!("Failed to send output to response channel: {}", err);
                        }
                        true
                    }
                }
            }
            tokio::select! {
                _ = tokio::time::sleep(timeout) => {
                    if let Err(err) = output_tx.send(OutMsg::Timeout) {
                        error!("Failed to send timeout message to response channel: {}", err);
                    }
                }
                _ = cancel_rx => {
                    warn!("The host thread returned while the sidevm task is running")
                }
                rv = &mut wasm_run => {
                    if let Err(err) = rv {
                        error!(target: "sidevm", ?err, "Js runtime exited with error.");
                        if let Err(err) = output_tx.send(OutMsg::Error(err.into())) {
                            error!("Failed to send error message to response channel: {}", err);
                        }
                    }
                }
                _ = async {
                    while let Some((vmid, event)) = event_rx.recv().await {
                        if forward_event(vmid, event, &output_tx, &sidevm_event_tx).await {
                            break;
                        }
                    }
                } => {}
            }
            // Continue to process events incase there are some pending events in the channel
            while let Ok((vmid, event)) = event_rx.try_recv() {
                if forward_event(vmid, event, &output_tx, &sidevm_event_tx).await {
                    break;
                }
            }
            if output_tx.send(OutMsg::Terminated).is_err() {
                error!("Failed to send termination message to response channel");
            }
        }
        .in_current_span(),
    );
    loop {
        match output_rx.recv() {
            Ok(OutMsg::Log(level, msg)) => {
                log_handler(id, level, msg);
            }
            Ok(OutMsg::Output(o)) => {
                let value =
                    JsValue::decode(&mut &o[..]).map_err(|_| anyhow!("Failed to decode output"))?;
                return Ok(value);
            }
            Ok(OutMsg::Terminated) => {
                return Err(anyhow!("Sidevm terminated without output"));
            }
            Ok(OutMsg::Timeout) => {
                return Err(anyhow!("Sidevm execution timeout"));
            }
            Ok(OutMsg::Error(err)) => {
                return Err(err);
            }
            Err(err) => {
                return Err(anyhow!("Failed to receive response from sidevm: {err}"));
            }
        }
    }
}

pub use keeper::*;
mod keeper;

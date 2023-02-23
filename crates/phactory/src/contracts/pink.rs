use std::time::Duration;

use crate::{
    contracts::{self, QueryContext, TransactionContext},
    system::{TransactionError, TransactionResult},
};
use anyhow::Result;
use parity_scale_codec::{Decode, Encode};
use phala_crypto::sr25519::Persistence;
use phala_mq::{ContractClusterId, MessageOrigin};
use phala_types::contract::messaging::ResourceType;
use pink::{
    capi::v1::{
        ecall::{ECalls, ECallsRo},
        ocall::{ExecContext, OCalls},
    },
    local_cache::{self, StorageQuotaExceeded},
    runtimes::v1::using_ocalls,
    types::ExecutionMode,
};
use serde::{Deserialize, Serialize};
use sidevm::service::{Command as SidevmCommand, CommandSender, SystemMessage};
use sp_core::sr25519;

use ::pink::{
    capi::v1,
    constants::WEIGHT_REF_TIME_PER_SECOND,
    types::{AccountId, Balance, BlockNumber, ExecSideEffects, Hash, TransactionArguments},
};

pub use phala_types::contract::InkCommand;

#[derive(Debug, Encode, Decode)]
pub enum Query {
    InkMessage {
        payload: Vec<u8>,
        /// Amount of tokens deposit to the caller.
        deposit: u128,
        /// Amount of tokens transfer from the caller to the target contract.
        transfer: u128,
        /// Whether to use the gas estimation mode.
        estimating: bool,
    },
    SidevmQuery(Vec<u8>),
    InkInstantiate {
        code_hash: sp_core::H256,
        salt: Vec<u8>,
        instantiate_data: Vec<u8>,
        /// Amount of tokens deposit to the caller.
        deposit: u128,
        /// Amount of tokens transfer from the caller to the target contract.
        transfer: u128,
    },
}

#[derive(Debug, Encode, Decode)]
pub enum Response {
    Payload(Vec<u8>),
}

#[derive(Debug, Encode, Decode)]
pub enum QueryError {
    BadOrigin,
    RuntimeError(String),
    SidevmNotFound,
    NoResponse,
    ServiceUnavailable,
    Timeout,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct ClusterConfig {
    pub log_handler: Option<AccountId>,
    pub runtime_version: (u32, u32),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Cluster {
    pub id: ContractClusterId,
    pub config: ClusterConfig,
    pub storage: pink::storage::ClusterStorage,
}

pub struct RuntimeHandleMut<'a> {
    cluster: &'a mut Cluster,
    logger: Option<CommandSender>,
    block_number: BlockNumber,
    pub(crate) effects: Option<ExecSideEffects>,
}

impl RuntimeHandleMut<'_> {
    fn readonly(&self) -> RuntimeHandle {
        RuntimeHandle {
            cluster: self.cluster,
            logger: self.logger.clone(),
            block_number: self.block_number,
        }
    }
    fn execute_mut<T>(&mut self, f: impl FnOnce() -> T) -> T {
        using_ocalls(self, f)
    }
}

pub struct RuntimeHandle<'a> {
    cluster: &'a Cluster,
    logger: Option<CommandSender>,
    block_number: BlockNumber,
}

impl RuntimeHandle<'_> {
    fn dup(&self) -> RuntimeHandle {
        RuntimeHandle {
            cluster: self.cluster,
            logger: self.logger.clone(),
            block_number: self.block_number,
        }
    }
    fn execute<T>(&self, f: impl FnOnce() -> T) -> T {
        using_ocalls(&mut self.dup(), f)
    }
    fn ensure_version(&self, version: (u32, u32)) {
        if self.cluster.config.runtime_version != version {
            panic!(
                "Cross call {version:?} is not supported in runtime version {:?}",
                self.cluster.config.runtime_version
            );
        }
    }
}

pub(crate) mod context {
    use pink::capi::v1::ocall::ExecContext;

    environmental::environmental!(exec_context: ExecContext);

    pub fn get() -> ExecContext {
        exec_context::with(|ctx| ctx.clone()).unwrap_or_default()
    }

    pub fn using<T>(mut ctx: ExecContext, f: impl FnOnce() -> T) -> T {
        exec_context::using(&mut ctx, f)
    }
}

impl OCalls for RuntimeHandle<'_> {
    fn storage_root(&self) -> Option<Hash> {
        self.cluster.storage.root()
    }

    fn storage_get(&self, key: Vec<u8>) -> Option<Vec<u8>> {
        self.cluster.storage.get(&key).map(|(_rc, val)| val.clone())
    }

    fn storage_commit(&mut self, _root: Hash, _changes: Vec<(Vec<u8>, (Vec<u8>, i32))>) {
        panic!("storage_commit called on readonly cluster");
    }

    fn log_to_server(&self, contract: AccountId, level: u8, message: String) {
        let Some(log_handler) = self.logger.as_ref() else {
            return;
        };
        let context = self.exec_context();
        let msg = SidevmCommand::PushSystemMessage(SystemMessage::PinkLog {
            block_number: self.block_number,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as _,
            exec_mode: context.mode.display().into(),
            contract: contract.clone().into(),
            level,
            message,
        });
        if log_handler.try_send(msg).is_err() {
            error!("Pink send log to server failed");
        }
    }

    fn emit_side_effects(&mut self, _effects: ExecSideEffects) {}

    fn exec_context(&self) -> ExecContext {
        context::get()
    }

    fn worker_pubkey(&self) -> [u8; 32] {
        context::get().worker_pubkey
    }

    fn cache_get(&self, contract: Vec<u8>, key: Vec<u8>) -> Option<Vec<u8>> {
        if !context::get().mode.is_query() {
            return None;
        }
        local_cache::get(&contract, &key)
    }

    fn cache_set(
        &self,
        contract: Vec<u8>,
        key: Vec<u8>,
        value: Vec<u8>,
    ) -> Result<(), StorageQuotaExceeded> {
        if context::get().mode.is_estimating() {
            return Ok(());
        }
        local_cache::set(&contract, &key, &value)
    }

    fn cache_set_expiration(&self, contract: Vec<u8>, key: Vec<u8>, expiration: u64) {
        if context::get().mode.is_estimating() {
            return;
        }
        local_cache::set_expiration(&contract, &key, expiration)
    }

    fn cache_remove(&self, contract: Vec<u8>, key: Vec<u8>) -> Option<Vec<u8>> {
        if context::get().mode.is_estimating() {
            return None;
        }
        local_cache::remove(&contract, &key)
    }
}

impl OCalls for RuntimeHandleMut<'_> {
    fn storage_root(&self) -> Option<Hash> {
        self.readonly().storage_root()
    }

    fn storage_get(&self, key: Vec<u8>) -> Option<Vec<u8>> {
        self.readonly().storage_get(key)
    }

    fn storage_commit(&mut self, root: Hash, changes: Vec<(Vec<u8>, (Vec<u8>, i32))>) {
        for (key, (value, rc)) in changes {
            self.cluster.storage.set(key, value, rc);
        }
        self.cluster.storage.set_root(root);
    }

    fn log_to_server(&self, contract: AccountId, level: u8, message: String) {
        self.readonly().log_to_server(contract, level, message)
    }

    fn emit_side_effects(&mut self, effects: ExecSideEffects) {
        self.effects = Some(effects);
    }

    fn exec_context(&self) -> ExecContext {
        self.readonly().exec_context()
    }

    fn worker_pubkey(&self) -> [u8; 32] {
        context::get().worker_pubkey
    }

    fn cache_get(&self, contract: Vec<u8>, key: Vec<u8>) -> Option<Vec<u8>> {
        self.readonly().cache_get(contract, key)
    }

    fn cache_set(
        &self,
        contract: Vec<u8>,
        key: Vec<u8>,
        value: Vec<u8>,
    ) -> Result<(), StorageQuotaExceeded> {
        self.readonly().cache_set(contract, key, value)
    }

    fn cache_set_expiration(&self, contract: Vec<u8>, key: Vec<u8>, expiration: u64) {
        self.readonly()
            .cache_set_expiration(contract, key, expiration)
    }

    fn cache_remove(&self, contract: Vec<u8>, key: Vec<u8>) -> Option<Vec<u8>> {
        self.readonly().cache_remove(contract, key)
    }
}

impl v1::CrossCall for RuntimeHandle<'_> {
    fn cross_call(&self, call_id: u32, data: &[u8]) -> Vec<u8> {
        self.ensure_version((1, 0));
        self.execute(move || ::pink::runtimes::v1::RUNTIME.ecall(call_id, data))
    }
}

impl v1::CrossCall for RuntimeHandleMut<'_> {
    fn cross_call(&self, call_id: u32, data: &[u8]) -> Vec<u8> {
        self.readonly().ensure_version((1, 0));
        self.readonly()
            .execute(move || ::pink::runtimes::v1::RUNTIME.ecall(call_id, data))
    }
}

impl v1::CrossCallMut for RuntimeHandleMut<'_> {
    fn cross_call_mut(&mut self, call_id: u32, data: &[u8]) -> Vec<u8> {
        self.readonly().ensure_version((1, 0));
        self.execute_mut(move || ::pink::runtimes::v1::RUNTIME.ecall(call_id, data))
    }
}

impl v1::ECall for RuntimeHandle<'_> {}
impl v1::ECall for RuntimeHandleMut<'_> {}

impl Cluster {
    pub fn new(
        id: &ContractClusterId,
        cluster_key: &sr25519::Pair,
        runtime_version: (u32, u32),
    ) -> Self {
        let mut cluster = Cluster {
            id: *id,
            storage: Default::default(),
            config: ClusterConfig {
                runtime_version,
                ..Default::default()
            },
        };
        let mut runtime = cluster.default_runtime_mut();
        runtime.set_key(cluster_key.dump_secret_key());
        cluster
    }

    pub fn default_runtime(&self) -> RuntimeHandle {
        self.runtime(None, 0)
    }

    pub fn default_runtime_mut(&mut self) -> RuntimeHandleMut {
        self.runtime_mut(None, 0)
    }

    pub fn runtime(
        &self,
        logger: Option<CommandSender>,
        block_number: BlockNumber,
    ) -> RuntimeHandle {
        RuntimeHandle {
            cluster: self,
            logger,
            block_number,
        }
    }

    pub fn runtime_mut(
        &mut self,
        logger: Option<CommandSender>,
        block_number: BlockNumber,
    ) -> RuntimeHandleMut {
        RuntimeHandleMut {
            cluster: self,
            logger,
            block_number,
            effects: None,
        }
    }

    pub fn key(&self) -> sr25519::Pair {
        let raw_key = self
            .default_runtime()
            .get_key()
            .expect("cluster key not set");
        sr25519::Pair::restore_from_secret_key(&raw_key)
    }

    pub fn system_contract(&self) -> Option<AccountId> {
        self.default_runtime().system_contract()
    }

    pub fn code_hash(&self, address: &AccountId) -> Option<Hash> {
        self.default_runtime().code_hash(address.clone())
    }

    pub fn upload_resource(
        &mut self,
        origin: &AccountId,
        resource_type: ResourceType,
        resource_data: Vec<u8>,
    ) -> Result<Hash, String> {
        match resource_type {
            ResourceType::InkCode => {
                self.default_runtime_mut()
                    .upload_code(origin.clone(), resource_data, true)
            }
            ResourceType::SidevmCode => self
                .default_runtime_mut()
                .upload_sidevm_code(origin.clone(), resource_data),
            ResourceType::IndeterministicInkCode => {
                self.default_runtime_mut()
                    .upload_code(origin.clone(), resource_data, false)
            }
        }
    }

    pub fn get_resource(&self, resource_type: ResourceType, hash: &Hash) -> Option<Vec<u8>> {
        match resource_type {
            ResourceType::InkCode => None,
            ResourceType::SidevmCode => self.default_runtime().get_sidevm_code(*hash),
            ResourceType::IndeterministicInkCode => None,
        }
    }

    pub fn deposit(&mut self, who: &::pink::types::AccountId, amount: Balance) {
        self.default_runtime_mut().deposit(who.clone(), amount)
    }

    pub(crate) async fn handle_query(
        &mut self,
        contract_id: &AccountId,
        origin: Option<&AccountId>,
        req: Query,
        context: &mut QueryContext,
    ) -> Result<(Response, Option<ExecSideEffects>), QueryError> {
        match req {
            Query::InkMessage {
                payload: input_data,
                deposit,
                transfer,
                estimating,
            } => {
                let _guard = context
                    .query_scheduler
                    .acquire(contract_id.clone(), context.weight)
                    .await
                    .or(Err(QueryError::ServiceUnavailable))?;

                let mode = if estimating {
                    ExecutionMode::Estimating
                } else {
                    ExecutionMode::Query
                };
                let ctx = ExecContext::new(
                    mode,
                    context.block_number,
                    context.now_ms,
                    context.worker_pubkey,
                );
                context::using(ctx, move || {
                    let origin = origin.cloned().ok_or(QueryError::BadOrigin)?;
                    let mut runtime =
                        self.runtime_mut(context.log_handler.clone(), context.block_number);
                    if deposit > 0 {
                        runtime.deposit(origin.clone(), deposit);
                    }
                    let args = TransactionArguments {
                        origin,
                        transfer,
                        gas_limit: WEIGHT_REF_TIME_PER_SECOND * 10,
                        gas_free: true,
                        storage_deposit_limit: None,
                    };
                    let ink_result =
                        runtime.contract_call(contract_id.clone(), input_data, mode, args);
                    let effects = if mode.is_estimating() {
                        None
                    } else {
                        runtime.effects.take().map(|e| e.into_query_only_effects())
                    };
                    Ok((Response::Payload(ink_result), effects))
                })
            }
            Query::SidevmQuery(payload) => {
                let handle = context
                    .sidevm_handle
                    .as_ref()
                    .ok_or(QueryError::SidevmNotFound)?;
                let cmd_sender = match handle {
                    contracts::SidevmHandle::Stopped(_) => return Err(QueryError::SidevmNotFound),
                    contracts::SidevmHandle::Running(sender) => sender,
                };
                let origin = origin.cloned().map(Into::into);

                let (reply_tx, rx) = tokio::sync::oneshot::channel();

                const SIDEVM_QUERY_TIMEOUT: Duration = Duration::from_secs(60 * 5);

                tokio::time::timeout(SIDEVM_QUERY_TIMEOUT, async move {
                    cmd_sender
                        .send(SidevmCommand::PushQuery {
                            origin,
                            payload,
                            reply_tx,
                        })
                        .await
                        .or(Err(QueryError::ServiceUnavailable))?;
                    rx.await
                        .or(Err(QueryError::NoResponse))
                        .map(Response::Payload)
                })
                .await
                .or(Err(QueryError::Timeout))?
                .map(|response| (response, None))
            }
            Query::InkInstantiate {
                code_hash,
                salt,
                instantiate_data,
                deposit,
                transfer,
            } => {
                let _guard = context
                    .query_scheduler
                    .acquire(contract_id.clone(), context.weight)
                    .await
                    .or(Err(QueryError::ServiceUnavailable))?;

                let origin = origin.cloned().ok_or(QueryError::BadOrigin)?;
                let ctx = ExecContext::new(
                    ExecutionMode::Estimating,
                    context.block_number,
                    context.now_ms,
                    context.worker_pubkey,
                );
                context::using(ctx, move || {
                    let mut runtime =
                        self.runtime_mut(context.log_handler.clone(), context.block_number);
                    if deposit > 0 {
                        runtime.deposit(origin.clone(), deposit);
                    }
                    let args = TransactionArguments {
                        origin,
                        transfer,
                        gas_limit: WEIGHT_REF_TIME_PER_SECOND * 10,
                        gas_free: true,
                        storage_deposit_limit: None,
                    };
                    let ink_result = runtime.contract_instantiate(
                        code_hash,
                        instantiate_data,
                        salt,
                        ExecutionMode::Estimating,
                        args,
                    );
                    Ok((Response::Payload(ink_result), None))
                })
            }
        }
    }

    pub(crate) fn handle_command(
        &mut self,
        contract_id: &AccountId,
        origin: MessageOrigin,
        cmd: InkCommand,
        context: &mut TransactionContext,
    ) -> TransactionResult {
        match cmd {
            InkCommand::InkMessage {
                nonce,
                message,
                transfer,
                gas_limit,
                storage_deposit_limit,
            } => {
                let mut gas_free = false;
                let origin: runtime::AccountId = match origin {
                    MessageOrigin::AccountId(origin) => origin.0.into(),
                    MessageOrigin::Pallet(_) => {
                        // The caller will be set to the system contract if it's from a pallet call
                        // and without charging for gas
                        gas_free = true;
                        self.system_contract()
                            .expect("BUG: system contract missing")
                    }
                    _ => return Err(TransactionError::BadOrigin),
                };

                let args = TransactionArguments {
                    origin: origin.clone(),
                    transfer,
                    gas_limit,
                    gas_free,
                    storage_deposit_limit,
                };

                let mut runtime =
                    self.runtime_mut(context.log_handler.clone(), context.block.block_number);
                let output = runtime.contract_call(
                    contract_id.clone(),
                    message,
                    ExecutionMode::Transaction,
                    args,
                );

                if let Some(log_handler) = &context.log_handler {
                    let msg = SidevmCommand::PushSystemMessage(SystemMessage::PinkMessageOutput {
                        origin: origin.into(),
                        contract: *contract_id.as_ref(),
                        block_number: context.block.block_number,
                        nonce: nonce.into_inner(),
                        output,
                    });
                    if log_handler.try_send(msg).is_err() {
                        error!("Pink emit message output to log handler failed");
                    }
                }
                Ok(runtime.effects)
            }
        }
    }

    pub(crate) fn snapshot(&self) -> Self {
        self.clone()
    }
}

pub trait ClusterContainer {
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn len(&self) -> usize;
    fn get_cluster(&self, cluster_id: &ContractClusterId) -> Option<&Cluster>;
    fn get_cluster_mut(&mut self, cluster_id: &ContractClusterId) -> Option<&mut Cluster>;
    fn remove_cluster(&mut self, cluster_id: &ContractClusterId) -> Option<Cluster>;
}

impl ClusterContainer for Option<Cluster> {
    fn len(&self) -> usize {
        if self.is_some() {
            1
        } else {
            0
        }
    }
    fn get_cluster(&self, cluster_id: &ContractClusterId) -> Option<&Cluster> {
        self.as_ref().filter(|c| c.id == *cluster_id)
    }
    fn get_cluster_mut(&mut self, cluster_id: &ContractClusterId) -> Option<&mut Cluster> {
        self.as_mut().filter(|c| c.id == *cluster_id)
    }
    fn remove_cluster(&mut self, cluster_id: &ContractClusterId) -> Option<Cluster> {
        _ = self.get_cluster(cluster_id)?;
        self.take()
    }
}

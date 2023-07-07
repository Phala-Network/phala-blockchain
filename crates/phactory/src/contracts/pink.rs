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
        ocall::{ExecContext, HttpRequest, HttpRequestError, HttpResponse, OCalls, StorageChanges},
    },
    local_cache::{self, StorageQuotaExceeded},
    runtimes::v1::using_ocalls,
    types::ExecutionMode,
};
use serde::{Deserialize, Serialize};
use sidevm::service::{Command as SidevmCommand, CommandSender, Metric, SystemMessage};
use sp_core::{blake2_256, sr25519, twox_64};

use ::pink::{
    capi::v1,
    constants::WEIGHT_REF_TIME_PER_SECOND,
    types::{AccountId, Balance, ExecSideEffects, Hash, TransactionArguments},
};

pub use phala_types::contract::InkCommand;

pub(crate) mod http_counters;

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
    pub secret_salt: [u8; 32],
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
    pub(crate) effects: Option<ExecSideEffects>,
}

impl RuntimeHandleMut<'_> {
    fn readonly(&self) -> RuntimeHandle {
        RuntimeHandle {
            cluster: self.cluster,
            logger: self.logger.clone(),
        }
    }
    fn execute_mut<T>(&mut self, f: impl FnOnce() -> T) -> T {
        using_ocalls(self, f)
    }

    pub fn call(
        &mut self,
        contract: AccountId,
        input_data: Vec<u8>,
        mode: ExecutionMode,
        tx_args: TransactionArguments,
    ) -> Vec<u8> {
        context::using_entry_contract(contract.clone(), move || {
            self.contract_call(contract, input_data, mode, tx_args)
        })
    }

    pub fn instantiate(
        &mut self,
        code_hash: Hash,
        salt: Vec<u8>,
        instantiate_data: Vec<u8>,
        mode: ExecutionMode,
        tx_args: TransactionArguments,
    ) -> Vec<u8> {
        let buf = phala_types::contract::contract_id_preimage(
            tx_args.origin.as_ref(),
            code_hash.as_ref(),
            self.readonly().cluster_id().as_ref(),
            &salt,
        );
        let entry = AccountId::from(blake2_256(&buf));
        context::using_entry_contract(entry, move || {
            self.contract_instantiate(code_hash, salt, instantiate_data, mode, tx_args)
        })
    }
}

pub struct RuntimeHandle<'a> {
    cluster: &'a Cluster,
    logger: Option<CommandSender>,
}

impl RuntimeHandle<'_> {
    fn dup(&self) -> RuntimeHandle {
        RuntimeHandle {
            cluster: self.cluster,
            logger: self.logger.clone(),
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
    use std::time::{Duration, Instant};

    use pink::{
        capi::v1::ocall::ExecContext,
        types::{AccountId, BlockNumber, ExecutionMode},
    };

    use crate::ChainStorage;

    environmental::environmental!(exec_context: trait GetContext);

    pub trait GetContext {
        fn chain_storage(&self) -> &ChainStorage;
        fn exec_context(&self) -> ExecContext;
        fn worker_pubkey(&self) -> [u8; 32];
        fn call_elapsed(&self) -> Duration;
    }

    pub struct ContractExecContext {
        pub mode: ExecutionMode,
        pub now_ms: u64,
        pub block_number: BlockNumber,
        pub worker_pubkey: [u8; 32],
        pub chain_storage: ChainStorage,
        pub start_at: Instant,
        pub req_id: u64,
    }

    impl ContractExecContext {
        pub fn new(
            mode: ExecutionMode,
            now_ms: u64,
            block_number: BlockNumber,
            worker_pubkey: [u8; 32],
            chain_storage: ChainStorage,
            req_id: u64,
        ) -> Self {
            Self {
                mode,
                now_ms,
                block_number,
                worker_pubkey,
                chain_storage,
                start_at: Instant::now(),
                req_id,
            }
        }
    }

    impl GetContext for ContractExecContext {
        fn chain_storage(&self) -> &ChainStorage {
            &self.chain_storage
        }

        fn exec_context(&self) -> ExecContext {
            ExecContext {
                mode: self.mode,
                block_number: self.block_number,
                now_ms: self.now_ms,
                req_id: Some(self.req_id),
            }
        }

        fn worker_pubkey(&self) -> [u8; 32] {
            self.worker_pubkey
        }

        fn call_elapsed(&self) -> Duration {
            self.start_at.elapsed()
        }
    }

    pub fn get() -> ExecContext {
        exec_context::with(|ctx| ctx.exec_context()).unwrap_or_default()
    }

    pub fn using<T>(ctx: &mut impl GetContext, f: impl FnOnce() -> T) -> T {
        exec_context::using(ctx, f)
    }

    pub fn with<T>(f: impl FnOnce(&dyn GetContext) -> T) -> T {
        exec_context::with(|ctx| f(ctx)).expect("exec_context not set")
    }

    pub fn worker_pubkey() -> [u8; 32] {
        exec_context::with(|ctx| ctx.worker_pubkey()).unwrap_or_default()
    }

    pub fn call_elapsed() -> Duration {
        exec_context::with(|ctx| ctx.call_elapsed()).unwrap_or_else(|| Duration::from_secs(0))
    }

    pub fn time_remaining() -> u64 {
        const MAX_QUERY_TIME: Duration = Duration::from_secs(10);
        MAX_QUERY_TIME.saturating_sub(call_elapsed()).as_millis() as _
    }

    pub use entry::{get_entry_contract, using_entry_contract};
    mod entry {
        use super::*;

        environmental::environmental!(entry_contract: AccountId);

        pub fn get_entry_contract() -> Option<AccountId> {
            entry_contract::with(|a| a.clone())
        }

        pub fn using_entry_contract<T>(mut contract: AccountId, f: impl FnOnce() -> T) -> T {
            entry_contract::using(&mut contract, f)
        }
    }
}

impl OCalls for RuntimeHandle<'_> {
    fn storage_root(&self) -> Option<Hash> {
        self.cluster.storage.root()
    }

    fn storage_get(&self, key: Vec<u8>) -> Option<Vec<u8>> {
        self.cluster.storage.get(&key).map(|(_rc, val)| val.clone())
    }

    fn storage_commit(&mut self, _root: Hash, _changes: StorageChanges) {
        panic!("storage_commit called on readonly cluster");
    }

    fn log_to_server(&self, contract: AccountId, level: u8, message: String) {
        let Some(log_handler) = self.logger.as_ref() else {
            return;
        };
        let Some(entry) = context::get_entry_contract() else {
            error!("Pink log called outside of contract execution");
            return;
        };
        let context = self.exec_context();
        let msg = SidevmCommand::PushSystemMessage(SystemMessage::PinkLog {
            block_number: context.block_number,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as _,
            exec_mode: context.mode.display().into(),
            contract: contract.into(),
            entry: entry.into(),
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
        context::worker_pubkey()
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

    fn latest_system_code(&self) -> Vec<u8> {
        context::with(|ctx| ctx.chain_storage().pink_system_code().1)
    }

    fn http_request(
        &self,
        contract: AccountId,
        request: HttpRequest,
    ) -> Result<HttpResponse, HttpRequestError> {
        let result = pink_extension_runtime::http_request(request, context::time_remaining());
        match &result {
            Ok(response) => {
                http_counters::add(contract, response.status_code);
            }
            Err(_) => {
                http_counters::add(contract, 0);
            }
        }
        result
    }
}

impl OCalls for RuntimeHandleMut<'_> {
    fn storage_root(&self) -> Option<Hash> {
        self.readonly().storage_root()
    }

    fn storage_get(&self, key: Vec<u8>) -> Option<Vec<u8>> {
        self.readonly().storage_get(key)
    }

    fn storage_commit(&mut self, root: Hash, changes: StorageChanges) {
        self.cluster.storage.commit(root, changes);
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
        self.readonly().worker_pubkey()
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

    fn latest_system_code(&self) -> Vec<u8> {
        self.readonly().latest_system_code()
    }

    fn http_request(
        &self,
        contract: AccountId,
        request: HttpRequest,
    ) -> Result<HttpResponse, HttpRequestError> {
        self.readonly().http_request(contract, request)
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
        let secret_key = cluster_key.dump_secret_key();
        let secret_salt = blake2_256(&secret_key);
        let mut cluster = Cluster {
            id: *id,
            storage: Default::default(),
            config: ClusterConfig {
                runtime_version,
                secret_salt,
                ..Default::default()
            },
        };
        let mut runtime = cluster.default_runtime_mut();
        runtime.set_key(secret_key);
        cluster
    }

    pub fn default_runtime(&self) -> RuntimeHandle {
        self.runtime(None)
    }

    pub fn default_runtime_mut(&mut self) -> RuntimeHandleMut {
        self.runtime_mut(None)
    }

    pub fn runtime(&self, logger: Option<CommandSender>) -> RuntimeHandle {
        RuntimeHandle {
            cluster: self,
            logger,
        }
    }

    pub fn runtime_mut(&mut self, logger: Option<CommandSender>) -> RuntimeHandleMut {
        RuntimeHandleMut {
            cluster: self,
            logger,
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
        mut self,
        contract_id: &AccountId,
        origin: Option<&AccountId>,
        req: Query,
        context: QueryContext,
    ) -> Result<(Response, Option<ExecSideEffects>), QueryError> {
        match req {
            Query::InkMessage {
                payload: input_data,
                deposit,
                transfer,
                estimating,
            } => {
                let origin = origin.cloned().ok_or(QueryError::BadOrigin)?;
                let _guard = context
                    .query_scheduler
                    .acquire(contract_id.clone(), context.weight)
                    .await
                    .or(Err(QueryError::ServiceUnavailable))?;

                if let Some(logger) = &context.log_handler {
                    let fp = twox_64(&(&origin, &self.config.secret_salt).encode());
                    if let Err(_err) = logger.try_send(SidevmCommand::PushSystemMessage(
                        SystemMessage::Metric(Metric::PinkQueryIn(fp)),
                    )) {
                        error!("Failed to send metric to log_server");
                    }
                }

                let mode = if estimating {
                    ExecutionMode::Estimating
                } else {
                    ExecutionMode::Query
                };
                let mut ctx = context::ContractExecContext::new(
                    mode,
                    context.now_ms,
                    context.block_number,
                    context.worker_pubkey,
                    context.chain_storage,
                    context.req_id,
                );
                let log_handler = context.log_handler.clone();
                let span = tracing::Span::current();
                let contract_id = contract_id.clone();
                tokio::task::spawn_blocking(move || {
                    let _guard = span.enter();
                    context::using(&mut ctx, move || {
                        context::using_entry_contract(contract_id.clone(), || {
                            let mut runtime = self.runtime_mut(log_handler);
                            let args = TransactionArguments {
                                origin,
                                transfer,
                                gas_limit: WEIGHT_REF_TIME_PER_SECOND * 10,
                                gas_free: true,
                                storage_deposit_limit: None,
                                deposit,
                            };
                            let ink_result = runtime.call(contract_id, input_data, mode, args);
                            let effects = if mode.is_estimating() {
                                None
                            } else {
                                runtime.effects.take().map(|e| e.into_query_only_effects())
                            };
                            Ok((Response::Payload(ink_result), effects))
                        })
                    })
                })
                .await
                .map_err(|_| QueryError::RuntimeError("Failed to spawn blocking task".into()))?
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
                let mut ctx = context::ContractExecContext::new(
                    ExecutionMode::Estimating,
                    context.now_ms,
                    context.block_number,
                    context.worker_pubkey,
                    context.chain_storage,
                    context.req_id,
                );
                let log_handler = context.log_handler.clone();
                context::using(&mut ctx, move || {
                    let mut runtime = self.runtime_mut(log_handler);
                    let args = TransactionArguments {
                        origin,
                        transfer,
                        gas_limit: WEIGHT_REF_TIME_PER_SECOND * 10,
                        gas_free: true,
                        storage_deposit_limit: None,
                        deposit,
                    };
                    let ink_result = runtime.instantiate(
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
                    deposit: 0,
                };

                let mut runtime = self.runtime_mut(context.log_handler.clone());
                let output = runtime.call(
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

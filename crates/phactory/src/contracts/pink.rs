use std::time::Duration;

use crate::{
    contracts::{self, QueryContext, TransactionContext},
    system::{TransactionError, TransactionResult},
};
use anyhow::{anyhow, Context, Result};
use parity_scale_codec::{Decode, Encode};
use phala_crypto::sr25519::{Persistence, Sr25519SecretKey, KDF};
use phala_mq::{ContractClusterId, ContractId, MessageOrigin};
use phala_serde_more as more;
use phala_types::contract::{messaging::ResourceType, ConvertTo};
use pink::{
    capi::v1::{
        ecall::{ECalls, ECallsRo},
        ocall::OCalls,
        CrossCall,
    },
    runtimes::v1::using_ocalls,
    types::ExecutionMode,
};
use serde::{Deserialize, Serialize};
use sidevm::service::{Command as SidevmCommand, CommandSender, SystemMessage};
use sp_core::sr25519;
use sp_runtime::DispatchError;
use std::collections::{BTreeMap, BTreeSet};

use ::pink::{
    capi::v1::{self, ecall::EventCallbacks},
    constants::WEIGHT_REF_TIME_PER_SECOND,
    types::{
        AccountId, Address, Balance, BlockNumber, ExecSideEffects, Hash, TransactionArguments,
        Weight,
    },
};

pub use phala_types::contract::InkCommand;

use super::SidevmHandle;

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
    // todo!: fill it according to chain config
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

    fn is_in_query(&self) -> bool {
        let todo = "is_in_query";
        todo!()
    }

    fn emit_log(&self, contract: AccountId, in_query: bool, level: u8, message: String) {
        let Some(log_handler) = self.logger.as_ref() else {
            return;
        };
        let msg = SidevmCommand::PushSystemMessage(SystemMessage::PinkLog {
            block_number: self.block_number,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as _,
            in_query,
            contract: contract.clone().into(),
            level,
            message,
        });
        if log_handler.try_send(msg).is_err() {
            error!("Pink emit_log failed");
        }
    }

    fn emit_side_effects(&mut self, _effects: ExecSideEffects) {}

    fn exec_mode(&self) -> ExecutionMode {
        todo!()
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

    fn is_in_query(&self) -> bool {
        self.readonly().is_in_query()
    }

    fn emit_log(&self, contract: AccountId, in_query: bool, level: u8, message: String) {
        self.readonly().emit_log(contract, in_query, level, message)
    }

    fn emit_side_effects(&mut self, effects: ExecSideEffects) {
        self.effects = Some(effects);
    }

    fn exec_mode(&self) -> ExecutionMode {
        self.readonly().exec_mode()
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
    pub fn test_default(id: &ContractClusterId, runtime_version: (u32, u32)) -> Self {
        let mut cluster = Cluster {
            id: *id,
            storage: Default::default(),
            config: ClusterConfig {
                runtime_version,
                ..Default::default()
            },
        };
        cluster.default_runtime_mut().set_cluster_id(*id);
        assert_eq!(cluster.default_runtime().cluster_id(), *id);
        cluster
    }

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
        runtime.set_cluster_id(*id);
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

    pub fn set_system_contract(&mut self, contract: AccountId) {
        self.default_runtime_mut().set_system_contract(contract);
    }

    pub fn code_hash(&self, address: &AccountId) -> Option<Hash> {
        self.default_runtime().code_hash(address.clone())
    }

    pub fn upload_resource(
        &mut self,
        origin: &AccountId,
        resource_type: ResourceType,
        resource_data: Vec<u8>,
    ) -> Result<Hash, Vec<u8>> {
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

    pub fn setup(
        &mut self,
        gas_price: Balance,
        deposit_per_item: Balance,
        deposit_per_byte: Balance,
        treasury_account: &::pink::types::AccountId,
    ) {
        self.default_runtime_mut().setup(
            gas_price,
            deposit_per_item,
            deposit_per_byte,
            treasury_account.clone(),
        );
    }

    pub fn deposit(&mut self, who: &::pink::types::AccountId, amount: Balance) {
        self.default_runtime_mut().deposit(who.clone(), amount)
    }

    // pub fn bare_call(
    //     &self,
    //     contract_id: &ContractId,
    //     input_data: Vec<u8>,
    //     in_query: bool,
    //     tx_args: TransactionArguments,
    // ) -> (ContractExecResult, ExecSideEffects) {
    //     todo!()
    // }

    pub fn instantiate(
        &self,
        code_hash: Hash,
        input_data: Vec<u8>,
        salt: Vec<u8>,
        in_query: bool,
        tx_args: TransactionArguments,
    ) -> Result<(ContractId, ExecSideEffects)> {
        todo!()
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

                let origin = origin.cloned().ok_or(QueryError::BadOrigin)?;
                let mut runtime =
                    self.runtime_mut(context.log_handler.clone(), context.block_number);
                if deposit > 0 {
                    runtime.deposit(origin.clone(), deposit);
                }
                let args = TransactionArguments {
                    origin,
                    now: context.now_ms,
                    block_number: context.block_number,
                    transfer,
                    gas_limit: WEIGHT_REF_TIME_PER_SECOND * 10,
                    gas_free: true,
                    storage_deposit_limit: None,
                };
                let mode = if estimating {
                    ExecutionMode::Estimating
                } else {
                    ExecutionMode::Query
                };
                let ink_result = runtime.contract_call(contract_id.clone(), input_data, mode, args);
                let effects = if mode.is_estimating() {
                    None
                } else {
                    runtime.effects.take().map(|e| e.into_query_only_effects())
                };
                Ok((Response::Payload(ink_result), effects))
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
                let mut runtime =
                    self.runtime_mut(context.log_handler.clone(), context.block_number);
                if deposit > 0 {
                    runtime.deposit(origin.clone(), deposit);
                }
                let args = TransactionArguments {
                    origin,
                    now: context.now_ms,
                    block_number: context.block_number,
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
            }
        }
    }

    pub(crate) fn handle_command(
        &mut self,
        contract_id: &AccountId,
        origin: MessageOrigin,
        cmd: InkCommand,
        context: &mut contracts::TransactionContext,
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
                    now: context.block.now_ms,
                    block_number: context.block.block_number,
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

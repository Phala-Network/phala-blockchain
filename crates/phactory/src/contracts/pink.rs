use std::time::Duration;

use crate::contracts;
use crate::system::{TransactionError, TransactionResult};
use anyhow::{anyhow, Result};
use parity_scale_codec::{Decode, Encode};
use phala_mq::{ContractClusterId, ContractId, MessageOrigin};
use phala_types::contract::ConvertTo;
use pink::predefined_accounts::pallet_account;
use pink::runtime::{BoxedEventCallbacks, ExecSideEffects};
use runtime::{AccountId, BlockNumber, Hash};
use sidevm::service::{Command as SidevmCommand, CommandSender, SystemMessage};
use sp_runtime::{traits::ConstU32, BoundedVec};

#[derive(Debug, Encode, Decode)]
pub enum Command {
    InkMessage {
        nonce: BoundedVec<u8, ConstU32<32>>,
        message: Vec<u8>,
    },
}

#[derive(Debug, Encode, Decode)]
pub enum Query {
    InkMessage(Vec<u8>),
    SidevmQuery(Vec<u8>),
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

#[derive(Encode, Decode, Clone)]
pub struct Pink {
    pub(crate) instance: pink::Contract,
    cluster_id: ContractClusterId,
}

impl Pink {
    #[allow(clippy::too_many_arguments)]
    pub fn instantiate(
        cluster_id: ContractClusterId,
        storage: &mut pink::Storage,
        origin: AccountId,
        code_hash: Hash,
        input_data: Vec<u8>,
        salt: Vec<u8>,
        block_number: BlockNumber,
        now: u64,
        callbacks: Option<BoxedEventCallbacks>,
    ) -> Result<(Self, ExecSideEffects)> {
        let (instance, effects) = pink::Contract::new(
            storage,
            origin.clone(),
            code_hash,
            input_data,
            cluster_id.as_bytes().to_vec(),
            salt,
            block_number,
            now,
            callbacks,
        )
        .map_err(|err| anyhow!("Instantiate contract failed: {:?} origin={:?}", err, origin,))?;
        Ok((
            Self {
                cluster_id,
                instance,
            },
            effects,
        ))
    }

    pub fn from_address(address: AccountId, cluster_id: ContractClusterId) -> Self {
        let instance = pink::Contract::from_address(address);
        Self {
            instance,
            cluster_id,
        }
    }

    pub fn id(&self) -> ContractId {
        self.instance.address.convert_to()
    }

    pub fn address(&self) -> AccountId {
        self.instance.address.clone()
    }

    pub fn set_on_block_end_selector(&mut self, selector: u32) {
        self.instance.set_on_block_end_selector(selector)
    }
}

impl Pink {
    pub(crate) async fn handle_query(
        &self,
        origin: Option<&AccountId>,
        req: Query,
        context: &mut contracts::QueryContext,
        side_effects: &mut ExecSideEffects,
    ) -> Result<Response, QueryError> {
        match req {
            Query::InkMessage(input_data) => {
                let _guard = context
                    .query_scheduler
                    .acquire(self.id(), context.weight)
                    .await
                    .or(Err(QueryError::ServiceUnavailable))?;

                let origin = origin.ok_or(QueryError::BadOrigin)?;
                let storage = &mut context.storage;

                let (ink_result, effects) = self.instance.bare_call(
                    storage,
                    origin.clone(),
                    input_data,
                    true,
                    context.block_number,
                    context.now_ms,
                    ContractEventCallback::from_log_sender(
                        &context.log_handler,
                        context.block_number,
                    ),
                );
                if ink_result.result.is_err() {
                    log::error!("Pink [{:?}] query exec error: {:?}", self.id(), ink_result);
                } else {
                    *side_effects = effects.into_query_only_effects();
                }
                Ok(Response::Payload(ink_result.encode()))
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
            }
        }
    }

    pub(crate) fn handle_command(
        &mut self,
        origin: MessageOrigin,
        cmd: Command,
        context: &mut contracts::TransactionContext,
    ) -> TransactionResult {
        match cmd {
            Command::InkMessage { nonce, message } => {
                let origin: runtime::AccountId = match origin {
                    MessageOrigin::AccountId(origin) => origin.0.into(),
                    MessageOrigin::Pallet(_) => pallet_account(),
                    _ => return Err(TransactionError::BadOrigin),
                };

                let storage = cluster_storage(context.contract_clusters, &self.cluster_id)
                    .expect("Pink cluster should always exists!");

                let (result, effects) = self.instance.bare_call(
                    storage,
                    origin.clone(),
                    message,
                    false,
                    context.block.block_number,
                    context.block.now_ms,
                    ContractEventCallback::from_log_sender(
                        &context.log_handler,
                        context.block.block_number,
                    ),
                );

                if let Some(log_handler) = &context.log_handler {
                    let msg = SidevmCommand::PushSystemMessage(SystemMessage::PinkMessageOutput {
                        origin: origin.into(),
                        contract: self.instance.address.clone().into(),
                        block_number: context.block.block_number,
                        nonce: nonce.into_inner(),
                        output: result.result.encode(),
                    });
                    if log_handler.try_send(msg).is_err() {
                        error!("Pink emit message output to log handler failed");
                    }
                }

                let _ = pink::transpose_contract_result(&result).map_err(|err| {
                    log::error!("Pink [{:?}] command exec error: {:?}", self.id(), err);
                    TransactionError::Other(format!("Call contract method failed: {:?}", err))
                })?;
                Ok(effects)
            }
        }
    }

    pub(crate) fn on_block_end(
        &mut self,
        context: &mut contracts::TransactionContext,
    ) -> TransactionResult {
        let storage = cluster_storage(context.contract_clusters, &self.cluster_id)
            .expect("Pink cluster should always exists!");
        let effects = self
            .instance
            .on_block_end(
                storage,
                context.block.block_number,
                context.block.now_ms,
                ContractEventCallback::from_log_sender(
                    &context.log_handler,
                    context.block.block_number,
                ),
            )
            .map_err(|err| {
                log::error!("Pink [{:?}] on_block_end exec error: {:?}", self.id(), err);
                TransactionError::Other(format!("Call contract on_block_end failed: {:?}", err))
            })?;
        Ok(effects)
    }

    pub(crate) fn snapshot(&self) -> Self {
        self.clone()
    }
}

fn cluster_storage<'a>(
    clusters: &'a mut cluster::ClusterKeeper,
    cluster_id: &ContractClusterId,
) -> Result<&'a mut pink::Storage> {
    clusters
        .get_cluster_storage_mut(cluster_id)
        .ok_or_else(|| anyhow!("Contract cluster {:?} not found! qed!", cluster_id))
}

pub mod cluster {
    use super::Pink;

    use anyhow::{Context, Result};
    use phala_crypto::sr25519::{Persistence, Sr25519SecretKey, KDF};
    use phala_mq::{ContractClusterId, ContractId};
    use phala_serde_more as more;
    use phala_types::contract::messaging::ResourceType;
    use pink::{
        runtime::{BoxedEventCallbacks, ExecSideEffects},
        types::{AccountId, Hash},
    };
    use runtime::BlockNumber;
    use serde::{Deserialize, Serialize};
    use sp_core::sr25519;
    use sp_runtime::{AccountId32, DispatchError};
    use std::collections::{BTreeMap, BTreeSet};

    #[derive(Default, Serialize, Deserialize)]
    pub struct ClusterKeeper {
        clusters: BTreeMap<ContractClusterId, Cluster>,
    }

    impl ClusterKeeper {
        pub fn is_empty(&self) -> bool {
            self.clusters.is_empty()
        }

        pub fn len(&self) -> usize {
            self.clusters.len()
        }

        #[allow(clippy::too_many_arguments)]
        pub fn instantiate_contract(
            &mut self,
            cluster_id: ContractClusterId,
            origin: AccountId,
            code_hash: Hash,
            input_data: Vec<u8>,
            salt: Vec<u8>,
            block_number: BlockNumber,
            now: u64,
            callbacks: Option<BoxedEventCallbacks>,
        ) -> Result<ExecSideEffects> {
            let cluster = self
                .get_cluster_mut(&cluster_id)
                .context("Cluster must exist before instantiation")?;
            let (_, effects) = Pink::instantiate(
                cluster_id,
                &mut cluster.storage,
                origin,
                code_hash,
                input_data,
                salt,
                block_number,
                now,
                callbacks,
            )?;
            Ok(effects)
        }

        pub fn get_cluster_storage_mut(
            &mut self,
            cluster_id: &ContractClusterId,
        ) -> Option<&mut pink::Storage> {
            Some(&mut self.clusters.get_mut(cluster_id)?.storage)
        }

        pub fn get_cluster_mut(&mut self, cluster_id: &ContractClusterId) -> Option<&mut Cluster> {
            self.clusters.get_mut(cluster_id)
        }

        pub fn get_cluster_or_default_mut(
            &mut self,
            cluster_id: &ContractClusterId,
            cluster_key: &sr25519::Pair,
        ) -> &mut Cluster {
            self.clusters.entry(*cluster_id).or_insert_with(|| {
                let mut cluster = Cluster {
                    storage: Default::default(),
                    contracts: Default::default(),
                    key: cluster_key.clone(),
                    config: Default::default(),
                };
                let seed_key = cluster_key
                    .derive_sr25519_pair(&[b"ink key derivation seed"])
                    .expect("Derive key seed should always success!");
                cluster.set_id(cluster_id);
                cluster.set_key_seed(seed_key.dump_secret_key());
                cluster
            })
        }

        pub fn remove_cluster(&mut self, cluster_id: &ContractClusterId) -> Option<Cluster> {
            self.clusters.remove(cluster_id)
        }

        pub fn iter(&self) -> impl Iterator<Item = (&ContractClusterId, &Cluster)> {
            self.clusters.iter()
        }
    }

    #[derive(Serialize, Deserialize, Default)]
    pub struct ClusterConfig {
        pub log_handler: Option<ContractId>,
        // Version used to control the contract API availability.
        pub version: (u16, u16),
    }

    #[derive(Serialize, Deserialize)]
    pub struct Cluster {
        pub storage: pink::Storage,
        contracts: BTreeSet<ContractId>,
        #[serde(with = "more::key_bytes")]
        key: sr25519::Pair,
        pub config: ClusterConfig,
    }

    impl Cluster {
        /// Add a new contract to the cluster. Returns true if the contract is new.
        pub fn add_contract(&mut self, address: ContractId) -> bool {
            self.contracts.insert(address)
        }

        pub fn key(&self) -> &sr25519::Pair {
            &self.key
        }

        pub fn system_contract(&mut self) -> Option<AccountId32> {
            self.storage.system_contract()
        }

        pub fn set_system_contract(&mut self, contract: AccountId32) {
            self.storage.set_system_contract(contract);
        }

        pub fn set_id(&mut self, id: &ContractClusterId) {
            self.storage.set_cluster_id(id.as_bytes());
        }

        pub fn set_key_seed(&mut self, seed: Sr25519SecretKey) {
            self.storage.set_key_seed(seed);
        }

        pub fn upload_resource(
            &mut self,
            origin: AccountId,
            resource_type: ResourceType,
            resource_data: Vec<u8>,
        ) -> Result<Hash, DispatchError> {
            match resource_type {
                ResourceType::InkCode => self.storage.upload_code(origin, resource_data),
                ResourceType::SidevmCode => self.storage.upload_sidevm_code(origin, resource_data),
            }
        }

        pub fn get_resource(
            &mut self,
            resource_type: ResourceType,
            hash: &Hash,
        ) -> Option<Vec<u8>> {
            match resource_type {
                ResourceType::InkCode => None,
                ResourceType::SidevmCode => self.storage.get_sidevm_code(hash),
            }
        }

        pub fn iter_contracts(&self) -> impl Iterator<Item = &ContractId> {
            self.contracts.iter()
        }
    }
}

pub(crate) struct ContractEventCallback {
    log_handler: CommandSender,
    block_number: BlockNumber,
}

impl ContractEventCallback {
    pub fn new(log_handler: CommandSender, block_number: BlockNumber) -> Self {
        ContractEventCallback {
            log_handler,
            block_number,
        }
    }

    pub fn from_log_sender(
        log_handler: &Option<CommandSender>,
        block_number: BlockNumber,
    ) -> Option<BoxedEventCallbacks> {
        Some(Box::new(ContractEventCallback::new(
            log_handler.as_ref().cloned()?,
            block_number,
        )))
    }
}

impl pink::runtime::EventCallbacks for ContractEventCallback {
    fn emit_log(&self, contract: &AccountId, in_query: bool, level: u8, message: String) {
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
        if self.log_handler.try_send(msg).is_err() {
            error!("Pink emit_log failed");
        }
    }
}

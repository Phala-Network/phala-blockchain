use std::time::Duration;

use crate::contracts;
use crate::system::{TransactionError, TransactionResult};
use anyhow::{anyhow, Result};
use parity_scale_codec::{Decode, Encode};
use phala_mq::{ContractClusterId, ContractId, MessageOrigin};
use phala_types::contract::ConvertTo;
use pink::runtime::{BoxedEventCallbacks, ExecSideEffects};
use pink::types::Weight;
use pink::weights::constants::WEIGHT_REF_TIME_PER_SECOND;
use runtime::{AccountId, BlockNumber, Hash};
use sidevm::service::{Command as SidevmCommand, CommandSender, SystemMessage};

pub use phala_types::contract::InkCommand as Command;

#[derive(Debug, Encode, Decode)]
pub enum Query {
    InkMessage {
        payload: Vec<u8>,
        /// Amount of tokens deposit to the caller.
        deposit: u128,
        /// Amount of tokens transfer from the caller to the target contract.
        transfer: u128,
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

#[derive(Encode, Decode, Clone)]
pub struct Pink {
    pub(crate) instance: pink::Contract,
    cluster_id: ContractClusterId,
}

impl Pink {
    pub fn instantiate(
        cluster_id: ContractClusterId,
        code_hash: Hash,
        input_data: Vec<u8>,
        salt: Vec<u8>,
        in_query: bool,
        tx_args: ::pink::TransactionArguments,
    ) -> Result<(Self, ExecSideEffects)> {
        let origin = tx_args.origin.clone();
        let (instance, effects) =
            pink::Contract::new(code_hash, input_data, salt, in_query, tx_args).map_err(|err| {
                anyhow!("Instantiate contract failed: {:?} origin={:?}", err, origin,)
            })?;
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

    pub fn set_on_block_end_selector(&mut self, selector: u32, gas_limit: u64) {
        self.instance.set_on_block_end_selector(selector, gas_limit)
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
            Query::InkMessage {
                payload: input_data,
                deposit,
                transfer,
            } => {
                let _guard = context
                    .query_scheduler
                    .acquire(self.id(), context.weight)
                    .await
                    .or(Err(QueryError::ServiceUnavailable))?;

                let origin = origin.cloned().ok_or(QueryError::BadOrigin)?;
                let storage = &mut context.storage;
                if deposit > 0 {
                    storage.deposit(&origin, deposit);
                }
                let args = ::pink::TransactionArguments {
                    origin,
                    now: context.now_ms,
                    block_number: context.block_number,
                    storage,
                    transfer,
                    gas_limit: Weight::from_parts(WEIGHT_REF_TIME_PER_SECOND * 10, u64::MAX),
                    gas_free: true,
                    storage_deposit_limit: None,
                    callbacks: ContractEventCallback::from_log_sender(
                        &context.log_handler,
                        context.block_number,
                    ),
                };
                let (ink_result, effects) = self.instance.bare_call(input_data, true, args);
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
            Query::InkInstantiate {
                code_hash,
                salt,
                instantiate_data,
                deposit,
                transfer,
            } => {
                let _guard = context
                    .query_scheduler
                    .acquire(self.id(), context.weight)
                    .await
                    .or(Err(QueryError::ServiceUnavailable))?;

                let origin = origin.cloned().ok_or(QueryError::BadOrigin)?;
                let storage = &mut context.storage;
                if deposit > 0 {
                    storage.deposit(&origin, deposit);
                }
                let args = ::pink::TransactionArguments {
                    origin,
                    now: context.now_ms,
                    block_number: context.block_number,
                    storage,
                    transfer,
                    gas_limit: Weight::from_parts(WEIGHT_REF_TIME_PER_SECOND * 10, u64::MAX),
                    gas_free: true,
                    storage_deposit_limit: None,
                    callbacks: ContractEventCallback::from_log_sender(
                        &context.log_handler,
                        context.block_number,
                    ),
                };
                let (ink_result, _effects) =
                    ::pink::Contract::instantiate(code_hash, instantiate_data, salt, true, args);
                if ink_result.result.is_err() {
                    log::error!(
                        "Pink [{:?}] est instantiate error: {:?}",
                        self.id(),
                        ink_result
                    );
                }
                Ok(Response::Payload(ink_result.encode()))
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
            Command::InkMessage {
                nonce,
                message,
                transfer,
                gas_limit,
                storage_deposit_limit,
            } => {
                let storage = cluster_storage(context.contract_clusters, &self.cluster_id)
                    .expect("Pink cluster should always exists!");

                let mut gas_free = false;
                let origin: runtime::AccountId = match origin {
                    MessageOrigin::AccountId(origin) => origin.0.into(),
                    MessageOrigin::Pallet(_) => {
                        // The caller will be set to the system contract if it's from a pallet call
                        // and without charging for gas
                        gas_free = true;
                        storage
                            .system_contract()
                            .expect("BUG: system contract missing")
                    }
                    _ => return Err(TransactionError::BadOrigin),
                };

                let args = ::pink::TransactionArguments {
                    origin: origin.clone(),
                    now: context.block.now_ms,
                    block_number: context.block.block_number,
                    storage,
                    transfer,
                    gas_limit: Weight::from_ref_time(gas_limit),
                    gas_free,
                    storage_deposit_limit,
                    callbacks: ContractEventCallback::from_log_sender(
                        &context.log_handler,
                        context.block.block_number,
                    ),
                };

                let (result, effects) = self.instance.bare_call(message, false, args);

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

                let _ = pink::transpose_contract_result(result).map_err(|err| {
                    log::error!("Pink [{:?}] command exec error: {:?}", self.id(), err);
                    TransactionError::Other(format!("Call contract method failed: {err:?}"))
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
                TransactionError::Other(format!("Call contract on_block_end failed: {err:?}"))
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
    use anyhow::{Context, Result};
    use parity_scale_codec::Decode;
    use phala_crypto::sr25519::{Persistence, Sr25519SecretKey, KDF};
    use phala_mq::{ContractClusterId, ContractId};
    use phala_serde_more as more;
    use phala_types::contract::messaging::ResourceType;
    use pink::{
        types::{AccountId, Balance, Hash},
        weights::Weight,
    };
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

        pub fn system_contract(&self) -> Option<AccountId32> {
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
            origin: &AccountId,
            resource_type: ResourceType,
            resource_data: Vec<u8>,
        ) -> Result<Hash, DispatchError> {
            match resource_type {
                ResourceType::InkCode => self.storage.upload_code(origin, resource_data, true),
                ResourceType::SidevmCode => self.storage.upload_sidevm_code(origin, resource_data),
                ResourceType::IndeterministicInkCode => {
                    self.storage.upload_code(origin, resource_data, false)
                }
            }
        }

        pub fn get_resource(&self, resource_type: ResourceType, hash: &Hash) -> Option<Vec<u8>> {
            match resource_type {
                ResourceType::InkCode => None,
                ResourceType::SidevmCode => self.storage.get_sidevm_code(hash),
                ResourceType::IndeterministicInkCode => None,
            }
        }

        pub fn iter_contracts(&self) -> impl Iterator<Item = &ContractId> {
            self.contracts.iter()
        }

        pub fn setup(
            &mut self,
            gas_price: Balance,
            deposit_per_item: Balance,
            deposit_per_byte: Balance,
            treasury_account: &::pink::types::AccountId,
        ) {
            self.storage.setup(
                gas_price,
                deposit_per_item,
                deposit_per_byte,
                treasury_account,
            );
        }

        pub fn deposit(&mut self, who: &::pink::types::AccountId, amount: Balance) {
            self.storage.deposit(who, amount)
        }

        pub fn set_system_contract_code(&mut self, code_hash: Hash) -> Result<(), DispatchError> {
            self.storage.set_system_contract_code(code_hash)?;
            self.sync_system_contract_version()
                .expect("Failed to sync the system contract version. Please upgrade pRuntime!");
            Ok(())
        }

        pub fn sync_system_contract_version(&mut self) -> Result<()> {
            let Some(system_address) = self.system_contract() else {
                anyhow::bail!("No system contract");
            };
            let system = pink::Contract::from_address(system_address.clone());
            // System::version
            let selector = 0x87c98a8d_u32.to_be_bytes().to_vec();
            let args = ::pink::TransactionArguments {
                origin: system_address,
                now: 1,
                block_number: 1,
                storage: &mut self.storage,
                transfer: 0,
                gas_limit: Weight::MAX,
                gas_free: true,
                storage_deposit_limit: None,
                callbacks: None,
            };
            let (result, _) = system.bare_call(selector, true, args);
            let output = result.result.map_err(|err| {
                anyhow::anyhow!("Failed to get the system contract version: {err:?}")
            })?;
            self.config.version = Decode::decode(&mut &output.data[..])
                .context("Failed to decode the system contract version")?;
            const SUPPORTED_API_VERSION: u16 = 0;
            if self.config.version.0 > SUPPORTED_API_VERSION {
                anyhow::bail!(
                    "The pink-system version is not supported, please upgrade the pRuntime"
                );
            }
            Ok(())
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

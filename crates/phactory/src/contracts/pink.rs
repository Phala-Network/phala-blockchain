use crate::contracts;
use crate::system::{TransactionError, TransactionResult};
use anyhow::{anyhow, Result};
use parity_scale_codec::{Decode, Encode};
use phala_mq::{ContractClusterId, MessageOrigin};
use pink::runtime::ExecSideEffects;
use runtime::{AccountId, BlockNumber};

use super::{contract_address_to_id, NativeContractMore};

#[derive(Debug, Encode, Decode)]
pub enum Command {
    InkMessage { nonce: Vec<u8>, message: Vec<u8> },
}

#[derive(Debug, Encode, Decode)]
pub enum Query {
    InkMessage(Vec<u8>),
}

#[derive(Debug, Encode, Decode)]
pub enum Response {
    InkMessageReturn(Vec<u8>),
}

#[derive(Debug, Encode, Decode)]
pub enum QueryError {
    BadOrigin,
    RuntimeError(String),
}

#[derive(Encode, Decode, Clone)]
pub struct Pink {
    instance: pink::Contract,
    cluster_id: ContractClusterId,
}

impl Pink {
    pub fn instantiate(
        cluster_id: ContractClusterId,
        storage: &mut pink::Storage,
        origin: AccountId,
        wasm_bin: Vec<u8>,
        input_data: Vec<u8>,
        salt: Vec<u8>,
        block_number: BlockNumber,
        now: u64,
    ) -> Result<(Self, ExecSideEffects)> {
        let (instance, effects) = pink::Contract::new(
            storage,
            origin.clone(),
            wasm_bin,
            input_data,
            cluster_id.as_bytes().to_vec(),
            salt,
            block_number,
            now,
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
}

impl contracts::NativeContract for Pink {
    type Cmd = Command;

    type QReq = Query;

    type QResp = Result<Response, QueryError>;

    fn handle_query(
        &self,
        origin: Option<&AccountId>,
        req: Query,
        context: &mut contracts::QueryContext,
    ) -> Result<Response, QueryError> {
        let origin = origin.ok_or(QueryError::BadOrigin)?;
        match req {
            Query::InkMessage(input_data) => {
                let storage = &mut context.storage;

                let (ink_result, _effects) = self.instance.bare_call(
                    storage,
                    origin.clone(),
                    input_data,
                    true,
                    context.block_number,
                    context.now_ms,
                );
                if ink_result.result.is_err() {
                    log::error!("Pink [{:?}] query exec error: {:?}", self.id(), ink_result);
                }
                return Ok(Response::InkMessageReturn(ink_result.encode()));
            }
        }
    }

    fn handle_command(
        &mut self,
        origin: MessageOrigin,
        cmd: Command,
        context: &mut contracts::NativeContext,
    ) -> TransactionResult {
        match cmd {
            Command::InkMessage { nonce: _, message } => {
                let origin: runtime::AccountId = match origin {
                    MessageOrigin::AccountId(origin) => origin.0.into(),
                    _ => return Err(TransactionError::BadOrigin),
                };

                let storage = cluster_storage(&mut context.contract_clusters, &self.cluster_id)
                    .expect("Pink cluster should always exists!");

                let (result, effects) = self.instance.bare_call(
                    storage,
                    origin.clone(),
                    message,
                    false,
                    context.block.block_number,
                    context.block.now_ms,
                );

                let ret = pink::transpose_contract_result(&result).map_err(|err| {
                    log::error!("Pink [{:?}] command exec error: {:?}", self.id(), err);
                    TransactionError::Other(format!("Call contract method failed: {:?}", err))
                })?;

                // TODO.kevin: store the output to some where.
                let _ = ret;
                Ok(effects)
            }
        }
    }

    fn on_block_end(&mut self, context: &mut contracts::NativeContext) -> TransactionResult {
        let storage = cluster_storage(&mut context.contract_clusters, &self.cluster_id)
            .expect("Pink cluster should always exists!");
        let effects = self
            .instance
            .on_block_end(storage, context.block.block_number, context.block.now_ms)
            .map_err(|err| {
                log::error!("Pink [{:?}] on_block_end exec error: {:?}", self.id(), err);
                TransactionError::Other(format!("Call contract on_block_end failed: {:?}", err))
            })?;
        Ok(effects)
    }

    fn snapshot(&self) -> Self {
        self.clone()
    }
}

impl NativeContractMore for Pink {
    fn id(&self) -> phala_mq::ContractId {
        contract_address_to_id(&self.instance.address)
    }

    fn set_on_block_end_selector(&mut self, selector: u32) {
        self.instance.set_on_block_end_selector(selector)
    }
}

fn cluster_storage<'a>(
    clusters: &'a mut cluster::ClusterKeeper,
    cluster_id: &ContractClusterId,
) -> Result<&'a mut pink::Storage> {
    clusters
        .get_cluster_storage_mut(cluster_id)
        .ok_or(anyhow!("Contract cluster {:?} not found! qed!", cluster_id))
}

pub mod cluster {
    use super::Pink;

    use anyhow::Result;
    use phala_crypto::sr25519::{Persistence, Sr25519SecretKey, KDF};
    use phala_mq::{ContractClusterId, ContractId};
    use phala_serde_more as more;
    use pink::{
        runtime::ExecSideEffects,
        types::{AccountId, Hash},
    };
    use runtime::BlockNumber;
    use serde::{Deserialize, Serialize};
    use sp_core::sr25519;
    use sp_runtime::DispatchError;
    use std::collections::{BTreeMap, BTreeSet};

    #[derive(Default, Serialize, Deserialize)]
    pub struct ClusterKeeper {
        clusters: BTreeMap<ContractClusterId, Cluster>,
    }

    impl ClusterKeeper {
        pub fn instantiate_contract(
            &mut self,
            cluster_id: ContractClusterId,
            origin: AccountId,
            wasm_bin: Vec<u8>,
            input_data: Vec<u8>,
            salt: Vec<u8>,
            contract_key: &sr25519::Pair,
            block_number: BlockNumber,
            now: u64,
        ) -> Result<ExecSideEffects> {
            let cluster = self.get_cluster_or_default_mut(&cluster_id, contract_key);
            let (_, effects) = Pink::instantiate(
                cluster_id,
                &mut cluster.storage,
                origin,
                wasm_bin,
                input_data,
                salt,
                block_number,
                now,
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
            self.clusters.entry(cluster_id.clone()).or_insert_with(|| {
                let mut cluster = Cluster {
                    storage: Default::default(),
                    contracts: Default::default(),
                    key: cluster_key.clone(),
                };
                let seed_key = cluster_key
                    .derive_sr25519_pair(&[b"ink key derivation seed"])
                    .expect("Derive key seed should always success!");
                cluster.set_id(cluster_id);
                cluster.set_key_seed(seed_key.dump_secret_key());
                cluster
            })
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct Cluster {
        pub storage: pink::Storage,
        contracts: BTreeSet<ContractId>,
        #[serde(with = "more::key_bytes")]
        key: sr25519::Pair,
    }

    impl Cluster {
        /// Add a new contract to the cluster. Returns true if the contract is new.
        pub fn add_contract(&mut self, address: ContractId) -> bool {
            self.contracts.insert(address)
        }

        pub fn key(&self) -> &sr25519::Pair {
            &self.key
        }

        pub fn set_id(&mut self, id: &ContractClusterId) {
            self.storage.set_cluster_id(id.as_bytes());
        }

        pub fn set_key_seed(&mut self, seed: Sr25519SecretKey) {
            self.storage.set_key_seed(seed);
        }

        pub fn upload_code(
            &mut self,
            origin: AccountId,
            code: Vec<u8>,
        ) -> Result<Hash, DispatchError> {
            self.storage.upload_code(origin, code)
        }

    }
}

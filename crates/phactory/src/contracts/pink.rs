use crate::contracts;
use crate::system::{TransactionError, TransactionResult};
use anyhow::{anyhow, Result};
use parity_scale_codec::{Decode, Encode};
use phala_mq::{ContractGroupId, ContractId, MessageOrigin};
use pink::runtime::ExecSideEffects;
use runtime::{AccountId, BlockNumber};
use phala_serde_more as more;

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

pub struct Pink {
    instance: pink::Contract,
    group: ContractGroupId,
}

impl Pink {
    pub fn instantiate(
        group: ContractGroupId,
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
            salt,
            block_number,
            now,
        )
        .map_err(|err| anyhow!("Instantiate contract failed: {:?} origin={:?}", err, origin,))?;
        Ok((Self { group, instance }, effects))
    }

    pub fn from_address(address: AccountId, group: ContractGroupId) -> Self {
        let instance = pink::Contract::from_address(address);
        Self { instance, group }
    }

    pub fn address_to_id(address: &AccountId) -> ContractId {
        let inner: &[u8; 32] = address.as_ref();
        inner.into()
    }
}

impl contracts::NativeContract for Pink {
    type Cmd = Command;

    type QReq = Query;

    type QResp = Result<Response, QueryError>;

    fn id(&self) -> ContractId {
        Pink::address_to_id(&self.instance.address)
    }

    fn group_id(&self) -> Option<phala_mq::ContractGroupId> {
        Some(self.group.clone())
    }

    fn handle_query(
        &mut self,
        origin: Option<&AccountId>,
        req: Query,
        context: &mut contracts::QueryContext,
    ) -> Result<Response, QueryError> {
        let origin = origin.ok_or(QueryError::BadOrigin)?;
        match req {
            Query::InkMessage(input_data) => {
                let storage = group_storage(&mut context.contract_groups, &self.group)
                    .expect("Pink group should always exists!");

                let (ret, _messages) = self
                    .instance
                    .bare_call(
                        storage,
                        origin.clone(),
                        input_data,
                        true,
                        context.block_number,
                        context.now_ms,
                    )
                    .map_err(|err| {
                        log::error!("Pink [{:?}] query exec error: {:?}", self.id(), err);
                        QueryError::RuntimeError(format!("Call contract method failed: {:?}", err))
                    })?;
                return Ok(Response::InkMessageReturn(ret));
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

                let storage = group_storage(&mut context.contract_groups, &self.group)
                    .expect("Pink group should always exists!");

                let (ret, effects) = self
                    .instance
                    .bare_call(
                        storage,
                        origin.clone(),
                        message,
                        false,
                        context.block.block_number,
                        context.block.now_ms,
                    )
                    .map_err(|err| {
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
        let storage = group_storage(&mut context.contract_groups, &self.group)
            .expect("Pink group should always exists!");
        let effects = self
            .instance
            .on_block_end(storage, context.block.block_number, context.block.now_ms)
            .map_err(|err| {
                log::error!("Pink [{:?}] on_block_end exec error: {:?}", self.id(), err);
                TransactionError::Other(format!("Call contract on_block_end failed: {:?}", err))
            })?;
        Ok(effects)
    }

    fn set_on_block_end_selector(&mut self, selector: u32) {
        self.instance.set_on_block_end_selector(selector)
    }
}

fn group_storage<'a>(
    groups: &'a mut group::GroupKeeper,
    group_id: &ContractGroupId,
) -> Result<&'a mut pink::Storage> {
    groups
        .get_group_storage_mut(group_id)
        .ok_or(anyhow!("Contract group {:?} not found! qed!", group_id))
}

pub mod group {
    use super::Pink;

    use anyhow::Result;
    use phala_mq::{ContractGroupId, ContractId};
    use pink::{runtime::ExecSideEffects, types::AccountId};
    use runtime::BlockNumber;
    use serde::{Deserialize, Serialize};
    use sp_core::sr25519;
    use std::collections::{BTreeMap, BTreeSet};
    use phala_serde_more as more;

    #[derive(Default, Serialize, Deserialize)]
    pub struct GroupKeeper {
        groups: BTreeMap<ContractGroupId, Group>,
    }

    impl GroupKeeper {
        pub fn instantiate_contract(
            &mut self,
            group_id: ContractGroupId,
            origin: AccountId,
            wasm_bin: Vec<u8>,
            input_data: Vec<u8>,
            salt: Vec<u8>,
            contract_key: &sr25519::Pair,
            block_number: BlockNumber,
            now: u64,
        ) -> Result<ExecSideEffects> {
            let group = self
                .groups
                .entry(group_id.clone())
                .or_insert_with(|| Group {
                    storage: Default::default(),
                    contracts: Default::default(),
                    key: contract_key.clone(),
                });
            let (_, effects) = Pink::instantiate(
                group_id,
                &mut group.storage,
                origin,
                wasm_bin,
                input_data,
                salt,
                block_number,
                now,
            )?;
            Ok(effects)
        }

        pub fn get_group_storage_mut(
            &mut self,
            group_id: &ContractGroupId,
        ) -> Option<&mut pink::Storage> {
            Some(&mut self.groups.get_mut(group_id)?.storage)
        }

        pub fn get_group_mut(&mut self, group_id: &ContractGroupId) -> Option<&mut Group> {
            self.groups.get_mut(group_id)
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct Group {
        #[serde(with = "more::todo")]
        pub storage: pink::Storage,
        contracts: BTreeSet<ContractId>,
        #[serde(with = "more::sr25519_hex")]
        key: sr25519::Pair,
    }

    impl Group {
        pub fn add_contract(&mut self, address: ContractId) {
            self.contracts.insert(address);
        }

        pub fn key(&self) -> &sr25519::Pair {
            &self.key
        }
    }
}

pub mod messaging {
    use parity_scale_codec::{Decode, Encode};
    use phala_crypto::sr25519::Sr25519SecretKey;
    use phala_mq::{bind_topic, ContractGroupId};
    use phala_types::WorkerPublicKey;
    use pink::types::AccountId;

    pub use phala_types::messaging::WorkerPinkReport;

    bind_topic!(WorkerPinkRequest, b"phala/pink/worker/request");
    #[derive(Encode, Decode, Debug)]
    pub enum WorkerPinkRequest {
        Instantiate {
            group_id: ContractGroupId,
            worker: WorkerPublicKey,
            nonce: Vec<u8>,
            owner: AccountId,
            wasm_bin: Vec<u8>,
            input_data: Vec<u8>,
            salt: Vec<u8>,
            key: Sr25519SecretKey,
        },
    }

    bind_topic!(GKPinkRequest, b"phala/pink/gk/request");
    #[derive(Encode, Decode, Debug)]
    pub enum GKPinkRequest {
        Instantiate {
            group_id: Option<ContractGroupId>, // None for create a new one
            worker: WorkerPublicKey, // TODO: None for choosing one by GK or by the group_id?
            wasm_bin: Vec<u8>,
            input_data: Vec<u8>,
        },
    }
}

#[test]
fn test_make_pink_request() {
    use crate::secret_channel::Payload;
    use hex_literal::hex;

    let request = messaging::WorkerPinkRequest::Instantiate {
        group_id: Default::default(),
        worker: phala_types::WorkerPublicKey(hex!(
            "3a3d45dc55b57bf542f4c6ff41af080ec675317f4ed50ae1d2713bf9f892692d"
        )),
        nonce: vec![],
        owner: hex!("3a3d45dc55b57bf542f4c6ff41af080ec675317f4ed50ae1d2713bf9f892692d").into(),
        wasm_bin: include_bytes!("fixtures/flip.contract").to_vec(),
        input_data: hex!("9bae9d5e01").to_vec(),
        salt: vec![],
        key: [0; 64],
    };
    let message = Payload::Plain(request);
    println!("message: {}", hex::encode(message.encode()));
}

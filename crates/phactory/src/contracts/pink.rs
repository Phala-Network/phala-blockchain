use crate::contracts;
use crate::system::{TransactionError, TransactionResult};
use anyhow::{anyhow, Result};
use parity_scale_codec::{Decode, Encode};
use phala_mq::{ContractGroupId, MessageOrigin};
use pink::runtime::ExecSideEffects;
use runtime::{AccountId, BlockNumber};

use super::NativeContractMore;

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

#[derive(Encode, Decode)]
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

    pub fn address_to_id(address: &AccountId) -> contracts::NativeContractId {
        let inner: &[u8; 32] = address.as_ref();
        inner.into()
    }
}

impl contracts::NativeContract for Pink {
    type Cmd = Command;

    type QReq = Query;

    type QResp = Result<Response, QueryError>;

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

                let storage = group_storage(&mut context.contract_groups, &self.group)
                    .expect("Pink group should always exists!");

                let (result, effects) = self
                    .instance
                    .bare_call(
                        storage,
                        origin.clone(),
                        message,
                        false,
                        context.block.block_number,
                        context.block.now_ms,
                    );

                let ret = pink::transpose_contract_result(&result)
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
}

impl NativeContractMore for Pink {
    fn id(&self) -> contracts::NativeContractId {
        Pink::address_to_id(&self.instance.address)
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
    use phala_serde_more as more;
    use pink::{runtime::ExecSideEffects, types::AccountId};
    use runtime::BlockNumber;
    use serde::{Deserialize, Serialize};
    use sp_core::sr25519;
    use std::collections::{BTreeMap, BTreeSet};

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
                .get_group_or_default_mut(&group_id, contract_key);
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

        pub fn get_group_or_default_mut(
            &mut self,
            group_id: &ContractGroupId,
            contract_key: &sr25519::Pair,
        ) -> &mut Group {
            self.groups
                .entry(group_id.clone())
                .or_insert_with(|| Group {
                    storage: Default::default(),
                    contracts: Default::default(),
                    key: contract_key.clone(),
                })
        }

        pub fn commit_changes(&mut self) -> anyhow::Result<()> {
            for group in self.groups.values_mut() {
                group.commit_changes()?;
            }
            Ok(())
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct Group {
        pub storage: pink::Storage,
        contracts: BTreeSet<ContractId>,
        #[serde(with = "more::key_bytes")]
        key: sr25519::Pair,
    }

    impl Group {
        /// Add a new contract to the group. Returns true if the contract is new.
        pub fn add_contract(&mut self, address: ContractId) -> bool {
            self.contracts.insert(address)
        }

        pub fn key(&self) -> &sr25519::Pair {
            &self.key
        }

        pub fn commit_changes(&mut self) -> anyhow::Result<()> {
            self.storage.commit_changes();
            Ok(())
        }
    }
}

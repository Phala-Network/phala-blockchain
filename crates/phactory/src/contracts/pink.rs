use crate::contracts;
use crate::system::{TransactionError, TransactionResult};
use anyhow::{anyhow, Result};
use parity_scale_codec::{Decode, Encode};
use phala_mq::{ContractGroupId, ContractId, MessageOrigin};
use runtime::AccountId;

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
    ) -> Result<Self> {
        let instance =
            pink::Contract::new(storage, origin.clone(), wasm_bin, input_data, salt)
                .map_err(|err| {
                    anyhow!(
                        "Instantiate contract failed: {:?} origin={:?}",
                        err,
                        origin,
                    )
                })?;
        Ok(Self { group, instance })
    }
}

impl contracts::NativeContract for Pink {
    type Cmd = Command;

    type QReq = Query;

    type QResp = Result<Response, QueryError>;

    fn id(&self) -> ContractId {
        let inner: &[u8; 32] = self.instance.address.as_ref();
        inner.into()
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
                let ret = self
                    .instance
                    .bare_call(storage, origin.clone(), input_data, true)
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

                let ret = self
                    .instance
                    .bare_call(storage, origin.clone(), message, false)
                    .map_err(|err| {
                        log::error!("Pink [{:?}] command exec error: {:?}", self.id(), err);
                        TransactionError::Other(format!("Call contract method failed: {:?}", err))
                    })?;
                // TODO.kevin: report the output to the chain?
                let _ = ret;
            }
        }
        Ok(())
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

    use crate::contracts::support::NativeContract as _;
    use anyhow::Result;
    use phala_mq::{ContractGroupId, ContractId};
    use pink::types::AccountId;
    use std::collections::{BTreeMap, BTreeSet};

    #[derive(Default)]
    pub struct GroupKeeper {
        groups: BTreeMap<ContractGroupId, Group>,
    }

    #[derive(Default)]
    pub struct Group {
        storage: pink::Storage,
        contracts: BTreeSet<ContractId>,
    }

    impl GroupKeeper {
        pub fn instantiate_contract(
            &mut self,
            group_id: ContractGroupId,
            origin: AccountId,
            wasm_bin: Vec<u8>,
            input_data: Vec<u8>,
            salt: Vec<u8>,
        ) -> Result<Pink> {
            let group = self.groups.entry(group_id.clone()).or_default();
            let pink = Pink::instantiate(
                group_id,
                &mut group.storage,
                origin,
                wasm_bin,
                input_data,
                salt,
            )?;
            group.contracts.insert(pink.id().clone());
            Ok(pink)
        }

        pub fn get_group_storage_mut(
            &mut self,
            group_id: &ContractGroupId,
        ) -> Option<&mut pink::Storage> {
            Some(&mut self.groups.get_mut(group_id)?.storage)
        }
    }
}

pub mod messaging {
    use parity_scale_codec::{Decode, Encode};
    use phala_crypto::{ecdh::EcdhPublicKey, sr25519::Sr25519SecretKey};
    use phala_mq::{bind_topic, ContractGroupId, ContractId};
    use phala_types::WorkerPublicKey;
    use pink::types::AccountId;

    pub use phala_types::messaging::{WorkerPinkReport, ContractInfo};

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

use crate::contracts;
use crate::system::{TransactionError, TransactionResult};
use anyhow::{anyhow, Context, Result};
use parity_scale_codec::{Decode, Encode};
use phala_mq::MessageOrigin;
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
}

impl Pink {
    pub fn instantiate(
        origin: AccountId,
        contract_content: &[u8],
        input_data: Vec<u8>,
        salt: Vec<u8>,
    ) -> Result<Self> {
        let file = pink::ContractFile::load(contract_content).context("Parse contract")?;
        let code_hash = file.source.hash;
        let instance = pink::Contract::new(origin.clone(), file.source.wasm, input_data, salt)
            .map_err(|err| {
                anyhow!(
                    "Instantiate contract failed: {:?} origin={:?}, code_hash: {:?}",
                    err,
                    origin,
                    code_hash,
                )
            })?;
        Ok(Self { instance })
    }
}

impl contracts::NativeContract for Pink {
    type Cmd = Command;

    type QReq = Query;

    type QResp = Result<Response, QueryError>;

    fn id(&self) -> phala_mq::ContractId {
        let inner: &[u8; 32] = self.instance.address.as_ref();
        inner.into()
    }

    fn handle_query(
        &mut self,
        origin: Option<&AccountId>,
        req: Query,
    ) -> Result<Response, QueryError> {
        let origin = origin.ok_or(QueryError::BadOrigin)?;
        match req {
            Query::InkMessage(input_data) => {
                let ret = self
                    .instance
                    .bare_call(origin.clone(), input_data, true)
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
        _context: &contracts::NativeContext,
        origin: MessageOrigin,
        cmd: Command,
    ) -> TransactionResult {
        match cmd {
            Command::InkMessage { nonce: _, message } => {
                let origin: runtime::AccountId = match origin {
                    MessageOrigin::AccountId(origin) => origin.0.into(),
                    _ => return Err(TransactionError::BadOrigin),
                };

                let ret = self
                    .instance
                    .bare_call(origin.clone(), message, false)
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

pub mod messaging {
    use phala_crypto::sr25519::Sr25519SecretKey;
    use phala_mq::{ContractId, bind_topic};
    use parity_scale_codec::{Encode, Decode};
    use phala_types::WorkerPublicKey;
    use pink::types::AccountId;

    bind_topic!(PinkRequest, b"phala/pink/request");
    #[derive(Encode, Decode, Debug)]
    pub enum PinkRequest {
        Deploy {
            worker: WorkerPublicKey,
            nonce: Vec<u8>,
            owner: AccountId,
            contract: Vec<u8>,
            input_data: Vec<u8>,
            salt: Vec<u8>,
            key: Sr25519SecretKey,
        },
    }

    bind_topic!(PinkReport, b"phala/pink/report");
    #[derive(Encode, Decode, Debug)]
    pub enum PinkReport {
        DeployStatus {
            nonce: Vec<u8>,
            owner: AccountId,
            result: Result<ContractId, String>,
        },
    }
}

use std::sync::{Arc, Mutex};

use phala_crypto::ecdh::EcdhPublicKey;
use phala_mq::traits::MessageChannel;
use runtime::BlockNumber;
use serde::{Deserialize, Serialize};
use sidevm::{instrument::instrument, VmId};

use super::pink::cluster::ClusterKeeper;
use super::*;
use crate::secret_channel::SecretReceiver;
use crate::types::BlockInfo;
use anyhow::{anyhow, bail};
use phala_serde_more as more;

pub struct ExecuteEnv<'a, 'b> {
    pub block: &'a mut BlockInfo<'b>,
    pub contract_clusters: &'a mut ClusterKeeper,
}

pub struct NativeContext<'a, 'b> {
    pub block: &'a mut BlockInfo<'b>,
    pub mq: &'a SignedMessageChannel,
    pub secret_mq: SecretMessageChannel<'a, SignedMessageChannel>,
    pub contract_clusters: &'a mut ClusterKeeper,
    pub self_id: ContractId,
}

pub struct QueryContext {
    pub block_number: BlockNumber,
    pub now_ms: u64,
    pub storage: ::pink::Storage,
}

impl NativeContext<'_, '_> {
    pub fn mq(&self) -> &SignedMessageChannel {
        self.mq
    }
}

pub trait NativeContract {
    type Cmd: Decode + Debug;
    type QReq: Decode + Debug;
    type QResp: Encode + Debug;

    fn handle_command(
        &mut self,
        _origin: MessageOrigin,
        _cmd: Self::Cmd,
        _context: &mut NativeContext,
    ) -> TransactionResult {
        Ok(Default::default())
    }
    fn handle_query(
        &self,
        origin: Option<&chain::AccountId>,
        req: Self::QReq,
        context: &mut QueryContext,
    ) -> Self::QResp;
    fn on_block_end(&mut self, _context: &mut NativeContext) -> TransactionResult {
        Ok(Default::default())
    }

    fn snapshot(&self) -> Self
    where
        Self: Sized;
}

pub(crate) struct Query {
    contract: AnyContract,
}

impl Query {
    pub fn handle_query(
        &self,
        origin: Option<&runtime::AccountId>,
        req: OpaqueQuery,
        context: &mut QueryContext,
    ) -> Result<OpaqueReply, OpaqueError> {
        self.contract.handle_query(origin, req, context)
    }
}

pub(crate) struct RawData(Vec<u8>);

impl Decode for RawData {
    fn decode<I: parity_scale_codec::Input>(
        input: &mut I,
    ) -> Result<Self, parity_scale_codec::Error> {
        // The remaining_len is not guaranteed to be correct by the trait Input definition. We only
        // decode the RawData with <&[u8] as Input>, which obviously impl the correct remaining_len.
        let mut remaining_len = input
            .remaining_len()?
            .ok_or("Can not decode RawData without length")?;
        let mut decoded = Vec::with_capacity(remaining_len);
        let mut buf = [0u8; 256];
        loop {
            let chunk = remaining_len.min(buf.len());
            input.read(&mut buf[..chunk])?;
            decoded.extend_from_slice(&buf[..chunk]);
            remaining_len -= chunk;
            if remaining_len == 0 {
                break;
            }
        }
        Ok(RawData(decoded))
    }
}

pub enum SidevmHandle {
    Running(sidevm::service::CommandSender),
    Terminated,
}

impl SidevmHandle {
    fn is_terminated(&self) -> bool {
        match self {
            SidevmHandle::Running(_) => false,
            SidevmHandle::Terminated => true,
        }
    }
}

impl Default for SidevmHandle {
    fn default() -> Self {
        SidevmHandle::Terminated
    }
}

#[derive(Serialize, Deserialize)]
struct SidevmInfo {
    code: Vec<u8>,
    memory_pages: u32,
    #[serde(skip, default)]
    handle: Arc<Mutex<SidevmHandle>>,
}

#[derive(Serialize, Deserialize)]
pub struct FatContract {
    #[serde(with = "more::scale_bytes")]
    contract: AnyContract,
    send_mq: SignedMessageChannel,
    cmd_rcv_mq: SecretReceiver<RawData>,
    #[serde(with = "crate::secret_channel::ecdh_serde")]
    ecdh_key: KeyPair,
    cluster_id: phala_mq::ContractClusterId,
    contract_id: phala_mq::ContractId,
    sidevm_info: Option<SidevmInfo>,
}

impl FatContract {
    pub(crate) fn new(
        contract: impl Into<AnyContract>,
        send_mq: SignedMessageChannel,
        cmd_rcv_mq: SecretReceiver<RawData>,
        ecdh_key: KeyPair,
        cluster_id: phala_mq::ContractClusterId,
        contract_id: phala_mq::ContractId,
    ) -> Self {
        FatContract {
            contract: contract.into(),
            send_mq,
            cmd_rcv_mq,
            ecdh_key,
            cluster_id,
            contract_id,
            sidevm_info: None,
        }
    }

    pub(crate) fn id(&self) -> ContractId {
        self.contract_id
    }

    pub(crate) fn cluster_id(&self) -> phala_mq::ContractClusterId {
        self.cluster_id
    }

    pub(crate) fn snapshot_for_query(&self) -> Query {
        Query {
            contract: self.contract.snapshot(),
        }
    }

    pub(crate) fn process_next_message(
        &mut self,
        env: &mut ExecuteEnv,
    ) -> Option<TransactionResult> {
        let secret_mq = SecretMessageChannel::new(&self.ecdh_key, &self.send_mq);
        let mut context = NativeContext {
            block: env.block,
            mq: &self.send_mq,
            secret_mq,
            contract_clusters: &mut env.contract_clusters,
            self_id: self.id(),
        };

        phala_mq::select! {
            next_cmd = self.cmd_rcv_mq => match next_cmd {
                Ok((_, cmd, origin)) => {
                    info!(target: "contract", "Contract {:?} handling command", self.id());
                    self.contract.handle_command(origin, cmd.0, &mut context)
                }
                Err(_e) => {
                    Err(TransactionError::ChannelError)
                }
            },
        }
    }

    pub(crate) fn on_block_end(&mut self, env: &mut ExecuteEnv) -> TransactionResult {
        let secret_mq = SecretMessageChannel::new(&self.ecdh_key, &self.send_mq);
        let mut context = NativeContext {
            block: env.block,
            mq: &self.send_mq,
            secret_mq,
            contract_clusters: &mut env.contract_clusters,
            self_id: self.id(),
        };
        self.contract.on_block_end(&mut context)
    }

    pub(crate) fn set_on_block_end_selector(&mut self, selector: u32) {
        if let AnyContract::Pink(pink) = &mut self.contract {
            pink.set_on_block_end_selector(selector)
        } else {
            log::error!("Can not set block_end_selector for native contract");
        }
    }

    pub(crate) fn push_message(&self, payload: Vec<u8>, topic: Vec<u8>) {
        self.send_mq.push_data(payload, topic)
    }

    pub(crate) fn push_osp_message(
        &self,
        payload: Vec<u8>,
        topic: Vec<u8>,
        remote_pubkey: Option<&EcdhPublicKey>,
    ) {
        let secret_mq = SecretMessageChannel::new(&self.ecdh_key, &self.send_mq);
        secret_mq
            .bind_remote_key(remote_pubkey)
            .push_data(payload, topic)
    }

    pub(crate) fn start_sidevm(
        &mut self,
        spawner: &sidevm::service::Spawner,
        code: Vec<u8>,
        memory_pages: u32,
    ) -> Result<()> {
        if self.sidevm_info.is_some() {
            bail!("Sidevm can only be started once");
        }
        let handle = do_start_sidevm(spawner, &code, memory_pages, self.contract_id.0)?;
        self.sidevm_info = Some(SidevmInfo {
            code,
            memory_pages,
            handle,
        });
        Ok(())
    }

    pub(crate) fn restart_sidevm_if_terminated(
        &mut self,
        spawner: &sidevm::service::Spawner,
    ) -> Result<()> {
        if let Some(sidevm_info) = &mut self.sidevm_info {
            if sidevm_info.handle.lock().unwrap().is_terminated() {
                let handle = do_start_sidevm(
                    spawner,
                    &sidevm_info.code,
                    sidevm_info.memory_pages,
                    self.cluster_id.0,
                )?;
                sidevm_info.handle = handle;
            }
        }
        Ok(())
    }

    pub(crate) fn push_message_to_sidevm(
        &self,
        spawner: &sidevm::service::Spawner,
        message: Vec<u8>,
    ) -> Result<()> {
        let handle = self
            .sidevm_info
            .as_ref()
            .ok_or_else(|| anyhow!("Push message to sidevm failed, no sidevm instance"))?
            .handle
            .clone();

        let tx = match &*handle.lock().unwrap() {
            SidevmHandle::Terminated => {
                error!(target: "sidevm", "Push message to sidevm failed, instance terminated");
                return Err(anyhow!(
                    "Push message to sidevm failed, instance terminated"
                ));
            }
            SidevmHandle::Running(tx) => tx.clone(),
        };
        spawner.spawn(async move {
            let result = tx
                .send(sidevm::service::Command::PushMessage(message))
                .await;
            if let Err(_) = result {
                error!(target: "sidevm", "Push message to sidevm failed, the vm might be already stopped");
            }
        });
        Ok(())
    }
}

fn do_start_sidevm(
    spawner: &sidevm::service::Spawner,
    code: &[u8],
    memory_pages: u32,
    id: VmId,
) -> Result<Arc<Mutex<SidevmHandle>>> {
    let todo = "connect the gas to some where";
    let gas = u128::MAX;
    let gas_per_breath = 1_000_000_000_000_u128; // about 1 sec
    let code = instrument(code).context("Failed to instrument the wasm code")?;
    let (sender, join_handle) = spawner.start(&code, memory_pages, id, gas, gas_per_breath)?;
    let handle = Arc::new(Mutex::new(SidevmHandle::Running(sender)));
    let cloned_handle = handle.clone();

    debug!(target: "sidevm", "Starting sidevm...");
    spawner.spawn(async move {
        if let Err(err) = join_handle.await {
            error!(target: "sidevm", "Sidevm process terminated with error: {:?}", err);
        }
        *cloned_handle.lock().unwrap() = SidevmHandle::Terminated;
    });
    Ok(handle)
}

pub use keeper::*;
mod keeper;

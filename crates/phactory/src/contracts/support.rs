use phala_crypto::ecdh::EcdhPublicKey;
use phala_mq::traits::MessageChannel;
use runtime::BlockNumber;
use serde::{Deserialize, Serialize};

use super::pink::cluster::ClusterKeeper;
use super::*;
use crate::secret_channel::SecretReceiver;
use crate::types::BlockInfo;
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
    sidevm_handle: Option<u64>,
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
            sidevm_handle: None,
        }
    }
}

impl FatContract {
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

    pub(crate) fn start_sidevm(&mut self, code: &[u8], memory_pages: u32) {
        todo!("TODO.kevin.start_sidevm");
    }
}

pub use keeper::*;
mod keeper;

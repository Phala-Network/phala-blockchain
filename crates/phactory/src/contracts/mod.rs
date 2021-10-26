use crate::secret_channel::{KeyPair, Peeler, PeelingReceiver, SecretMessageChannel};
use std::convert::TryFrom as _;
use std::fmt::Debug;

use crate::system::{TransactionError, TransactionResult};
use crate::types::{deopaque_query, OpaqueError, OpaqueQuery, OpaqueReply};
use anyhow::{Context, Error, Result};
use chain::AccountId;
use parity_scale_codec::{Decode, Encode};
use phala_mq::{MessageOrigin, SignedMessageChannel};

pub mod assets;
pub mod balances;
pub mod btc_lottery;
pub mod data_plaza;
// pub mod diem;
pub mod geolocation;
pub mod pink;
pub mod substrate_kitties;
pub mod web3analytics;

pub use phala_types::contract::*;

fn account_id_from_hex(s: &str) -> Result<AccountId> {
    let bytes = hex::decode(s)
        .map_err(Error::msg)
        .context("Failed to decode AccountId hex")?;
    AccountId::try_from(&bytes[..])
        .map_err(|err| anyhow::anyhow!("Failed to convert AccountId: {:?}", err))
}

pub use support::*;
mod support {
    use phala_crypto::ecdh::EcdhPublicKey;
    use phala_mq::traits::MessageChannel;
    use ::pink::runtime::ExecSideEffects;
    use runtime::BlockNumber;

    use super::pink::group::GroupKeeper;
    use super::*;
    use crate::types::BlockInfo;

    pub struct ExecuteEnv<'a> {
        pub block: &'a BlockInfo<'a>,
        pub contract_groups: &'a mut GroupKeeper,
    }

    pub struct NativeContext<'a> {
        pub block: &'a BlockInfo<'a>,
        pub mq: &'a SignedMessageChannel,
        pub secret_mq: SecretMessageChannel<'a, SignedMessageChannel>,
        pub contract_groups: &'a mut GroupKeeper,
    }

    pub struct QueryContext<'a> {
        pub block_number: BlockNumber,
        pub now_ms: u64,
        pub contract_groups: &'a mut GroupKeeper,
    }

    impl NativeContext<'_> {
        pub fn mq(&self) -> &SignedMessageChannel {
            self.mq
        }
    }

    pub trait Contract {
        fn id(&self) -> ContractId;
        fn handle_query(
            &mut self,
            origin: Option<&chain::AccountId>,
            req: OpaqueQuery,
            context: &mut QueryContext,
        ) -> Result<OpaqueReply, OpaqueError>;
        fn group_id(&self) -> Option<phala_mq::ContractGroupId>;
        fn process_next_message(&mut self, env: &mut ExecuteEnv) -> Option<TransactionResult>;
        fn on_block_end(&mut self, env: &mut ExecuteEnv) -> TransactionResult;
        fn push_message(&self, payload: Vec<u8>, topic: Vec<u8>);
        fn push_osp_message(
            &self,
            payload: Vec<u8>,
            topic: Vec<u8>,
            remote_pubkey: Option<&EcdhPublicKey>,
        );
    }

    pub trait NativeContract {
        type Cmd: Decode + Debug;
        type QReq: Decode + Debug;
        type QResp: Encode + Debug;

        fn id(&self) -> ContractId;
        fn group_id(&self) -> Option<phala_mq::ContractGroupId> {
            None
        }
        fn handle_command(
            &mut self,
            _origin: MessageOrigin,
            _cmd: Self::Cmd,
            _context: &mut NativeContext,
        ) -> TransactionResult {
            Ok(Default::default())
        }
        fn handle_query(
            &mut self,
            origin: Option<&chain::AccountId>,
            req: Self::QReq,
            context: &mut QueryContext,
        ) -> Self::QResp;
        fn on_block_end(&mut self, _context: &mut NativeContext) {}
    }

    pub struct NativeCompatContract<Con, Cmd, CmdWrp, CmdPlr, QReq, QResp>
    where
        Cmd: Decode + Debug,
        CmdWrp: Decode + Debug,
        CmdPlr: Peeler<Wrp = CmdWrp, Msg = Cmd>,
        QReq: Decode + Debug,
        QResp: Encode + Debug,
        Con: NativeContract<Cmd = Cmd, QReq = QReq, QResp = QResp>,
    {
        contract: Con,
        send_mq: SignedMessageChannel,
        cmd_rcv_mq: PeelingReceiver<Cmd, CmdWrp, CmdPlr>,
        ecdh_key: KeyPair,
    }

    impl<Con, Cmd, CmdWrp, CmdPlr, QReq, QResp>
        NativeCompatContract<Con, Cmd, CmdWrp, CmdPlr, QReq, QResp>
    where
        Cmd: Decode + Debug,
        CmdWrp: Decode + Debug,
        CmdPlr: Peeler<Wrp = CmdWrp, Msg = Cmd>,
        QReq: Decode + Debug,
        QResp: Encode + Debug,
        Con: NativeContract<Cmd = Cmd, QReq = QReq, QResp = QResp>,
        Cmd: Decode + Debug,
    {
        pub fn new(
            contract: Con,
            send_mq: SignedMessageChannel,
            cmd_rcv_mq: PeelingReceiver<Cmd, CmdWrp, CmdPlr>,
            ecdh_key: KeyPair,
        ) -> Self {
            NativeCompatContract {
                contract,
                send_mq,
                cmd_rcv_mq,
                ecdh_key,
            }
        }
    }

    impl<Con, Cmd, CmdWrp, CmdPlr, QReq, QResp> Contract
        for NativeCompatContract<Con, Cmd, CmdWrp, CmdPlr, QReq, QResp>
    where
        Cmd: Decode + Debug,
        CmdWrp: Decode + Debug,
        CmdPlr: Peeler<Wrp = CmdWrp, Msg = Cmd>,
        QReq: Decode + Debug,
        QResp: Encode + Debug,
        Con: NativeContract<Cmd = Cmd, QReq = QReq, QResp = QResp> + Send,
        Cmd: Decode + Debug,
    {
        fn id(&self) -> ContractId {
            self.contract.id()
        }

        fn group_id(&self) -> Option<phala_mq::ContractGroupId> {
            self.contract.group_id()
        }

        fn handle_query(
            &mut self,
            origin: Option<&runtime::AccountId>,
            req: OpaqueQuery,
            context: &mut QueryContext,
        ) -> Result<OpaqueReply, OpaqueError> {
            debug!(target: "contract", "Contract {:?} handling query", self.id());
            let response = self
                .contract
                .handle_query(origin, deopaque_query(req)?, context);
            Ok(response.encode())
        }

        fn process_next_message(&mut self, env: &mut ExecuteEnv) -> Option<TransactionResult> {
            let secret_mq = SecretMessageChannel::new(&self.ecdh_key, &self.send_mq);
            let mut context = NativeContext {
                block: env.block,
                mq: &self.send_mq,
                secret_mq,
                contract_groups: &mut env.contract_groups,
            };

            phala_mq::select! {
                next_cmd = self.cmd_rcv_mq => match next_cmd {
                    Ok((_, cmd, origin)) => {
                        info!(target: "contract", "Contract {:?} handling command", self.id());
                        self.contract.handle_command(origin, cmd, &mut context)
                    }
                    Err(e) => {
                        Err(TransactionError::ChannelError)
                    }
                },
            }
        }

        fn on_block_end(&mut self, env: &mut ExecuteEnv) -> TransactionResult {
            let secret_mq = SecretMessageChannel::new(&self.ecdh_key, &self.send_mq);
            let mut context = NativeContext {
                block: env.block,
                mq: &self.send_mq,
                secret_mq,
                contract_groups: &mut env.contract_groups,
            };
            self.contract.on_block_end(&mut context);
            Ok(Default::default())
        }

        fn push_message(&self, payload: Vec<u8>, topic: Vec<u8>) {
            self.send_mq.push_data(payload, topic)
        }

        fn push_osp_message(
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
    }
}

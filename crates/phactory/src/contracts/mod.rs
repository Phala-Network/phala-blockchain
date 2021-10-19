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
    use super::pink::group::GroupKeeper;
    use super::*;
    use crate::types::BlockInfo;

    pub struct ExecuteEnv<'a> {
        pub block: &'a BlockInfo<'a>,
        pub contract_groups: &'a mut GroupKeeper,
    }

    pub struct NativeContext<'a> {
        pub block: &'a BlockInfo<'a>,
        mq: &'a SignedMessageChannel,
        #[allow(unused)] // TODO.kevin: remove this.
        secret_mq: SecretMessageChannel<'a, SignedMessageChannel>,
        pub contract_groups: &'a mut GroupKeeper,
    }

    pub struct QueryContext<'a> {
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
        fn process_messages(&mut self, env: &mut ExecuteEnv);
    }

    pub trait NativeContract {
        type Cmd: Decode + Debug;
        type QReq: Decode + Debug;
        type QResp: Encode + Debug;

        fn id(&self) -> ContractId;
        fn handle_command(
            &mut self,
            _origin: MessageOrigin,
            _cmd: Self::Cmd,
            _context: &mut NativeContext,
        ) -> TransactionResult {
            Ok(())
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

        fn handle_query(
            &mut self,
            origin: Option<&runtime::AccountId>,
            req: OpaqueQuery,
            context: &mut QueryContext,
        ) -> Result<OpaqueReply, OpaqueError> {
            let response = self
                .contract
                .handle_query(origin, deopaque_query(req)?, context);
            Ok(response.encode())
        }

        fn process_messages(&mut self, env: &mut ExecuteEnv) {
            let secret_mq = SecretMessageChannel::new(&self.ecdh_key, &self.send_mq);
            let mut context = NativeContext {
                block: env.block,
                mq: &self.send_mq,
                secret_mq,
                contract_groups: &mut env.contract_groups,
            };
            loop {
                let ok = phala_mq::select! {
                    next_cmd = self.cmd_rcv_mq => match next_cmd {
                        Ok((_, cmd, origin)) => {
                            let status = self.contract.handle_command(origin, cmd, &mut context);
                            if let Err(err) = status {
                                log::error!("Contract {:?} handle command error: {:?}", self.id(), err);
                            }
                        }
                        Err(e) => {
                            error!("Read command failed [{}]: {:?}", self.id(), e);
                        }
                    },
                };
                if ok.is_none() {
                    break;
                }
            }
            self.contract.on_block_end(&mut context)
        }
    }
}

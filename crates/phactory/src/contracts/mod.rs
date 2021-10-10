use crate::secret_channel::{
    storage_prefix_for_topic_pubkey, KeyPair, Peeler, PeelingReceiver, SecretMessageChannel,
};
use std::fmt::Debug;
use std::convert::TryFrom as _;

use crate::system::{TransactionError, TransactionResult};
use crate::types::{deopaque_query, OpaqueError, OpaqueQuery, OpaqueReply};
use anyhow::{Context, Error, Result};
use chain::AccountId;
use parity_scale_codec::{Decode, Encode};
use phala_mq::{MessageOrigin, Sr25519MessageChannel as MessageChannel};

pub mod assets;
pub mod balances;
pub mod btc_lottery;
pub mod data_plaza;
// pub mod diem;
pub mod substrate_kitties;
pub mod web3analytics;
pub mod geolocation;

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
    use core::convert::TryInto;

    use super::*;
    use crate::types::BlockInfo;

    pub struct ExecuteEnv<'a> {
        pub block: &'a BlockInfo<'a>,
    }

    pub struct NativeContext<'a> {
        pub block: &'a BlockInfo<'a>,
        mq: &'a MessageChannel,
        #[allow(unused)] // TODO.kevin: remove this.
        secret_mq: SecretMessageChannel<'a>,
    }

    impl NativeContext<'_> {
        pub fn mq(&self) -> &MessageChannel {
            self.mq
        }
    }

    pub trait Contract {
        fn id(&self) -> ContractId;
        fn handle_query(
            &mut self,
            origin: Option<&chain::AccountId>,
            req: OpaqueQuery,
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
            _context: &NativeContext,
            _origin: MessageOrigin,
            _cmd: Self::Cmd,
        ) -> TransactionResult {
            Ok(())
        }
        fn handle_query(
            &mut self,
            origin: Option<&chain::AccountId>,
            req: Self::QReq,
        ) -> Self::QResp;
        fn on_block_end(
            &mut self,
            _context: &NativeContext,
        ) {}
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
        send_mq: MessageChannel,
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
            send_mq: MessageChannel,
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
        Cmd: Decode + Debug + Send + Sync,
        CmdWrp: Decode + Debug + Send + Sync,
        CmdPlr: Peeler<Wrp = CmdWrp, Msg = Cmd> + Send + Sync,
        QReq: Decode + Debug,
        QResp: Encode + Debug,
        Con: NativeContract<Cmd = Cmd, QReq = QReq, QResp = QResp> + Send + Sync,
        Cmd: Decode + Debug,
    {
        fn id(&self) -> ContractId {
            self.contract.id()
        }

        fn handle_query(
            &mut self,
            origin: Option<&runtime::AccountId>,
            req: OpaqueQuery,
        ) -> Result<OpaqueReply, OpaqueError> {
            let response = self.contract.handle_query(origin, deopaque_query(req)?);
            Ok(response.encode())
        }

        fn process_messages(&mut self, env: &mut ExecuteEnv) {
            let storage = env.block.storage;
            let key_map = |topic: &[u8]| {
                // TODO.kevin: query contract pubkey for contract topic's when the feature in GK is available.
                storage
                    .get(&storage_prefix_for_topic_pubkey(topic))
                    .map(|v| v.try_into().ok())
                    .flatten()
            };
            let secret_mq = SecretMessageChannel::new(&self.ecdh_key, &self.send_mq, &key_map);
            let context = NativeContext {
                block: env.block,
                mq: &self.send_mq,
                secret_mq,
            };
            loop {
                let ok = phala_mq::select! {
                    next_cmd = self.cmd_rcv_mq => match next_cmd {
                        Ok((_, cmd, origin)) => {
                            let _status = self.contract.handle_command(&context, origin, cmd);
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
            self.contract.on_block_end(&context)
        }
    }
}

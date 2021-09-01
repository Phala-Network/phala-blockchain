use crate::secret_channel::{
    storage_prefix_for_topic_pubkey, KeyPair, Peeler, PeelingReceiver, SecretMessageChannel,
};
use crate::std::fmt::Debug;
use crate::system::System;
use std::convert::TryFrom as _;

use crate::system::{TransactionError, TransactionResult};
use crate::types::{deopaque_query, OpaqueError, OpaqueQuery, OpaqueReply};
use anyhow::{Context, Error, Result};
use chain::AccountId;
use parity_scale_codec::{Decode, Encode};
use phala_mq::{MessageOrigin, Sr25519MessageChannel as MessageChannel};
use phala_crypto::ecdh::EcdhPublicKey;
use phala_types::messaging::CommandPayload;
use phala_enclave_api::crypto::EncryptedData;

pub mod assets;
pub mod balances;
pub mod btc_lottery;
pub mod data_plaza;
// pub mod diem;
pub mod substrate_kitties;
pub mod web3analytics;
pub mod woothee;
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
        pub system: &'a mut System,
    }

    pub struct NativeContext<'a> {
        pub block: &'a BlockInfo<'a>,
        mq: &'a MessageChannel,
        #[allow(unused)] // TODO.kevin: remove this.
        secret_mq: SecretMessageChannel<'a>,
        contract_key: &'a KeyPair,
    }

    impl NativeContext<'_> {
        pub fn mq(&self) -> &MessageChannel {
            self.mq
        }

        pub fn ecdh_decrypt<T>(&self, encrypted_data: Vec<u8>) -> Result<T> where T: Decode {
            let decoded_encrypted_data = match EncryptedData::decode(&mut &encrypted_data[..]) {
                Ok(e) => e,
                Err(_) => return Err(Error::msg("failed to decode encoded encrypted data")),
            };
            let encoded_data = match decoded_encrypted_data.decrypt(self.contract_key) {
                Ok(e) => e,
                Err(_) => return Err(Error::msg("failed to decrypt encrypted data")),
            };
            let data = match T::decode(&mut &encoded_data[..]) {
                Ok(e) => e,
                Err(_) => return Err(Error::msg("failed to decode encoded data")),
            };

            Ok(data)
        }
    }

    pub trait Contract {
        fn id(&self) -> ContractId;
        fn public_contract_ecdh_key(&self) -> EcdhPublicKey;
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

        fn id(&self) -> ContractId32;
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
        contract_ecdh_key: KeyPair,
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
            contract_ecdh_key: KeyPair,
        ) -> Self {
            NativeCompatContract {
                contract,
                send_mq,
                cmd_rcv_mq,
                ecdh_key,
                contract_ecdh_key,
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
            id256(self.contract.id())
        }

        fn public_contract_ecdh_key(&self) -> EcdhPublicKey {
            self.contract_ecdh_key.public()
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
                contract_key: &self.contract_ecdh_key,
            };
            loop {
                let ok = phala_mq::select! {
                    next_cmd = self.cmd_rcv_mq => match next_cmd {
                        Ok((_, cmd, origin)) => {
                            let _status = self.contract.handle_command(&context, origin, cmd);
                        }
                        Err(e) => {
                            error!("Read command failed: {:?}", e);
                        }
                    },
                };
                if ok.is_none() {
                    break;
                }
            }
        }
    }
}

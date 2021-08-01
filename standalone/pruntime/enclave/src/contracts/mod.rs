use crate::error_msg;
use crate::msg_channel::osp::{
    storage_prefix_for_topic_pubkey, KeyPair, OspMq, Peeler, PeelingReceiver,
};
use crate::std::fmt::Debug;
use crate::std::string::String;
use crate::system::System;
use crate::system::TransactionReceipt;

use super::TransactionStatus;
use crate::types::{deopaque_query, OpaqueError, OpaqueQuery, OpaqueReply};
use anyhow::{Context, Error, Result};
use core::{fmt, str};
use parity_scale_codec::{Decode, Encode};
use phala_mq::{BindTopic, MessageOrigin, Sr25519MessageChannel as MessageChannel};
use phala_types::messaging::PushCommand;

use serde::{
    de::{self, DeserializeOwned, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use sp_core::H256;
use sp_runtime_interface::pass_by::PassByInner as _;

pub mod assets;
pub mod balances;
pub mod btc_lottery;
pub mod data_plaza;
// TODO.kevin: take it back
// pub mod diem;
pub mod substrate_kitties;
pub mod web3analytics;
pub mod woothee;

pub type ContractId = u32;
pub const SYSTEM: ContractId = 0;
pub const DATA_PLAZA: ContractId = 1;
pub const BALANCES: ContractId = 2;
pub const ASSETS: ContractId = 3;
pub const WEB3_ANALYTICS: ContractId = 4;
pub const DIEM: ContractId = 5;
pub const SUBSTRATE_KITTIES: ContractId = 6;
pub const BTC_LOTTERY: ContractId = 7;

pub fn account_id_from_hex(accid_hex: &String) -> Result<chain::AccountId> {
    use core::convert::TryFrom;
    let bytes = hex::decode(accid_hex).map_err(Error::msg)?;
    chain::AccountId::try_from(bytes.as_slice()).map_err(|_| Error::msg("Bad account id"))
}

/// Serde moduele to serialize or deserialize Balance
pub mod serde_balance {
    use crate::std::str::FromStr;
    use crate::std::string::{String, ToString};
    use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &chain::Balance, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = value.to_string();
        String::serialize(&s, serializer)
    }
    pub fn deserialize<'de, D>(deserializer: D) -> Result<chain::Balance, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        chain::Balance::from_str(&s).map_err(de::Error::custom)
    }
}

/// Serde module to serialize or deserialize anyhow Errors
pub mod serde_anyhow {
    use crate::std::string::{String, ToString};
    use anyhow::Error;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &Error, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = value.to_string();
        String::serialize(&s, serializer)
    }
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Error, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Error::msg(s))
    }
}

#[derive(Default, Debug, Ord, PartialOrd, Eq, PartialEq, Clone, Encode, Decode)]
pub struct AccountIdWrapper(pub chain::AccountId);

impl AccountIdWrapper {
    fn try_from(b: &[u8]) -> Result<Self> {
        let mut a = AccountIdWrapper::default();
        use core::convert::TryFrom;
        a.0 = sp_core::crypto::AccountId32::try_from(b)
            .map_err(|_| Error::msg("Failed to parse AccountId32"))?;
        Ok(a)
    }
    fn from_hex(s: &str) -> Result<Self> {
        let bytes = hex::decode(s)
            .map_err(Error::msg)
            .context("Failed to decode AccountId hex")?;
        Self::try_from(&bytes)
    }
    fn to_string(&self) -> String {
        hex::encode(&self.0)
    }
}

impl From<H256> for AccountIdWrapper {
    fn from(hash: H256) -> Self {
        Self((*hash.inner()).into())
    }
}

impl Serialize for AccountIdWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let data_hex = self.to_string();
        serializer.serialize_str(&data_hex)
    }
}

impl<'de> Deserialize<'de> for AccountIdWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(AcidVisitor)
    }
}

struct AcidVisitor;

impl<'de> Visitor<'de> for AcidVisitor {
    type Value = AccountIdWrapper;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("AccountID is the hex of [u8;32]")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if v.len() == 64 {
            let bytes =
                hex::decode(v).map_err(|e| E::custom(format!("Cannot decode hex: {}", e)))?;
            AccountIdWrapper::try_from(&bytes)
                .map_err(|_| E::custom("Unknown error creating AccountIdWrapper"))
        } else {
            Err(E::custom(format!("AccountId hex length wrong: {}", v)))
        }
    }
}

pub use support::*;
mod support {
    use super::*;
    use crate::types::BlockInfo;

    pub struct ExecuteEnv<'a> {
        pub block: &'a BlockInfo<'a>,
        pub system: &'a mut System,
    }

    pub struct NativeContext<'a> {
        pub block: &'a BlockInfo<'a>,
        mq: &'a MessageChannel,
        osp_mq: OspMq<'a>,
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
        type Cmd: Decode + BindTopic + Debug;
        type Event: Decode + BindTopic + Debug;
        type QReq: Serialize + DeserializeOwned + Debug;
        type QResp: Serialize + DeserializeOwned + Debug;

        fn id(&self) -> ContractId;
        fn handle_command(
            &mut self,
            _context: &NativeContext,
            _origin: MessageOrigin,
            _cmd: PushCommand<Self::Cmd>,
        ) -> TransactionStatus {
            TransactionStatus::Ok
        }
        fn handle_event(
            &mut self,
            _context: &NativeContext,
            _origin: MessageOrigin,
            _event: Self::Event,
        ) {
        }
        fn handle_query(
            &mut self,
            origin: Option<&chain::AccountId>,
            req: Self::QReq,
        ) -> Self::QResp;
    }

    pub struct NativeCompatContract<
        Con,
        Cmd,
        CmdWrp,
        CmdPlr,
        Event,
        EventWrp,
        EventPlr,
        QReq,
        QResp,
    >
    where
        Cmd: Decode + BindTopic + Debug,
        CmdWrp: Decode + BindTopic + Debug,
        CmdPlr: Peeler<Wrp = CmdWrp, Msg = PushCommand<Cmd>>,
        Event: Decode + BindTopic + Debug,
        EventWrp: Decode + BindTopic + Debug,
        EventPlr: Peeler<Wrp = EventWrp, Msg = Event>,
        QReq: Serialize + DeserializeOwned + Debug,
        QResp: Serialize + DeserializeOwned + Debug,
        Con: NativeContract<Cmd = Cmd, Event = Event, QReq = QReq, QResp = QResp>,
    {
        contract: Con,
        send_mq: MessageChannel,
        cmd_rcv_mq: PeelingReceiver<PushCommand<Cmd>, CmdWrp, CmdPlr>,
        event_rcv_mq: PeelingReceiver<Event, EventWrp, EventPlr>,
        ecdh_key: KeyPair,
    }

    impl<Con, Cmd, CmdWrp, CmdPlr, Event, EventWrp, EventPlr, QReq, QResp>
        NativeCompatContract<Con, Cmd, CmdWrp, CmdPlr, Event, EventWrp, EventPlr, QReq, QResp>
    where
        Cmd: Decode + BindTopic + Debug,
        CmdWrp: Decode + BindTopic + Debug,
        CmdPlr: Peeler<Wrp = CmdWrp, Msg = PushCommand<Cmd>>,
        Event: Decode + BindTopic + Debug,
        EventWrp: Decode + BindTopic + Debug,
        EventPlr: Peeler<Wrp = EventWrp, Msg = Event>,
        QReq: Serialize + DeserializeOwned + Debug,
        QResp: Serialize + DeserializeOwned + Debug,
        Con: NativeContract<Cmd = Cmd, Event = Event, QReq = QReq, QResp = QResp>,
        PushCommand<Cmd>: Decode + BindTopic + Debug,
    {
        pub fn new(
            contract: Con,
            send_mq: MessageChannel,
            cmd_rcv_mq: PeelingReceiver<PushCommand<Cmd>, CmdWrp, CmdPlr>,
            event_rcv_mq: PeelingReceiver<Event, EventWrp, EventPlr>,
            ecdh_key: KeyPair,
        ) -> Self {
            NativeCompatContract {
                contract,
                send_mq,
                cmd_rcv_mq,
                event_rcv_mq,
                ecdh_key,
            }
        }
    }

    impl<Con, Cmd, CmdWrp, CmdPlr, Event, EventWrp, EventPlr, QReq, QResp> Contract
        for NativeCompatContract<Con, Cmd, CmdWrp, CmdPlr, Event, EventWrp, EventPlr, QReq, QResp>
    where
        Cmd: Decode + BindTopic + Debug + Send + Sync,
        CmdWrp: Decode + BindTopic + Debug + Send + Sync,
        CmdPlr: Peeler<Wrp = CmdWrp, Msg = PushCommand<Cmd>> + Send + Sync,
        Event: Decode + BindTopic + Debug + Send + Sync,
        EventWrp: Decode + BindTopic + Debug + Send + Sync,
        EventPlr: Peeler<Wrp = EventWrp, Msg = Event> + Send + Sync,
        QReq: Serialize + DeserializeOwned + Debug,
        QResp: Serialize + DeserializeOwned + Debug,
        Con: NativeContract<Cmd = Cmd, Event = Event, QReq = QReq, QResp = QResp> + Send + Sync,
        PushCommand<Cmd>: Decode + BindTopic + Debug,
    {
        fn id(&self) -> ContractId {
            self.contract.id()
        }

        fn handle_query(
            &mut self,
            origin: Option<&runtime::AccountId>,
            req: OpaqueQuery,
        ) -> Result<OpaqueReply, OpaqueError> {
            serde_json::to_value(
                self.contract.handle_query(
                    origin,
                    deopaque_query(req)
                        .map_err(|_| error_msg("Malformed request"))?
                        .request,
                ),
            )
            .map_err(|_| error_msg("serde_json serilize failed"))
        }

        fn process_messages(&mut self, env: &mut ExecuteEnv) {
            let storage = env.block.storage;
            let key_map =
                |topic: &phala_mq::Path| storage.get(&storage_prefix_for_topic_pubkey(topic));
            let osp_mq = OspMq::new(&self.ecdh_key, &self.send_mq, &key_map);
            let context = NativeContext {
                block: env.block,
                mq: &self.send_mq,
                osp_mq,
            };
            loop {
                let ok = phala_mq::select! {
                    next_cmd = self.cmd_rcv_mq => match next_cmd {
                        Ok((_, cmd, origin)) => {
                            let cmd_number = cmd.number;
                            let status = self.contract.handle_command(&context, origin.clone(), cmd);
                            env.system.add_receipt(
                                cmd_number,
                                TransactionReceipt {
                                    account: origin,
                                    block_num: env.block.block_number,
                                    contract_id: self.id(),
                                    status,
                                },
                            );
                        }
                        Err(e) => {
                            error!("Read command failed: {:?}", e);
                        }
                    },
                    next_event = self.event_rcv_mq => match next_event {
                        Ok((_, event, origin)) => {
                            self.contract.handle_event(&context, origin, event);
                        }
                        Err(e) => {
                            error!("Read event failed: {:?}", e);
                        },
                    },
                };
                if ok.is_none() {
                    break;
                }
            }
        }
    }
}

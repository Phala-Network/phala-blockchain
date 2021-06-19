use crate::error_msg;
use crate::std::fmt::Debug;
use crate::std::string::String;
use crate::system::System;
use crate::system::TransactionReceipt;

use super::TransactionStatus;
use crate::types::{deopaque_query, OpaqueError, OpaqueQuery, OpaqueReply, TxRef};
use anyhow::{Context, Error, Result};
use core::{fmt, str};
use parity_scale_codec::{Decode, Encode};
use phala_mq::{BindTopic, MessageDispatcher, MessageOrigin, TypedReceiveError, TypedReceiver};
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
pub mod diem;
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

pub trait LegacyContract<Cmd, QReq, QResp>
where
    Cmd: Serialize + DeserializeOwned + Debug,
    QReq: Serialize + DeserializeOwned + Debug,
    QResp: Serialize + DeserializeOwned + Debug,
{
    fn id(&self) -> ContractId;
    fn handle_command(
        &mut self,
        _origin: &chain::AccountId,
        _txref: &TxRef,
        _cmd: Cmd,
    ) -> TransactionStatus {
        TransactionStatus::Ok
    }

    fn handle_query(&mut self, origin: Option<&chain::AccountId>, req: QReq) -> QResp;
    fn handle_event(&mut self, _re: runtime::Event) {}
}

pub struct ExecuteEnv<'a> {
    pub block_number: chain::BlockNumber,
    pub system: &'a mut System,
}

pub trait Contract: Send + Sync {
    fn id(&self) -> ContractId;
    fn handle_query(
        &mut self,
        origin: Option<&chain::AccountId>,
        req: OpaqueQuery,
    ) -> Result<OpaqueReply, OpaqueError>;
    fn process_events(&mut self, env: &mut ExecuteEnv);
}

// TODO.kevin: move it to a seperate crate. Do we really need this type to distingush commands from events?
#[derive(Encode, Decode, Debug)]
pub struct PushCommand<Cmd> {
    pub command: Cmd,
    pub number: u64,
}

impl<Cmd: BindTopic> BindTopic for PushCommand<Cmd> {
    const TOPIC: &'static [u8] = <Cmd as BindTopic>::TOPIC;
}

pub trait NativeContract {
    type Cmd: Decode + BindTopic + Debug;
    type Event: Decode + BindTopic + Debug;
    type QReq: Serialize + DeserializeOwned + Debug;
    type QResp: Serialize + DeserializeOwned + Debug;

    fn id(&self) -> ContractId;
    fn handle_command(
        &mut self,
        _origin: MessageOrigin,
        _txref: &TxRef,
        _cmd: Self::Cmd,
    ) -> TransactionStatus {
        TransactionStatus::Ok
    }
    fn handle_event(&mut self, _origin: MessageOrigin, _event: Self::Event) {}
    fn handle_query(&mut self, origin: Option<&chain::AccountId>, req: Self::QReq) -> Self::QResp;
}

pub struct NativeCompatContract<Con, Cmd, Event, QReq, QResp>
where
    Cmd: Decode + BindTopic + Debug,
    Event: Decode + BindTopic + Debug,
    QReq: Serialize + DeserializeOwned + Debug,
    QResp: Serialize + DeserializeOwned + Debug,
    Con: NativeContract<Cmd = Cmd, Event = Event, QReq = QReq, QResp = QResp>,
{
    contract: Con,
    cmd_rcv_mq: TypedReceiver<PushCommand<Cmd>>,
    event_rcv_mq: TypedReceiver<Event>,
}

impl<Con, Cmd, Event, QReq, QResp> NativeCompatContract<Con, Cmd, Event, QReq, QResp>
where
    Cmd: Decode + BindTopic + Debug + Send + Sync,
    Event: Decode + BindTopic + Debug + Send + Sync,
    QReq: Serialize + DeserializeOwned + Debug,
    QResp: Serialize + DeserializeOwned + Debug,
    Con: NativeContract<Cmd = Cmd, Event = Event, QReq = QReq, QResp = QResp> + Send + Sync,
    PushCommand<Cmd>: Decode + BindTopic + Debug,
{
    pub fn new(contract: Con, dispatcher: &mut MessageDispatcher) -> Self {
        NativeCompatContract {
            contract,
            cmd_rcv_mq: dispatcher.subscribe_bound(),
            event_rcv_mq: dispatcher.subscribe_bound(),
        }
    }
}

impl<Con, Cmd, Event, QReq, QResp> Contract for NativeCompatContract<Con, Cmd, Event, QReq, QResp>
where
    Cmd: Decode + BindTopic + Debug + Send + Sync,
    Event: Decode + BindTopic + Debug + Send + Sync,
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

    fn process_events(&mut self, env: &mut ExecuteEnv) {
        // TODO.kevin: how about encrypted messages
        loop {
            let ok = phala_mq::select! {
                next_cmd = self.cmd_rcv_mq => match next_cmd {
                    Ok(Some((_seq, cmd, origin))) => {
                        let pos = TxRef {
                            blocknum: env.block_number,
                            index: cmd.number,
                        };
                        // TODO.kevin: allow all kind of origin to send commands to contract?
                        if let MessageOrigin::AccountId(id) = origin.clone() {
                            let status = self.contract.handle_command(origin, &pos, cmd.command);
                            env.system.add_receipt(
                                cmd.number,
                                TransactionReceipt {
                                    account: id.into(),
                                    block_num: pos.blocknum,
                                    contract_id: self.id(),
                                    status,
                                },
                            );
                        } else {
                            error!("Received command from invalid origin: {:?}", origin);
                        }
                    }
                    Ok(None) => {}
                    Err(e) => match e {
                        TypedReceiveError::CodecError(e) => {
                            error!("Decode command failed: {:?}", e);
                        }
                        TypedReceiveError::SenderGone => {
                            error!("Commnad queue sender has gone");
                        }
                    },
                },
                next_event = self.event_rcv_mq => match next_event {
                    Ok(Some((_seq, event, origin))) => {
                        self.contract.handle_event(origin, event);
                    }
                    Ok(None) => {}
                    Err(e) => match e {
                        TypedReceiveError::CodecError(e) => {
                            error!("Decode event failed: {:?}", e);
                        }
                        TypedReceiveError::SenderGone => {
                            error!("Event queue sender has gone");
                        }
                    },
                },
            };
            if ok.is_none() {
                break;
            }
        }
    }
}

pub fn account_id_from_hex(accid_hex: &String) -> Result<chain::AccountId> {
    use core::convert::TryFrom;
    let bytes = hex::decode(accid_hex).map_err(Error::msg)?;
    chain::AccountId::try_from(bytes.as_slice()).map_err(|_| Error::msg("Bad account id"))
}

/// Serde module to serialize or deserialize parity scale codec types
pub mod serde_scale {
    use crate::std::vec::Vec;
    use parity_scale_codec::{Decode, Encode};
    use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Encode,
    {
        let encoded: Vec<u8> = value.encode();
        Vec::<u8>::serialize(&encoded, serializer)
    }
    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: Decode,
    {
        let encoded = Vec::<u8>::deserialize(deserializer)?;
        T::decode(&mut &encoded[..]).map_err(de::Error::custom)
    }
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

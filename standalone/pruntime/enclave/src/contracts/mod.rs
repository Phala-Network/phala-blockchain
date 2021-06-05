use crate::std::fmt::Debug;
use crate::std::string::String;

use super::TransactionStatus;
use crate::types::TxRef;
use anyhow::{Context, Error, Result};
use core::{fmt, str};
use parity_scale_codec::{Decode, Encode};
use serde::{
    de::{self, DeserializeOwned, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};

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

pub trait Contract<Cmd, QReq, QResp>: Serialize + DeserializeOwned + Debug
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

impl<'a> AccountIdWrapper {
    fn try_from(b: &'a [u8]) -> Result<Self> {
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

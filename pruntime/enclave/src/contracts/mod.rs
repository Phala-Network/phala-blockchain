use crate::std::string::String;
use crate::std::fmt::Debug;

use core::{fmt,str};
use serde::{de::{self, Visitor, DeserializeOwned}, Serialize, Deserialize, Serializer, Deserializer};
use super::TransactionStatus;
use parity_scale_codec::{Encode, Decode};
use crate::types::TxRef;

pub mod data_plaza;
pub mod balance;
pub mod assets;
pub mod web3analytics;
pub mod helloworld;
pub mod secret_note;
pub mod woothee;

pub type ContractId = u32;
pub const SYSTEM: ContractId = 0;
pub const DATA_PLAZA: ContractId = 1;
pub const BALANCE: ContractId = 2;
pub const ASSETS: ContractId = 3;
pub const WEB3_ANALYTICS: ContractId = 4;
pub const HELLO_WORLD: ContractId = 5;
pub const SECRET_NOTE: ContractId = 6;

pub trait Contract<Cmd, QReq, QResp>: Serialize + DeserializeOwned + Debug
where
  Cmd: Serialize + DeserializeOwned + Debug,
  QReq: Serialize + DeserializeOwned + Debug,
  QResp: Serialize + DeserializeOwned + Debug
{
  fn id(&self) -> ContractId;
  fn handle_command(&mut self, origin: &chain::AccountId, txref: &TxRef, cmd: Cmd) -> TransactionStatus;
  fn handle_query(&mut self, origin: Option<&chain::AccountId>, req: QReq) -> QResp;
  fn handle_event(&mut self, _re: runtime::Event) {}
}

pub fn account_id_from_hex(accid_hex: &String) -> Result<chain::AccountId, ()> {
  use core::convert::TryFrom;
  let bytes = crate::hex::decode_hex(accid_hex);
  chain::AccountId::try_from(bytes.as_slice())
}

pub mod serde_balance {
  use crate::std::string::{String, ToString};
  use crate::std::str::FromStr;
  use serde::{de, Serialize, Deserialize, Serializer, Deserializer};

  pub fn serialize<S>(value: &chain::Balance, serializer: S) -> Result<S::Ok, S::Error>
  where S: Serializer {
    let s = value.to_string();
    String::serialize(&s, serializer)
  }
  pub fn deserialize<'de, D>(deserializer: D) -> Result<chain::Balance, D::Error>
  where D: Deserializer<'de> {
    let s = String::deserialize(deserializer)?;
    chain::Balance::from_str(&s).map_err(de::Error::custom)
  }
}

#[derive(Default, Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
#[derive(Encode, Decode)]
pub struct AccountIdWrapper( pub chain::AccountId );

impl<'a> AccountIdWrapper {
  fn from(b: &'a [u8]) -> Self {
    let mut a = AccountIdWrapper::default();
    use core::convert::TryFrom;
    a.0 = sp_core::crypto::AccountId32::try_from(b).unwrap();
    a
  }
  fn from_hex(s: &str) -> Self {
    let bytes = crate::hex::decode_hex(s);  // TODO: error handling
    AccountIdWrapper::from(&bytes)
  }
  fn to_string(&self) -> String {
    crate::hex::encode_hex_compact(self.0.as_ref())
  }
}

impl Serialize for AccountIdWrapper {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: Serializer,
  {
    let data_hex = self.to_string();
    serializer.serialize_str(&data_hex)
  }
}

impl<'de> Deserialize<'de> for AccountIdWrapper{
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where D: Deserializer<'de>
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
  where E: de::Error,
  {
    if v.len() == 64 {
      let bytes = crate::hex::decode_hex(v);  // TODO: error handling
      Ok(AccountIdWrapper::from(&bytes))
    } else {
      Err(E::custom(format!("AccountId hex length wrong: {}", v)))
    }
  }
}

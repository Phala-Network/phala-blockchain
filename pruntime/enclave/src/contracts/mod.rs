use crate::std::string::String;
use crate::std::fmt::Debug;

use serde::{de::DeserializeOwned, Serialize};

use crate::types::TxRef;

pub mod data_plaza;
pub mod balance;
pub mod assets;

pub type ContractId = u32;
pub const DATA_PLAZA: ContractId = 1;
pub const BALANCE: ContractId = 2;
pub const ASSETS: ContractId = 3;

pub trait Contract<Cmd, QReq, QResp>: Serialize + DeserializeOwned + Debug
where
  Cmd: Serialize + DeserializeOwned + Debug,
  QReq: Serialize + DeserializeOwned + Debug,
  QResp: Serialize + DeserializeOwned + Debug
{
  fn id(&self) -> ContractId;
  fn handle_command(&mut self, origin: &chain::AccountId, txref: &TxRef, cmd: Cmd);
  fn handle_query(&mut self, origin: Option<&chain::AccountId>, req: QReq) -> QResp;
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

use crate::std::string::String;
use crate::std::fmt::Debug;
use serde::{Serialize, Deserialize, de::DeserializeOwned};

use crate::cryptography::{AeadCipher, Origin};

extern crate runtime as chain;

// supportive

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TxRef {
  pub blocknum: chain::BlockNumber,
  pub index: u32,
  pub tx_hash: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Payload {
  Plain(String),
  Cipher(AeadCipher),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SignedQuery {
  pub query_payload: String,
  pub origin: Option<Origin>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Query<T> {
  pub contract_id: u32,
  pub nonce: u32,
  pub request: T
}
impl<T> Query<T> where T : Serialize + DeserializeOwned + Debug + Clone {}

pub type OpaqueQuery = Query<serde_json::Value>;
pub fn deopaque_query<T>(q: OpaqueQuery) -> Result<Query<T>, Error>
  where T: Serialize + DeserializeOwned + Debug + Clone {
  Ok(Query {
    contract_id: q.contract_id,
    nonce: q.nonce,
    request: serde_json::from_value(q.request).map_err(|_| Error::DecodeError)?
  })
}

pub enum Error {
  DecodeError,
}

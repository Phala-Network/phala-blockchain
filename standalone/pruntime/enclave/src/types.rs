use crate::std::fmt::Debug;
use crate::std::string::String;
use anyhow::Result;
use core::fmt;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::cryptography::{AeadCipher, Origin};

extern crate runtime as chain;

// supportive

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TxRef {
    pub blocknum: chain::BlockNumber,
    pub index: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Payload {
    Plain(String),
    Cipher(AeadCipher),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SignedQuery {
    pub query_payload: String,
    pub origin: Option<Origin>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Query<T> {
    pub contract_id: u32,
    pub nonce: u32,
    pub request: T,
}
impl<T> Query<T> where T: Serialize + DeserializeOwned + Debug + Clone {}

pub type OpaqueQuery = Query<serde_json::Value>;
pub fn deopaque_query<T>(q: OpaqueQuery) -> Result<Query<T>, Error>
where
    T: Serialize + DeserializeOwned + Debug + Clone,
{
    Ok(Query {
        contract_id: q.contract_id,
        nonce: q.nonce,
        request: serde_json::from_value(q.request).map_err(|_| Error::DecodeError)?,
    })
}

#[derive(Debug)]
pub enum Error {
    DecodeError,
    PersistentRuntimeNotFound,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::DecodeError => write!(f, "decode error"),
            Error::PersistentRuntimeNotFound => write!(f, "persistent runtime not found"),
        }
    }
}

use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec::Vec;
use scale::{Decode, Encode};
use serde::Deserialize;

mod era;
pub use era::Era;

#[derive(Deserialize, Encode, Clone, Debug, PartialEq)]
pub struct NextNonce<'a> {
    jsonrpc: &'a str,
    pub(crate) result: u64,
    id: u32,
}

#[derive(Encode, Decode, Clone, Debug)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct NextNonceOk {
    pub(crate) next_nonce: u64,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct RuntimeVersion<'a> {
    pub(crate) jsonrpc: &'a str,
    #[serde(borrow)]
    pub(crate) result: RuntimeVersionResult<'a>,
    pub(crate) id: u32,
}

#[derive(Deserialize, Encode, Clone, Debug, PartialEq)]
#[serde(bound(deserialize = "alloc::vec::Vec<(&'a str, u32)>: Deserialize<'de>"))]
pub struct RuntimeVersionResult<'a> {
    #[serde(alias = "specName")]
    pub(crate) spec_name: &'a str,
    #[serde(alias = "implName")]
    pub(crate) impl_name: &'a str,
    #[serde(alias = "authoringVersion")]
    pub(crate) authoring_version: u32,
    #[serde(alias = "specVersion")]
    pub(crate) spec_version: u32,
    #[serde(alias = "implVersion")]
    pub(crate) impl_version: u32,
    #[serde(borrow)]
    pub(crate) apis: Vec<(&'a str, u32)>,
    #[serde(alias = "transactionVersion")]
    pub(crate) transaction_version: u32,
    #[serde(alias = "stateVersion")]
    pub(crate) state_version: u32,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct RuntimeVersionOk {
    pub(crate) spec_name: String,
    pub(crate) impl_name: String,
    pub(crate) authoring_version: u32,
    pub(crate) spec_version: u32,
    pub(crate) impl_version: u32,
    pub(crate) apis: Vec<(String, u32)>,
    pub(crate) transaction_version: u32,
    pub(crate) state_version: u32,
}

#[derive(Deserialize, Encode, Clone, Debug, PartialEq)]
pub struct GenesisHash<'a> {
    pub(crate) jsonrpc: &'a str,
    pub(crate) result: &'a str,
    pub(crate) id: u32,
}

// TODO: handle the failure case
#[derive(Deserialize, Encode, Clone, Debug, PartialEq)]
pub struct TransactionResponse<'a> {
    pub(crate) jsonrpc: &'a str,
    pub(crate) result: &'a str,
    pub(crate) id: u32,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct ExtraParam {
    // 0 if Immortal, or Vec<u64, u64> for period and the phase.
    era: Era,
    // Tip for the block producer.
    tip: u128,
}

#[derive(Deserialize, Encode, Clone, Debug, PartialEq)]
pub struct GetStorageResponse<'a> {
    pub(crate) jsonrpc: &'a str,
    pub(crate) result: Option<&'a str>,
    pub(crate) id: u32,
}

/// Wraps an already encoded byte vector, prevents being encoded as a raw byte vector as part of
/// the transaction payload
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Encoded(pub Vec<u8>);

impl scale::Encode for Encoded {
    fn encode(&self) -> Vec<u8> {
        self.0.to_owned()
    }
}

use derive_more::FromStr;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};

#[derive(
    Serialize,
    Deserialize,
    Encode,
    Decode,
    Debug,
    Clone,
    PartialEq,
    Eq,
    TypeInfo,
    Hash,
    PartialOrd,
    Ord,
    FromStr,
)]
pub enum EndpointType {
    I2p = 0,
    Http,
}

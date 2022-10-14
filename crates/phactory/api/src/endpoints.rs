use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};

#[cfg(feature = "std")]
use derive_more::FromStr;

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
)]
#[cfg_attr(feature = "std", derive(FromStr))]
pub enum EndpointType {
    I2p = 0,
    Http,
}

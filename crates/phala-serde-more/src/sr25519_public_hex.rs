use serde::{de, Deserializer, Serializer};
use sp_core::sr25519;

use crate::scale_hex::HexBytesVisitor;

pub fn serialize<S: Serializer>(data: &sr25519::Public, ser: S) -> Result<S::Ok, S::Error> {
    ser.serialize_str(&hex::encode(&data.0))
}

pub fn deserialize<'de, De: Deserializer<'de>>(der: De) -> Result<sr25519::Public, De::Error> {
    let bytes = der.deserialize_bytes(HexBytesVisitor)?;
    Ok(sr25519::Public(bytes.try_into().or(Err(de::Error::custom("invalid public key")))?))
}

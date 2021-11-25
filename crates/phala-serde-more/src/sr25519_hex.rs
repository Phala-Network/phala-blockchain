use serde::{de, Deserializer, Serializer};
use sp_core::{sr25519, Pair};

use crate::scale_hex::HexBytesVisitor;

pub fn serialize<S: Serializer>(data: &sr25519::Pair, ser: S) -> Result<S::Ok, S::Error> {
    let bytes = data.as_ref().secret.to_bytes();
    ser.serialize_str(&hex::encode(&bytes))
}

pub fn deserialize<'de, De: Deserializer<'de>>(der: De) -> Result<sr25519::Pair, De::Error> {
    let bytes = der.deserialize_bytes(HexBytesVisitor)?;
    sr25519::Pair::from_seed_slice(&bytes).or(Err(de::Error::custom("invalid sr25519 key")))
}

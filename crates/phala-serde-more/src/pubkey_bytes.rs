use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sp_core::sr25519;

pub fn serialize<S: Serializer>(data: &sr25519::Public, ser: S) -> Result<S::Ok, S::Error> {
    data.0.serialize(ser)
}

pub fn deserialize<'de, De: Deserializer<'de>>(der: De) -> Result<sr25519::Public, De::Error> {
    let bytes = Deserialize::deserialize(der)?;
    Ok(sr25519::Public(bytes))
}

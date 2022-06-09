use alloc::vec::Vec;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use sp_core::{sr25519, Pair};

pub fn serialize<S: Serializer>(data: &Vec<sr25519::Pair>, ser: S) -> Result<S::Ok, S::Error> {
    let bytes: Vec<Vec<u8>> = data
        .iter()
        .map(|data| data.as_ref().secret.to_bytes().to_vec())
        .collect();
    bytes.serialize(ser)
}

pub fn deserialize<'de, De: Deserializer<'de>>(der: De) -> Result<Vec<sr25519::Pair>, De::Error> {
    let bytes: Vec<Vec<u8>> = Deserialize::deserialize(der)?;
    bytes
        .iter()
        .map(|bytes| {
            sr25519::Pair::from_seed_slice(&bytes).or(Err(de::Error::custom("invalid sr25519 key")))
        })
        .collect()
}

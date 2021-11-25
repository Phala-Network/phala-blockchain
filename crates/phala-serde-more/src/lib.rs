#![no_std]
extern crate alloc;

pub mod scale_hex;

#[cfg(feature = "crypto")]
pub mod sr25519_hex;

#[cfg(feature = "crypto")]
pub mod sr25519_public_hex;

pub mod todo {

    use serde::{de, Deserializer, Serializer};
    use sp_core::sr25519;

    use crate::scale_hex::HexBytesVisitor;

    pub fn serialize<S: Serializer, T>(data: &T, ser: S) -> Result<S::Ok, S::Error> {
        todo!("TODO.kevin.must")
    }

    pub fn deserialize<'de, De: Deserializer<'de>, T>(der: De) -> Result<T, De::Error> {
        todo!("TODO.kevin.must")
    }

}
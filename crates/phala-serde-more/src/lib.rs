#![no_std]
extern crate alloc;

pub mod scale_bytes;

#[cfg(feature = "crypto")]
pub mod key_bytes;

#[cfg(feature = "crypto")]
pub mod option_key_bytes;

#[cfg(feature = "crypto")]
pub mod pubkey_bytes;

pub mod todo {

    use serde::{Deserializer, Serializer};

    pub fn serialize<S: Serializer, T>(data: &T, ser: S) -> Result<S::Ok, S::Error> {
        todo!("TODO.kevin.must")
    }

    pub fn deserialize<'de, De: Deserializer<'de>, T>(der: De) -> Result<T, De::Error> {
        todo!("TODO.kevin.must")
    }

}
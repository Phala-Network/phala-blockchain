use serde::{
    de::{self, Visitor},
    Deserializer, Serializer,
};

pub struct HexBytesVisitor;
impl<'de> Visitor<'de> for HexBytesVisitor {
    type Value = alloc::vec::Vec<u8>;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("a hexadecimal byte array")
    }

    fn visit_str<E: de::Error>(self, s: &str) -> Result<Self::Value, E> {
        hex::decode(s).map_err(E::custom)
    }
}

pub fn serialize<S: Serializer, T: scale::Encode>(data: &T, ser: S) -> Result<S::Ok, S::Error> {
    let bytes = data.encode();
    ser.serialize_str(&hex::encode(&bytes))
}

pub fn deserialize<'de, De: Deserializer<'de>, T: scale::Decode>(der: De) -> Result<T, De::Error> {
    let bytes = der.deserialize_bytes(HexBytesVisitor)?;
    T::decode(&mut bytes.as_slice()).map_err(de::Error::custom)
}

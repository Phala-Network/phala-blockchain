use alloc::vec::Vec;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use sp_core::{sr25519, Pair};

pub fn serialize<S: Serializer>(data: &sr25519::Pair, ser: S) -> Result<S::Ok, S::Error> {
    let bytes = data.as_ref().secret.to_bytes().to_vec();
    bytes.serialize(ser)
}

pub fn deserialize<'de, De: Deserializer<'de>>(der: De) -> Result<sr25519::Pair, De::Error> {
    let bytes: Vec<u8> = Deserialize::deserialize(der)?;
    sr25519::Pair::from_seed_slice(&bytes).or(Err(de::Error::custom("invalid sr25519 key")))
}

#[test]
fn it_works() {
    use sp_core::Encode;
    use sp_core::sr25519::Pair;
    use sp_core::Pair as _;

    #[derive(Serialize, Deserialize)]
    struct TestData {
        #[serde(with = "crate::key_bytes")]
        pair: Pair,
    }

    let data = TestData {
        pair: Pair::from_seed_slice(&[0u8; 32]).unwrap(),
    };

    let serialized = serde_cbor::to_vec(&data).unwrap();
    let deserialized = serde_cbor::from_slice::<TestData>(&serialized).unwrap();
    assert_eq!(deserialized.pair.public(), data.pair.public());

    assert!(serde_cbor::from_slice::<TestData>(&[]).is_err());
    assert!(serde_cbor::from_slice::<TestData>(&b"foo".encode()).is_err());
}

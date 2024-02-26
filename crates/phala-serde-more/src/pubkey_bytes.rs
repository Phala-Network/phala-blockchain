use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sp_core::sr25519;

pub fn serialize<S: Serializer>(data: &sr25519::Public, ser: S) -> Result<S::Ok, S::Error> {
    data.0.serialize(ser)
}

pub fn deserialize<'de, De: Deserializer<'de>>(der: De) -> Result<sr25519::Public, De::Error> {
    let bytes = Deserialize::deserialize(der)?;
    Ok(sr25519::Public(bytes))
}

#[test]
fn it_works() {
    use sp_core::Encode;

    #[derive(Serialize, Deserialize)]
    struct TestData {
        #[serde(with = "crate::pubkey_bytes")]
        key: sr25519::Public,
    }

    let data = TestData {
        key: sr25519::Public([0u8; 32]),
    };

    let serialized = serde_cbor::to_vec(&data).unwrap();
    let deserialized = serde_cbor::from_slice::<TestData>(&serialized).unwrap();
    assert_eq!(deserialized.key, data.key);

    assert!(serde_cbor::from_slice::<TestData>(&[]).is_err());
    assert!(serde_cbor::from_slice::<TestData>(&b"foo".encode()).is_err());
}

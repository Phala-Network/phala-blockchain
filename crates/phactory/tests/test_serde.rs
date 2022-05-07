use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct V0 {
    a: u32,
    b: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct V1 {
    a: u32,
    #[serde(default)]
    c: u32,
    b: u32,
}

#[test]
fn test_versioning_rmp_serde() {
    let a = V0 { a: 42, b: 24 };

    let buf = rmp_serde::to_vec(&a).unwrap();
    let de: Result<V1, _> = rmp_serde::from_slice(&buf);
    insta::assert_debug_snapshot!(de);
}

#[test]
fn test_versioning_serde_cbor() {
    let a = V0 { a: 42, b: 24 };

    let buf = serde_cbor::to_vec(&a).unwrap();
    let de: Result<V1, _> = serde_cbor::from_slice(&buf);
    insta::assert_debug_snapshot!(de);
}

#[test]
fn test_versioning_ciborium() {
    let a = V0 { a: 42, b: 24 };

    let mut buf = Vec::new();
    ciborium::ser::into_writer(&a, &mut buf).unwrap();
    let de: Result<V1, _> = ciborium::de::from_reader(&*buf);
    insta::assert_debug_snapshot!(de);
}

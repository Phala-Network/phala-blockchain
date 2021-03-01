// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use serde::{de::DeserializeOwned, Deserialize};
use serde_reflection::{Format, FormatHolder, Samples, Tracer, TracerConfig};

#[derive(Deserialize)]
enum E {
    Unit,
}

#[derive(Deserialize)]
struct Unit;

#[derive(Deserialize)]
struct NewType(u64);

#[derive(Deserialize)]
struct Tuple(u64, u32);

#[derive(Deserialize)]
#[serde(rename = "FooStruct")]
#[allow(dead_code)]
struct Struct {
    a: u64,
}

fn test_type<T>(expected_name: &'static str)
where
    T: DeserializeOwned,
{
    // this crate
    assert_eq!(serde_name::trace_name::<T>(), Some(expected_name));

    // serde-reflection
    let mut tracer = Tracer::new(TracerConfig::default());
    let samples = Samples::new();
    let (mut ident, _samples) = tracer.trace_type::<T>(&samples).unwrap();
    ident.normalize().unwrap();
    assert_eq!(ident, Format::TypeName(expected_name.into()));
}

#[test]
fn test_serde_name_and_reflection() {
    test_type::<E>("E");
    test_type::<Unit>("Unit");
    test_type::<NewType>("NewType");
    test_type::<Tuple>("Tuple");
    test_type::<Struct>("FooStruct");

    assert_eq!(serde_name::trace_name::<u64>(), None);
    assert_eq!(serde_name::trace_name::<(E, E)>(), None);
}

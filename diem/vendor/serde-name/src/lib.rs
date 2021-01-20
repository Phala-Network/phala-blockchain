// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

#![forbid(unsafe_code)]

//! This crate provides a fast and reliable way to extract the Serde name of a Rust container.
//!
//! ```rust
//! # use serde::Deserialize;
//! # use serde_name::trace_name;
//! #[derive(Deserialize)]
//! struct Foo {
//!   bar: Bar,
//! }
//!
//! #[derive(Deserialize)]
//! #[serde(rename = "ABC")]
//! enum Bar { A, B, C }
//!
//! assert_eq!(trace_name::<Foo>(), Some("Foo"));
//! assert_eq!(trace_name::<Bar>(), Some("ABC"));
//! assert_eq!(trace_name::<Option<Bar>>(), None);
//! ```

#![cfg_attr(all(feature = "mesalock_sgx",
not(target_env = "sgx")), no_std)]
#![cfg_attr(all(target_env = "sgx", target_vendor = "mesalock"), feature(rustc_private))]

#[cfg(all(feature = "mesalock_sgx", not(target_env = "sgx")))]
#[macro_use]
extern crate sgx_tstd as std;

use serde::de::Visitor;
use thiserror::Error;

/// Compute the Serde name of a container.
pub fn trace_name<'de, T>() -> Option<&'static str>
where
    T: serde::de::Deserialize<'de>,
{
    match T::deserialize(SerdeName) {
        Err(SerdeNameError(name)) => name,
        _ => unreachable!(),
    }
}

/// Minimal instrumented implementation of `serde::de::Deserializer`
/// This always returns a `SerdeNameError` as soon as we have learnt the name
/// of the type (or the absence of name) from Serde.
struct SerdeName;

/// Custom error value used to report the result of the analysis.
#[derive(Clone, Debug, Error, PartialEq)]
#[error("{0:?}")]
struct SerdeNameError(Option<&'static str>);

impl serde::de::Error for SerdeNameError {
    fn custom<T: std::fmt::Display>(_msg: T) -> Self {
        unreachable!();
    }
}

macro_rules! declare_deserialize {
    ($method:ident) => {
        fn $method<V>(self, _visitor: V) -> std::result::Result<V::Value, SerdeNameError>
        where
            V: Visitor<'de>,
        {
            Err(SerdeNameError(None))
        }
    };
}

impl<'de> serde::de::Deserializer<'de> for SerdeName {
    type Error = SerdeNameError;

    declare_deserialize!(deserialize_any);
    declare_deserialize!(deserialize_identifier);
    declare_deserialize!(deserialize_ignored_any);
    declare_deserialize!(deserialize_bool);
    declare_deserialize!(deserialize_i8);
    declare_deserialize!(deserialize_i16);
    declare_deserialize!(deserialize_i32);
    declare_deserialize!(deserialize_i64);
    declare_deserialize!(deserialize_i128);
    declare_deserialize!(deserialize_u8);
    declare_deserialize!(deserialize_u16);
    declare_deserialize!(deserialize_u32);
    declare_deserialize!(deserialize_u64);
    declare_deserialize!(deserialize_u128);
    declare_deserialize!(deserialize_f32);
    declare_deserialize!(deserialize_f64);
    declare_deserialize!(deserialize_char);
    declare_deserialize!(deserialize_str);
    declare_deserialize!(deserialize_string);
    declare_deserialize!(deserialize_bytes);
    declare_deserialize!(deserialize_byte_buf);
    declare_deserialize!(deserialize_option);
    declare_deserialize!(deserialize_unit);
    declare_deserialize!(deserialize_seq);
    declare_deserialize!(deserialize_map);

    fn deserialize_tuple<V>(
        self,
        _len: usize,
        _visitor: V,
    ) -> std::result::Result<V::Value, SerdeNameError>
    where
        V: Visitor<'de>,
    {
        Err(SerdeNameError(None))
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        _visitor: V,
    ) -> std::result::Result<V::Value, SerdeNameError>
    where
        V: Visitor<'de>,
    {
        Err(SerdeNameError(Some(name)))
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        _visitor: V,
    ) -> std::result::Result<V::Value, SerdeNameError>
    where
        V: Visitor<'de>,
    {
        Err(SerdeNameError(Some(name)))
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> std::result::Result<V::Value, SerdeNameError>
    where
        V: Visitor<'de>,
    {
        Err(SerdeNameError(Some(name)))
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> std::result::Result<V::Value, SerdeNameError>
    where
        V: Visitor<'de>,
    {
        Err(SerdeNameError(Some(name)))
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> std::result::Result<V::Value, SerdeNameError>
    where
        V: Visitor<'de>,
    {
        Err(SerdeNameError(Some(name)))
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

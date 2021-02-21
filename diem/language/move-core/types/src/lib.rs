// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Core types for Move.

#![cfg_attr(all(feature = "mesalock_sgx", not(target_env = "sgx")), no_std)]
#![cfg_attr(all(target_env = "sgx", target_vendor = "mesalock"), feature(rustc_private))]

#[cfg(all(feature = "mesalock_sgx", not(target_env = "sgx")))]
#[macro_use]
extern crate sgx_tstd as std;

pub mod account_address;
pub mod gas_schedule;
pub mod identifier;
pub mod language_storage;
pub mod move_resource;
pub mod parser;
// #[cfg(any(test, feature = "fuzzing"))]
// pub mod proptest_types;
pub mod transaction_argument;
#[cfg(test)]
mod unit_tests;
pub mod value;
pub mod vm_status;

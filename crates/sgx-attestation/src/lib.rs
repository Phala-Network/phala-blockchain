#![cfg_attr(not(test), no_std)]

#[macro_use]
extern crate alloc;

#[derive(Debug)]
pub enum Error {
    InvalidCertificate,
    InvalidSignature,
    CodecError,
}

pub mod ias;
pub mod dcap;

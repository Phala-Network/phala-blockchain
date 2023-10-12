#![cfg_attr(not(test), no_std)]

extern crate alloc;

#[derive(Debug)]
pub enum Error {
    InvalidCertificate,
    InvalidSignature,
    CodecError,
}

pub mod ias;

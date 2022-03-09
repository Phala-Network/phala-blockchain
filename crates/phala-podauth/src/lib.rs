#![no_std]

extern crate alloc;

use alloc::{string::String, vec::Vec};
use parity_scale_codec::{Decode, Encode};

#[derive(Encode, Decode, Debug)]
pub enum Request {
    Auth {
        /// The sgx quote
        quote: Vec<u8>,
    },
    GetRootCert,
}

#[derive(Encode, Decode, Debug)]
pub enum Response {
    Auth {
        /// Issue a certificate to the pod, in pem format
        cert: String,
        /// The private key of the certificate, in pem format
        key: String,
    },
    RootCert {
        /// The root certificate, in pem format
        cert: String,
    },
}

#[derive(Encode, Decode, Debug)]
pub enum QueryError {
    InvalidQuote,
    MakeCertFailed,
}

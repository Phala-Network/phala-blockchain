#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use parity_scale_codec::{Decode, Encode};

#[derive(Encode, Decode)]
pub struct AuthRequest {
    /// The sgx quote
    pub quote: Vec<u8>,
}

#[derive(Encode, Decode)]
pub struct AuthResponse {
    /// Issue a certificate to the pod, in pem format
    pub cert: Vec<u8>,
    /// The private key of the certificate, in pem format
    pub key: Vec<u8>,
}

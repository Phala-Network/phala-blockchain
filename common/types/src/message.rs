#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use alloc::vec::Vec;
use codec::{Decode, Encode};
use sp_core::ecdsa;
use sp_core::crypto::Pair;
use sp_core::ecdsa::Signature;
#[cfg(feature = "enable_serde")]
use serde::{Deserialize, Serialize};

pub trait Message: Encode + Decode + Clone  {}
impl<T: Encode + Decode + Clone > Message for T {}


#[derive(Debug,Default)]
#[cfg_attr(feature = "enable_serde", derive(Serialize, Deserialize))]
pub struct SignedMessage<M: Message> {
    pub data: M,
    pub sequence: u64,
    pub signature: Vec<u8>,
}


#[derive(Encode,Decode,Default)]
#[cfg_attr(feature = "enable_serde", derive(Serialize, Deserialize))]
pub struct EncodeMsg<M:Message>{
    pub data:M,
    pub sequence:u64
}
impl<M: Message> EncodeMsg<M> {
    pub fn new(data: M, sequence: u64) -> Self {
        let data_clone = data.clone();
        Self {
            data:data_clone,
            sequence
        }
    }
}
impl<M: Message> SignedMessage<M> {
    pub fn new(data: M, sequence: u64, keypair: &ecdsa::Pair) -> Self {
        let data_clone = data.clone();
        Self {
            data,
            sequence,
            signature: Self::sign(sequence,data_clone, &keypair),
        }
    }
    pub fn sign(sequence:u64,data: M, keypair: &ecdsa::Pair) -> Vec<u8> {
        
        let encode_msg = EncodeMsg{data,sequence};
        let encode_value = &Encode::encode(&encode_msg);
        let signature = keypair.sign(encode_value);
        signature.0.to_vec()
    }
    pub fn verify(&self, keypair: &ecdsa::Pair, sequence: u64, data: M, vec_sig: Vec<u8>) -> bool {
        let encode_msg = EncodeMsg { data, sequence };
        let message = encode_msg.encode();
        let public = keypair.public();
        let mut sig_raw:[u8;65] = [0;65];
        let mut i = 0;
        for k in vec_sig {
            sig_raw[i] = k;
            i = i +1;
        }
        let signature: Signature = Signature::from_raw(sig_raw);
        ecdsa::Pair::verify(&signature, message, &public)
    }
}
    



    
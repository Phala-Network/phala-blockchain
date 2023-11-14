use alloc::vec::Vec;
use scale::{Decode, Encode};
use scale_info::TypeInfo;

#[derive(Debug, Clone, Encode, Decode, TypeInfo, PartialEq, Eq)]
pub struct QuoteCollateral {
    pub pck_crl_issuer_chain: Vec<u8>,
    pub root_ca_crl: Vec<u8>,
    pub pck_crl: Vec<u8>,
    pub tcb_info_issuer_chain: Vec<u8>,
    pub tcb_info: Vec<u8>,
    pub qe_identity_issuer_chain: Vec<u8>,
    pub qe_identity: Vec<u8>,
}

#[cfg(feature = "std")]
pub mod get_collateral;

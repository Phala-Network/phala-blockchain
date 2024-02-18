use alloc::string::String;
use alloc::vec::Vec;
use scale::{Decode, Encode};
use scale_info::TypeInfo;

#[derive(Encode, Decode, TypeInfo, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttestationType {
    Epid,
    Dcap,
}

#[derive(Encode, Decode, TypeInfo, Debug, Clone, PartialEq, Eq)]
pub struct SgxQuote {
    pub attestation_type: AttestationType,
    pub quote: Vec<u8>,
}

#[derive(Encode, Decode, TypeInfo, Debug, Clone, PartialEq, Eq)]
pub enum AttestationReport {
    SgxIas {
        ra_report: Vec<u8>,
        signature: Vec<u8>,
        raw_signing_cert: Vec<u8>,
    },
    SgxDcap {
        quote: Vec<u8>,
        collateral: Option<Collateral>,
    },
}

#[derive(Encode, Decode, TypeInfo, Debug, Clone, PartialEq, Eq)]
pub enum Collateral {
    SgxV30(SgxV30QuoteCollateral),
}

#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, Debug)]
pub struct SgxV30QuoteCollateral {
    pub pck_crl_issuer_chain: String,
    pub root_ca_crl: String,
    pub pck_crl: String,
    pub tcb_info_issuer_chain: String,
    pub tcb_info: String,
    pub tcb_info_signature: Vec<u8>,
    pub qe_identity_issuer_chain: String,
    pub qe_identity: String,
    pub qe_identity_signature: Vec<u8>,
}

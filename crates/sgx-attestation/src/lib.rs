#![cfg_attr(not(test), no_std)]

#[macro_use]
extern crate alloc;

use alloc::string::String;
use scale::{Decode, Encode};
use scale_info::TypeInfo;

#[derive(Encode, Decode, TypeInfo, Debug, Clone, PartialEq, Eq)]
pub enum Error {
    InvalidCertificate,
    InvalidSignature,
    CodecError,

    // DCAP
    RawDataInvalid,
    MissingField { field: String },
    InvalidFieldValue { field: String },
    UnsupportedFieldValue { field: String },
    TCBInfoExpired,
    KeyLengthIsInvalid,
    PublicKeyIsInvalid,
    RsaSignatureIsInvalid,
    DerEncodingError,
    UnknownMREnclave,
    UnknownMRSigner,
    UnsupportedDCAPQuoteVersion,
    UnsupportedDCAPAttestationKeyType,
    UnsupportedQuoteAuthData,
    UnsupportedDCAPPckCertFormat,
    LeafCertificateParsingError,
    CertificateChainIsInvalid,
    CertificateChainIsTooShort,
    IntelExtensionCertificateDecodingError,
    IntelExtensionAmbiguity,
    CpuSvnOidIsMissing,
    CpuSvnLengthMismatch,
    CpuSvnDecodingError,
    PceSvnOidIsMissing,
    PceSvnDecodingError,
    PceSvnLengthMismatch,
    FmspcOidIsMissing,
    FmspcLengthMismatch,
    FmspcDecodingError,
    FmspcMismatch,
    QEReportHashMismatch,
    IsvEnclaveReportSignatureIsInvalid,
}

pub mod ias;
pub mod dcap;

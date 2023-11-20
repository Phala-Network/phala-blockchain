#![cfg_attr(not(test), no_std)]

#[macro_use]
extern crate alloc;

use alloc::string::String;

#[derive(Debug)]
pub enum Error {
    InvalidCertificate,
    InvalidSignature,
    CodecError,

    // DCAP
    RawDataInvalid,
    MissingField { field: String },
    InvalidFieldValue { field: String },
    UnsupportedFieldValue { field: String },
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
    QEReportHashMismatch,
    IsvEnclaveReportSignatureIsInvalid,
}

pub mod ias;
pub mod dcap;

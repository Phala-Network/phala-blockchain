#![cfg_attr(all(not(test), not(feature = "std")), no_std)]

#[macro_use]
extern crate alloc;

use scale::{Decode, Encode};
use scale_info::TypeInfo;

pub mod dcap;
pub mod ias;

#[cfg(feature = "report")]
pub mod gramine;

#[derive(Encode, Decode, TypeInfo, Debug, Clone, PartialEq, Eq)]
pub enum Error {
    InvalidCertificate,
    InvalidSignature,
    CodecError,

    // DCAP
    TCBInfoExpired,
    KeyLengthIsInvalid,
    PublicKeyIsInvalid,
    RsaSignatureIsInvalid,
    DerEncodingError,
    UnsupportedDCAPQuoteVersion,
    UnsupportedDCAPAttestationKeyType,
    UnsupportedQuoteAuthData,
    UnsupportedDCAPPckCertFormat,
    LeafCertificateParsingError,
    CertificateChainIsInvalid,
    CertificateChainIsTooShort,
    IntelExtensionCertificateDecodingError,
    IntelExtensionAmbiguity,
    CpuSvnLengthMismatch,
    CpuSvnDecodingError,
    PceSvnDecodingError,
    PceSvnLengthMismatch,
    FmspcLengthMismatch,
    FmspcDecodingError,
    FmspcMismatch,
    QEReportHashMismatch,
    IsvEnclaveReportSignatureIsInvalid,
    DerDecodingError,
    OidIsMissing,
}

#[derive(Encode, Decode, TypeInfo, Debug, Clone, PartialEq, Eq)]
pub enum AttestationType {
    Epid,
    Dcap,
}

pub use pink_types::sgx::{AttestationReport, Collateral, SgxV30QuoteCollateral};

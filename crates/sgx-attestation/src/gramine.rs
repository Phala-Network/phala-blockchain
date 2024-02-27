use std::fs;
use std::io::Result;

use super::{AttestationType, SgxQuote};

/// Create an SGX quote from the given data.
pub fn create_quote_vec(data: &[u8]) -> Result<Vec<u8>> {
    fs::write("/dev/attestation/user_report_data", data)?;
    fs::read("/dev/attestation/quote")
}

/// Create an SGX quote from the given data.
pub fn create_quote(data: &[u8]) -> Option<SgxQuote> {
    let quote = create_quote_vec(data).ok()?;
    let attestation_type = attestation_type()?;
    Some(SgxQuote {
        attestation_type,
        quote,
    })
}

/// Get the attestation type of the current running gramine instance.
///
/// Possible values are "epid", "dcap" or None if the file does not exist.
pub fn attestation_type_str() -> Option<String> {
    fs::read_to_string("/dev/attestation/attestation_type").ok()
}

/// Get the attestation type of the current running gramine instance.
pub fn attestation_type() -> Option<AttestationType> {
    attestation_type_str().and_then(|s| match s.as_str() {
        "epid" => Some(AttestationType::Epid),
        "dcap" => Some(AttestationType::Dcap),
        _ => None,
    })
}

/// Returns true if the current running gramine instance is using DCAP.
pub fn is_dcap() -> bool {
    attestation_type() == Some(AttestationType::Dcap)
}

/// Returns true if the current process is running inside a gramine enclave.
pub fn is_in_enclave() -> bool {
    std::path::Path::new("/dev/attestation/attestation_type").exists()
}

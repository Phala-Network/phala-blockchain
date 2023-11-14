#![allow(dead_code)]
pub const TEE_TYPE_SGX: u32 = 0x00000000;
pub const TEE_TYPE_TDX: u32 = 0x00000081;

pub const ECDSA_256_WITH_P256_CURVE: u16 = 2;
pub const ECDSA_384_WITH_P384_CURVE: u16 = 3;
pub const ECDSA_P256_SIGNATURE_BYTE_LEN: usize = 64;
pub const BODY_BYTE_SIZE: usize = 6;
pub const BODY_SGX_ENCLAVE_REPORT_TYPE: u16 = 1;
pub const BODY_TD_REPORT10_TYPE: u16 = 2;
pub const BODY_TD_REPORT15_TYPE: u16 = 3;
pub const ENCLAVE_REPORT_BYTE_LEN: usize = 384;
pub const TD_REPORT10_BYTE_LEN: usize = 584;
pub const TD_REPORT15_BYTE_LEN: usize = 648;

pub const PCK_ID_PLAIN_PPID: u16 = 1;
pub const PCK_ID_ENCRYPTED_PPID_2048: u16 = 2;
pub const PCK_ID_ENCRYPTED_PPID_3072: u16 = 3;
pub const PCK_ID_PCK_CERTIFICATE: u16 = 4;
pub const PCK_ID_PCK_CERT_CHAIN: u16 = 5;
pub const PCK_ID_QE_REPORT_CERTIFICATION_DATA: u16 = 6;

pub const ALLOWED_QUOTE_VERSIONS: [u16; 3] = [3, 4, 5];
pub const ALLOWED_BODY_TYPES: [u16; 3] = [
    BODY_SGX_ENCLAVE_REPORT_TYPE,
    BODY_TD_REPORT10_TYPE,
    BODY_TD_REPORT15_TYPE,
];
pub const ALLOWED_TEE_TYPES: [u32; 2] = [TEE_TYPE_SGX, TEE_TYPE_TDX];
pub const ALLOWED_ATTESTATION_KEY_TYPES: [u16; 1] = [ECDSA_256_WITH_P256_CURVE];
pub const INTEL_QE_VENDOR_ID: [u8; 16] = [
    0x93, 0x9A, 0x72, 0x33, 0xF7, 0x9C, 0x4C, 0xA9, 0x94, 0x0A, 0x0D, 0xB3, 0x95, 0x7F, 0x06, 0x07,
];
pub const HEADER_BYTE_LEN: usize = 48;
pub const AUTH_DATA_SIZE_BYTE_LEN: usize = 4;

pub const ECDSA_SIGNATURE_BYTE_LEN: usize = 64;
pub const ECDSA_PUBKEY_BYTE_LEN: usize = 64;
pub const QE_REPORT_BYTE_LEN: usize = ENCLAVE_REPORT_BYTE_LEN;
pub const QE_REPORT_SIG_BYTE_LEN: usize = ECDSA_SIGNATURE_BYTE_LEN;
pub const CERTIFICATION_DATA_TYPE_BYTE_LEN: usize = 2;
pub const CERTIFICATION_DATA_SIZE_BYTE_LEN: usize = 4;
pub const QE_AUTH_DATA_SIZE_BYTE_LEN: usize = 2;
pub const QE_CERT_DATA_TYPE_BYTE_LEN: usize = 2;
pub const QE_CERT_DATA_SIZE_BYTE_LEN: usize = 4;

pub const AUTH_DATA_MIN_BYTE_LEN: usize = ECDSA_SIGNATURE_BYTE_LEN
    + ECDSA_PUBKEY_BYTE_LEN
    + QE_REPORT_BYTE_LEN
    + QE_REPORT_SIG_BYTE_LEN
    + QE_AUTH_DATA_SIZE_BYTE_LEN
    + QE_CERT_DATA_TYPE_BYTE_LEN
    + QE_CERT_DATA_SIZE_BYTE_LEN;

pub const QUOTE_MIN_BYTE_LEN: usize =
    // Actual minimal size is a Quote V3 with Enclave report
    HEADER_BYTE_LEN
        + ENCLAVE_REPORT_BYTE_LEN
        + AUTH_DATA_SIZE_BYTE_LEN
        + AUTH_DATA_MIN_BYTE_LEN;

pub mod oids {
    use const_oid::ObjectIdentifier as OID;

    const fn oid(s: &str) -> OID {
        OID::new_unwrap(s)
    }

    pub const SGX_EXTENSION: OID = oid("1.2.840.113741.1.13.1");
    pub const TCB: OID = oid("1.2.840.113741.1.13.1.2");
    pub const PPID: OID = oid("1.2.840.113741.1.13.1.1");
    pub const PCEID: OID = oid("1.2.840.113741.1.13.1.3");
    pub const FMSPC: OID = oid("1.2.840.113741.1.13.1.4");
    pub const SGX_TYPE: OID = oid("1.2.840.113741.1.13.1.5"); // ASN1 Enumerated
    pub const PLATFORM_INSTANCE_ID: OID = oid("1.2.840.113741.1.13.1.6");
    pub const CONFIGURATION: OID = oid("1.2.840.113741.1.13.1.7");
}

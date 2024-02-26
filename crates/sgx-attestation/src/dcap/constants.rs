#![allow(dead_code)]

pub type MrSigner = [u8; 32];
pub type MrEnclave = [u8; 32];
pub type Fmspc = [u8; 6];
pub type CpuSvn = [u8; 16];
pub type Svn = u16;

pub const QUOTE_VERSION_V3: u16 = 3;
pub const ATTESTATION_KEY_TYPE_ECDSA256_WITH_P256_CURVE: u16 = 2;
pub const ATTESTATION_KEY_TYPE_ECDSA484_WITH_P384_CURVE: u16 = 3;

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

pub const ATTESTATION_KEY_LEN: usize = 64;
pub const AUTHENTICATION_DATA_LEN: usize = 32;
pub const QE_HASH_DATA_BYTE_LEN: usize = ATTESTATION_KEY_LEN + AUTHENTICATION_DATA_LEN;

/// The needed code for a trust anchor can be extracted using `webpki` with something like this:
/// println!("{:?}", webpki::TrustAnchor::try_from_cert_der(&root_cert));
#[allow(clippy::zero_prefixed_literal)]
pub static DCAP_SERVER_ROOTS: &[webpki::types::TrustAnchor<'static>; 1] =
    &[webpki::types::TrustAnchor {
        subject: webpki::types::Der::from_slice(&[
            49, 26, 48, 24, 06, 03, 85, 04, 03, 12, 17, 73, 110, 116, 101, 108, 32, 83, 71, 88, 32,
            82, 111, 111, 116, 32, 67, 65, 49, 26, 48, 24, 06, 03, 85, 04, 10, 12, 17, 73, 110,
            116, 101, 108, 32, 67, 111, 114, 112, 111, 114, 97, 116, 105, 111, 110, 49, 20, 48, 18,
            06, 03, 85, 04, 07, 12, 11, 83, 97, 110, 116, 97, 32, 67, 108, 97, 114, 97, 49, 11, 48,
            09, 06, 03, 85, 04, 08, 12, 02, 67, 65, 49, 11, 48, 09, 06, 03, 85, 04, 06, 19, 02, 85,
            83,
        ]),
        subject_public_key_info: webpki::types::Der::from_slice(&[
            48, 19, 06, 07, 42, 134, 72, 206, 61, 02, 01, 06, 08, 42, 134, 72, 206, 61, 03, 01, 07,
            03, 66, 00, 04, 11, 169, 196, 192, 192, 200, 97, 147, 163, 254, 35, 214, 176, 44, 218,
            16, 168, 187, 212, 232, 142, 72, 180, 69, 133, 97, 163, 110, 112, 85, 37, 245, 103,
            145, 142, 46, 220, 136, 228, 13, 134, 11, 208, 204, 78, 226, 106, 172, 201, 136, 229,
            05, 169, 83, 85, 140, 69, 63, 107, 09, 04, 174, 115, 148,
        ]),
        name_constraints: None,
    }];

pub mod oids {
    use const_oid::ObjectIdentifier as OID;

    const fn oid(s: &str) -> OID {
        OID::new_unwrap(s)
    }

    pub const SGX_EXTENSION: OID = oid("1.2.840.113741.1.13.1");
    pub const PPID: OID = oid("1.2.840.113741.1.13.1.1");
    pub const TCB: OID = oid("1.2.840.113741.1.13.1.2");
    pub const PCEID: OID = oid("1.2.840.113741.1.13.1.3");
    pub const FMSPC: OID = oid("1.2.840.113741.1.13.1.4");
    pub const SGX_TYPE: OID = oid("1.2.840.113741.1.13.1.5"); // ASN1 Enumerated
    pub const PLATFORM_INSTANCE_ID: OID = oid("1.2.840.113741.1.13.1.6");
    pub const CONFIGURATION: OID = oid("1.2.840.113741.1.13.1.7");
    pub const PCESVN: OID = oid("1.2.840.113741.1.13.1.2.17");
    pub const CPUSVN: OID = oid("1.2.840.113741.1.13.1.2.18");

    #[test]
    fn const_oid_works() {
        assert_eq!(
            SGX_EXTENSION.as_bytes(),
            oid("1.2.840.113741.1.13.1").as_bytes()
        );
    }
}

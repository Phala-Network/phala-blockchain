use alloc::borrow::ToOwned;
use alloc::vec::Vec;
use core::fmt::Debug;
use core::result::Result;

use byteorder::{ByteOrder, LittleEndian};

use crate::dcap::utils::*;
use crate::Error;

pub(crate) const ENCLAVE_REPORT_BYTE_LEN: usize = 384;

pub(crate) const HEADER_BYTE_LEN: usize = 48;

pub(crate) const AUTH_DATA_SIZE_BYTE_LEN: usize = 4;

pub(crate) const ECDSA_SIGNATURE_BYTE_LEN: usize = 64;
pub(crate) const ECDSA_PUBKEY_BYTE_LEN: usize = 64;
pub(crate) const QE_REPORT_BYTE_LEN: usize = ENCLAVE_REPORT_BYTE_LEN;
pub(crate) const QE_REPORT_SIG_BYTE_LEN: usize = ECDSA_SIGNATURE_BYTE_LEN;
pub(crate) const CERTIFICATION_DATA_TYPE_BYTE_LEN: usize = 2;
pub(crate) const CERTIFICATION_DATA_SIZE_BYTE_LEN: usize = 4;
pub(crate) const QE_AUTH_DATA_SIZE_BYTE_LEN: usize = 2;
pub(crate) const QE_CERT_DATA_TYPE_BYTE_LEN: usize = 2;
pub(crate) const QE_CERT_DATA_SIZE_BYTE_LEN: usize = 4;

pub(crate) const AUTH_DATA_MIN_BYTE_LEN: usize = ECDSA_SIGNATURE_BYTE_LEN
    + ECDSA_PUBKEY_BYTE_LEN
    + QE_REPORT_BYTE_LEN
    + QE_REPORT_SIG_BYTE_LEN
    + QE_AUTH_DATA_SIZE_BYTE_LEN
    + QE_CERT_DATA_TYPE_BYTE_LEN
    + QE_CERT_DATA_SIZE_BYTE_LEN;

pub(crate) const QUOTE_MIN_BYTE_LEN: usize =
    // Actual minimal size is a Quote V3 with Enclave report
    HEADER_BYTE_LEN
        + ENCLAVE_REPORT_BYTE_LEN
        + AUTH_DATA_SIZE_BYTE_LEN
        + AUTH_DATA_MIN_BYTE_LEN;

pub(crate) const ATTESTATION_KEY_LEN: usize = 64;
pub(crate) const AUTHENTICATION_DATA_LEN: usize = 32;

pub(crate) const QE_HASH_DATA_BYTE_LEN: usize = ATTESTATION_KEY_LEN + AUTHENTICATION_DATA_LEN;

pub(crate) const INTEL_QE_VENDOR_ID: [u8; 16] = [
    0x93, 0x9A, 0x72, 0x33, 0xF7, 0x9C, 0x4C, 0xA9, 0x94, 0x0A, 0x0D, 0xB3, 0x95, 0x7F, 0x06, 0x07,
];

pub(crate) const TEE_TYPE_SGX: u32 = 0u32;

pub(crate) type MrSigner = [u8; 32];
pub(crate) type MrEnclave = [u8; 32];
pub(crate) type Fmspc = [u8; 6];
pub(crate) type CpuSvn = [u8; 16];
pub(crate) type Svn = u16;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum QuoteVersion {
    V3,
    // Doc said always this
    Unsupported { raw: u16 },
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AttestationKeyType {
    ECDSA256WithP256Curve,
    // Doc said always this
    ECDSA384WithP384Curve,
    Unsupported { raw: u16 },
}

#[derive(Clone)]
pub struct Header {
    pub version: QuoteVersion,
    pub attestation_key_type: AttestationKeyType,
    pub tee_type: u32,
    // Doc said this is reserved, but implementation is this, it's 0 as doc said.
    pub qe_svn: Svn,
    pub pce_svn: Svn,
    pub qe_vendor_id: [u8; 16],
    pub user_data: [u8; 20],
}

impl Header {
    pub fn from_slice(raw_header: &[u8]) -> Result<Self, Error> {
        if raw_header.len() != HEADER_BYTE_LEN {
            return Err(Error::RawDataInvalid);
        }

        let version = LittleEndian::read_u16(&raw_header[..2]);
        let attestation_key_type = LittleEndian::read_u16(&raw_header[2..4]);
        let tee_type = LittleEndian::read_u32(&raw_header[4..8]);
        let qe_svn = LittleEndian::read_u16(&raw_header[8..10]);
        let pce_svn = LittleEndian::read_u16(&raw_header[10..12]);
        let qe_vendor_id: [u8; 16] = raw_header[12..28].try_into().unwrap();
        let user_data: [u8; 20] = raw_header[28..48].try_into().unwrap();

        // println!("- Quote header -");
        // println!("version: {}", version);
        // println!("attestation key type: {}", attestation_key_type);
        // println!("tee type: {}", tee_type);
        // println!("qe svn: {}", qe_svn);
        // println!("pce svn: {}", pce_svn);
        // println!("qe vendor id: 0x{}", hex::encode(qe_vendor_id));
        // println!("user data: 0x{}", hex::encode(user_data));
        // println!("----------------");

        let version = match version {
            3 => QuoteVersion::V3,
            _ => QuoteVersion::Unsupported { raw: version },
        };
        if version != QuoteVersion::V3 {
            return Err(Error::UnsupportedFieldValue {
                field: "version".to_owned(),
            });
        }

        let attestation_key_type = match attestation_key_type {
            2 => AttestationKeyType::ECDSA256WithP256Curve,
            3 => AttestationKeyType::ECDSA384WithP384Curve,
            _ => AttestationKeyType::Unsupported {
                raw: attestation_key_type,
            },
        };
        // The doc says 3 (ECDSA-384-with-P-384 curve) currently not supported
        if !matches!(
            attestation_key_type,
            AttestationKeyType::ECDSA256WithP256Curve
        ) {
            return Err(Error::UnsupportedFieldValue {
                field: "attestation_key_type".to_owned(),
            });
        }

        if tee_type != TEE_TYPE_SGX {
            return Err(Error::UnsupportedFieldValue {
                field: "tee_type".to_owned(),
            });
        }
        if qe_vendor_id != INTEL_QE_VENDOR_ID {
            return Err(Error::UnsupportedFieldValue {
                field: "qe_vendor_id".to_owned(),
            });
        }

        Ok(Self {
            version,
            attestation_key_type,
            tee_type,
            qe_svn,
            pce_svn,
            qe_vendor_id,
            user_data,
        })
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct EnclaveReport {
    pub cpu_svn: CpuSvn,
    pub misc_select: u32,
    // pub reserved1: [u8; 28],
    pub attributes: [u8; 16],
    pub mr_enclave: MrEnclave,
    // pub reserved2: [u8; 32],
    pub mr_signer: MrSigner,
    // pub reserved_3: [u8; 96],
    pub isv_prod_id: u16,
    pub isv_svn: Svn,
    // pub reserved5: [u8; 60],
    pub report_data: [u8; 64],
}

impl EnclaveReport {
    pub fn from_slice(raw_report: &[u8]) -> Result<Self, Error> {
        if raw_report.len() != ENCLAVE_REPORT_BYTE_LEN {
            return Err(Error::RawDataInvalid);
        }

        let cpu_svn: [u8; 16] = raw_report[..16].try_into().unwrap();
        let misc_select = LittleEndian::read_u32(&raw_report[16..20]);
        // let _reserved: [u8; 28] = raw_report[20..48].try_into().unwrap();
        let attributes: [u8; 16] = raw_report[48..64].try_into().unwrap();
        let mr_enclave: [u8; 32] = raw_report[64..96].try_into().unwrap();
        // let _reserved: [u8; 32] = raw_report[96..128].try_into().unwrap();
        let mr_signer: [u8; 32] = raw_report[128..160].try_into().unwrap();
        // let _reserved: [u8; 96] = raw_report[160..256].try_into().unwrap();
        let isv_prod_id = LittleEndian::read_u16(&raw_report[256..258]);
        let isv_svn = LittleEndian::read_u16(&raw_report[258..260]);
        // let _reserved: [u8; 60] = raw_report[260..320].try_into().unwrap();
        let report_data: [u8; 64] = raw_report[320..384].try_into().unwrap();

        // println!("- Quote enclave report -");
        // println!("cpu svn: 0x{}", hex::encode(cpu_svn));
        // println!("misc select: {}", misc_select);
        // println!("attributes: 0x{}", hex::encode(attributes));
        // println!("mr enclave: 0x{}", hex::encode(mr_enclave));
        // println!("mr signer: 0x{}", hex::encode(mr_signer));
        // println!("isv prod id: {}", isv_prod_id);
        // println!("isv svn: {}", isv_svn);
        // println!("report data: {}", core::str::from_utf8(&report_data).unwrap_or(format!("0x{}", hex::encode(report_data)).as_str()));
        // println!("------------------------");

        Ok(Self {
            cpu_svn,
            misc_select,
            attributes,
            mr_enclave,
            mr_signer,
            isv_prod_id,
            isv_svn,
            report_data,
        })
    }
}

pub struct CertificationData<'a> {
    pub data_type: u16,
    pub certs: Vec<webpki::types::CertificateDer<'a>>,
}

// impl<'a> Debug for CertificationData<'a> {
//     fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
//         f.debug_struct("CertificationData")
//             .field("data_type", &self.data_type)
//             .field("leaf_cert", &hex::encode(&self.leaf_cert.der().as_ref()))
//             .field("intermediate_certs", &self.intermediate_certs.iter().map(|c| c.as_ref()).collect::<Vec<_>>())
//             .finish()
//     }
// }

impl<'a> Debug for CertificationData<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CertificationData")
            .field("data_type", &self.data_type)
            .field(
                "certs",
                &self.certs.iter().map(|c| c.as_ref()).collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl<'a> CertificationData<'a> {
    pub fn from_slice(raw_data: &'a [u8]) -> Result<CertificationData, Error> {
        if raw_data.len() <= CERTIFICATION_DATA_SIZE_BYTE_LEN + CERTIFICATION_DATA_TYPE_BYTE_LEN {
            return Err(Error::RawDataInvalid);
        }

        let data_type = LittleEndian::read_u16(&raw_data[..2]);
        // TODO: guard type
        let data_size = LittleEndian::read_u32(&raw_data[2..6]) as usize;
        // TODO: guard size

        let data = &raw_data[6..(6 + data_size)];

        // println!("- Certification data -");
        // println!("data type: {}", data_type);
        // println!("data_size: {}", data_size);
        // println!("----------------------");

        let certs = extract_certs(data);
        if certs.len() < 2 {
            return Err(Error::InvalidFieldValue {
                field: "data_id".to_owned(),
            });
        }

        Ok(Self { data_type, certs })
    }
}

#[derive(Debug)]
pub enum QuoteAuthData<'a> {
    Ecdsa256Bit {
        signature: Vec<u8>,
        attestation_key: Vec<u8>,
        qe_report: Vec<u8>,
        qe_report_signature: Vec<u8>,
        qe_auth_data: Vec<u8>,
        certification_data: CertificationData<'a>,
    },
    // TODO: V4
    Unsupported,
}

impl<'a> QuoteAuthData<'a> {
    pub fn from_slice(
        attestation_key_type: AttestationKeyType,
        raw_data: &'a [u8],
    ) -> Result<Self, Error> {
        match attestation_key_type {
            AttestationKeyType::ECDSA256WithP256Curve => {
                Self::new_ecdsa256_with_p256_curve(raw_data)
            }
            _ => Ok(Self::Unsupported),
        }
    }

    fn new_ecdsa256_with_p256_curve(raw_data: &'a [u8]) -> Result<Self, Error> {
        let signature = raw_data[..64].to_vec();
        let attestation_key = raw_data[64..128].to_vec();
        let qe_report = raw_data[128..512].to_vec();
        let qe_report_signature = raw_data[512..576].to_vec();
        let qe_auth_data_size = LittleEndian::read_u16(&raw_data[576..578]) as usize;
        let qe_auth_data = raw_data[578..(578 + qe_auth_data_size)].to_vec();
        let raw_certification_data = &raw_data[(578 + qe_auth_data_size)..];
        let certification_data = CertificationData::from_slice(raw_certification_data)?;

        // println!("- ECDSA 256-bit Quote Signature -");
        // println!("signature: {}", hex::encode(signature.clone()));
        // println!("attestation_key: {}", hex::encode(attestation_key.clone()));
        // println!("qe report signature: {}", hex::encode(qe_report_signature.clone()));
        // println!("qe auth data size: {}", qe_auth_data_size);
        // println!("qe auth data: 0x{}", hex::encode(qe_auth_data.clone()));
        // println!("---------------------------------");

        Ok(Self::Ecdsa256Bit {
            signature,
            attestation_key,
            qe_report,
            qe_report_signature,
            qe_auth_data,
            certification_data,
        })
    }
}

pub struct Quote<'a> {
    pub header: Header,
    pub enclave_report: EnclaveReport,
    // Doc calls it `Quote Signature Data Len`
    pub signed_data: QuoteAuthData<'a>, // Doc calls it `Quote Signature Data`
}

impl<'a> Quote<'a> {
    pub fn parse(raw_quote: &'a [u8]) -> Result<Self, Error> {
        if raw_quote.len() < QUOTE_MIN_BYTE_LEN {
            return Err(Error::RawDataInvalid);
        }

        let raw_header = &raw_quote[..HEADER_BYTE_LEN];
        let header = Header::from_slice(raw_header)?;

        let raw_enclave_report =
            &raw_quote[HEADER_BYTE_LEN..(HEADER_BYTE_LEN + ENCLAVE_REPORT_BYTE_LEN)];
        let enclave_report = EnclaveReport::from_slice(raw_enclave_report).expect("Parse error");

        let auth_data_size = LittleEndian::read_u32(
            &raw_quote[(HEADER_BYTE_LEN + ENCLAVE_REPORT_BYTE_LEN)
                ..(HEADER_BYTE_LEN + ENCLAVE_REPORT_BYTE_LEN + 4)],
        ) as usize;
        let raw_signed_data = &raw_quote[(HEADER_BYTE_LEN + ENCLAVE_REPORT_BYTE_LEN + 4)
            ..(HEADER_BYTE_LEN + ENCLAVE_REPORT_BYTE_LEN + 4 + auth_data_size)];
        let signed_data =
            QuoteAuthData::<'a>::from_slice(header.clone().attestation_key_type, raw_signed_data)
                .expect("Parse error");

        // println!("auth_data_size: {}", auth_data_size);

        Ok(Self {
            header,
            enclave_report,
            signed_data,
        })
    }
}

use alloc::string::String;
use alloc::vec::Vec;

use scale::{Decode, Input};

use crate::dcap::constants::*;

#[derive(Debug)]
pub struct Data<T> {
    pub data: Vec<u8>,
    _marker: core::marker::PhantomData<T>,
}

impl<T: Decode + Into<u64>> Decode for Data<T> {
    fn decode<I: Input>(input: &mut I) -> Result<Self, scale::Error> {
        let len = T::decode(input)?;
        let mut data = vec![0u8; len.into() as usize];
        input.read(&mut data)?;
        Ok(Data {
            data,
            _marker: core::marker::PhantomData,
        })
    }
}

#[derive(Decode, Debug)]
pub struct Header {
    pub version: u16,
    pub attestation_key_type: u16,
    pub tee_type: u32,
    pub qe_svn: u16,
    pub pce_svn: u16,
    pub qe_vendor_id: [u8; 16],
    pub user_data: [u8; 20],
}

#[derive(Decode, Debug)]
pub struct Body {
    pub body_type: u16,
    pub size: u32,
}

#[derive(Decode, Debug)]
pub struct EnclaveReport {
    pub cpu_svn: [u8; 16],
    pub misc_select: u32,
    pub reserved1: [u8; 28],
    pub attributes: [u8; 16],
    pub mr_enclave: [u8; 32],
    pub reserved2: [u8; 32],
    pub mr_signer: [u8; 32],
    pub reserved3: [u8; 96],
    pub isv_prod_id: u16,
    pub isv_svn: u16,
    pub reserved4: [u8; 60],
    pub report_data: [u8; 64],
}

#[derive(Decode)]
pub struct CertificationData {
    pub cert_type: u16,
    pub body: Data<u32>,
}

impl core::fmt::Debug for CertificationData {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let body_str = String::from_utf8_lossy(&self.body.data);
        f.debug_struct("CertificationData")
            .field("cert_type", &self.cert_type)
            .field("body", &body_str)
            .finish()
    }
}

#[derive(Decode, Debug)]
pub struct QEReportCertificationData {
    pub qe_report: [u8; ENCLAVE_REPORT_BYTE_LEN],
    pub qe_report_signature: [u8; QE_REPORT_SIG_BYTE_LEN],
    pub qe_auth_data: Data<u16>,
    pub certification_data: CertificationData,
}

#[derive(Decode, Debug)]
pub struct AuthDataV3 {
    pub ecdsa_signature: [u8; ECDSA_SIGNATURE_BYTE_LEN],
    pub ecdsa_attestation_key: [u8; ECDSA_PUBKEY_BYTE_LEN],
    pub qe_report: [u8; ENCLAVE_REPORT_BYTE_LEN],
    pub qe_report_signature: [u8; QE_REPORT_SIG_BYTE_LEN],
    pub qe_auth_data: Data<u16>,
    pub certification_data: CertificationData,
}

#[derive(Debug)]
pub struct AuthDataV4 {
    pub ecdsa_signature: [u8; ECDSA_SIGNATURE_BYTE_LEN],
    pub ecdsa_attestation_key: [u8; ECDSA_PUBKEY_BYTE_LEN],
    pub certification_data: CertificationData,
    pub qe_report_data: QEReportCertificationData,
}

impl Decode for AuthDataV4 {
    fn decode<I: Input>(input: &mut I) -> Result<Self, scale::Error> {
        let ecdsa_signature = Decode::decode(input)?;
        let ecdsa_attestation_key = Decode::decode(input)?;
        let certification_data: CertificationData = Decode::decode(input)?;
        let qe_report_data =
            QEReportCertificationData::decode(&mut &certification_data.body.data[..])?;
        Ok(AuthDataV4 {
            ecdsa_signature,
            ecdsa_attestation_key,
            certification_data,
            qe_report_data,
        })
    }
}

#[derive(Debug)]
pub enum AuthData {
    V3(AuthDataV3),
    V4(AuthDataV4),
}

fn decode_auth_data(ver: u16, input: &mut &[u8]) -> Result<AuthData, scale::Error> {
    match ver {
        3 => {
            let auth_data = AuthDataV3::decode(input)?;
            Ok(AuthData::V3(auth_data))
        }
        4 => {
            let auth_data = AuthDataV4::decode(input)?;
            Ok(AuthData::V4(auth_data))
        }
        _ => Err(scale::Error::from("unsupported quote version")),
    }
}

#[derive(Debug)]
pub struct Quote {
    pub header: Header,
    pub report: EnclaveReport,
    pub auth_data: AuthData,
}

impl Decode for Quote {
    fn decode<I: Input>(input: &mut I) -> Result<Self, scale::Error> {
        let header = Header::decode(input)?;
        let report;
        let data;
        if header.version > 4 {
            let body = Body::decode(input)?;
            if body.body_type != BODY_SGX_ENCLAVE_REPORT_TYPE {
                return Err(scale::Error::from("unsupported body type"));
            }
            report = EnclaveReport::decode(input)?;
            data = Data::<u32>::decode(input)?;
        } else {
            report = EnclaveReport::decode(input)?;
            data = Data::<u32>::decode(input)?;
        }
        let auth_data = decode_auth_data(header.version, &mut &data.data[..])?;
        Ok(Quote {
            header,
            report,
            auth_data,
        })
    }
}

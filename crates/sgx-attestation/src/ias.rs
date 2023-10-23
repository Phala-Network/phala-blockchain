use core::time::Duration;

use alloc::string::String;
use alloc::vec::Vec;
use base64::{engine::general_purpose, Engine as _};
use pink_json::de::Error as JsonError;
use scale::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::Error;

#[derive(Debug, Decode, Encode)]
pub struct EnclaveQuoteBody {
    pub version: [u8; 2],
    pub sign_type: [u8; 2],
    pub epid_group_id: [u8; 4],
    pub qe_svn: [u8; 2],
    pub pce_svn: [u8; 2],
    pub _reserved0: [u8; 4],
    pub basename: [u8; 32],
    pub cpu_svn: [u8; 16],
    pub misc_select: [u8; 4],
    pub _reserved1: [u8; 12],
    pub isv_ext_prod_id: [u8; 16],
    pub attributes: [u8; 16],
    pub mr_enclave: [u8; 32],
    pub _reserved2: [u8; 32],
    pub mr_signer: [u8; 32],
    pub reserved3: [u8; 32],
    pub config_id: [u8; 64],
    pub isv_prod_id: [u8; 2],
    pub isv_svn: [u8; 2],
    pub config_svn: [u8; 2],
    pub reserved4: [u8; 42],
    pub isv_family_id: [u8; 16],
    pub report_data: [u8; 64],
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RaReport {
    pub id: String,
    pub timestamp: String,
    pub version: u8,
    pub epid_pseudonym: String,
    #[serde(rename = "advisoryURL")]
    pub advisory_url: String,
    #[serde(rename = "advisoryIDs")]
    pub advisory_ids: Vec<String>,
    pub isv_enclave_quote_status: String,
    pub isv_enclave_quote_body: String,
}

fn b64_decode(s: &str) -> Result<Vec<u8>, Error> {
    general_purpose::STANDARD
        .decode(s)
        .or(Err(Error::CodecError))
}

impl RaReport {
    pub fn decode_quote(&self) -> Result<EnclaveQuoteBody, Error> {
        let quote_body = b64_decode(&self.isv_enclave_quote_body)?;
        EnclaveQuoteBody::decode(&mut &quote_body[..]).or(Err(Error::CodecError))
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignedIasReport {
    pub ra_report: String,
    pub raw_signing_cert: String,
    pub signature: String,
}

impl SignedIasReport {
    pub fn parse_report(&self) -> Result<RaReport, JsonError> {
        pink_json::from_str(&self.ra_report)
    }

    pub fn verify(&self, now_since_unix_epoch: Duration) -> Result<(), Error> {
        let report = self.ra_report.as_str().as_bytes();
        let signature = b64_decode(&self.signature)?;
        let raw_signing_cert = b64_decode(&self.raw_signing_cert)?;
        verify_signature(report, &signature, &raw_signing_cert, now_since_unix_epoch)
    }
}

pub fn verify_signature(
    message: &[u8],
    signature: &[u8],
    raw_cert: &[u8],
    now_since_unix_epoch: Duration,
) -> Result<(), Error> {
    let sig_cert_der = webpki::types::CertificateDer::from(raw_cert);
    let sig_cert = webpki::EndEntityCert::try_from(&sig_cert_der);
    let sig_cert = sig_cert.or(Err(Error::InvalidCertificate))?;
    let verify_result =
        sig_cert.verify_signature(webpki::RSA_PKCS1_2048_8192_SHA256, message, signature);
    verify_result.or(Err(Error::InvalidSignature))?;
    // Validate certificate
    let chain: Vec<webpki::types::CertificateDer> = Vec::new();
    let time_now = webpki::types::UnixTime::since_unix_epoch(now_since_unix_epoch);
    sig_cert
        .verify_for_usage(
            SUPPORTED_SIG_ALGS,
            IAS_SERVER_ROOTS,
            &chain,
            time_now,
            webpki::KeyUsage::server_auth(),
            None,
        )
        .or(Err(Error::InvalidSignature))?;
    Ok(())
}

type SignatureAlgorithms = &'static [&'static dyn webpki::types::SignatureVerificationAlgorithm];

static SUPPORTED_SIG_ALGS: SignatureAlgorithms = &[
    webpki::RSA_PKCS1_2048_8192_SHA256,
    webpki::RSA_PKCS1_2048_8192_SHA384,
    webpki::RSA_PKCS1_2048_8192_SHA512,
    webpki::RSA_PKCS1_3072_8192_SHA384,
];

static IAS_SERVER_ROOTS: &[webpki::types::TrustAnchor<'static>; 1] = &[
	/*
	 * -----BEGIN CERTIFICATE-----
	 * MIIFSzCCA7OgAwIBAgIJANEHdl0yo7CUMA0GCSqGSIb3DQEBCwUAMH4xCzAJBgNV
	 * BAYTAlVTMQswCQYDVQQIDAJDQTEUMBIGA1UEBwwLU2FudGEgQ2xhcmExGjAYBgNV
	 * BAoMEUludGVsIENvcnBvcmF0aW9uMTAwLgYDVQQDDCdJbnRlbCBTR1ggQXR0ZXN0
	 * YXRpb24gUmVwb3J0IFNpZ25pbmcgQ0EwIBcNMTYxMTE0MTUzNzMxWhgPMjA0OTEy
	 * MzEyMzU5NTlaMH4xCzAJBgNVBAYTAlVTMQswCQYDVQQIDAJDQTEUMBIGA1UEBwwL
	 * U2FudGEgQ2xhcmExGjAYBgNVBAoMEUludGVsIENvcnBvcmF0aW9uMTAwLgYDVQQD
	 * DCdJbnRlbCBTR1ggQXR0ZXN0YXRpb24gUmVwb3J0IFNpZ25pbmcgQ0EwggGiMA0G
	 * CSqGSIb3DQEBAQUAA4IBjwAwggGKAoIBgQCfPGR+tXc8u1EtJzLA10Feu1Wg+p7e
	 * LmSRmeaCHbkQ1TF3Nwl3RmpqXkeGzNLd69QUnWovYyVSndEMyYc3sHecGgfinEeh
	 * rgBJSEdsSJ9FpaFdesjsxqzGRa20PYdnnfWcCTvFoulpbFR4VBuXnnVLVzkUvlXT
	 * L/TAnd8nIZk0zZkFJ7P5LtePvykkar7LcSQO85wtcQe0R1Raf/sQ6wYKaKmFgCGe
	 * NpEJUmg4ktal4qgIAxk+QHUxQE42sxViN5mqglB0QJdUot/o9a/V/mMeH8KvOAiQ
	 * byinkNndn+Bgk5sSV5DFgF0DffVqmVMblt5p3jPtImzBIH0QQrXJq39AT8cRwP5H
	 * afuVeLHcDsRp6hol4P+ZFIhu8mmbI1u0hH3W/0C2BuYXB5PC+5izFFh/nP0lc2Lf
	 * 6rELO9LZdnOhpL1ExFOq9H/B8tPQ84T3Sgb4nAifDabNt/zu6MmCGo5U8lwEFtGM
	 * RoOaX4AS+909x00lYnmtwsDVWv9vBiJCXRsCAwEAAaOByTCBxjBgBgNVHR8EWTBX
	 * MFWgU6BRhk9odHRwOi8vdHJ1c3RlZHNlcnZpY2VzLmludGVsLmNvbS9jb250ZW50
	 * L0NSTC9TR1gvQXR0ZXN0YXRpb25SZXBvcnRTaWduaW5nQ0EuY3JsMB0GA1UdDgQW
	 * BBR4Q3t2pn680K9+QjfrNXw7hwFRPDAfBgNVHSMEGDAWgBR4Q3t2pn680K9+Qjfr
	 * NXw7hwFRPDAOBgNVHQ8BAf8EBAMCAQYwEgYDVR0TAQH/BAgwBgEB/wIBADANBgkq
	 * hkiG9w0BAQsFAAOCAYEAeF8tYMXICvQqeXYQITkV2oLJsp6J4JAqJabHWxYJHGir
	 * IEqucRiJSSx+HjIJEUVaj8E0QjEud6Y5lNmXlcjqRXaCPOqK0eGRz6hi+ripMtPZ
	 * sFNaBwLQVV905SDjAzDzNIDnrcnXyB4gcDFCvwDFKKgLRjOB/WAqgscDUoGq5ZVi
	 * zLUzTqiQPmULAQaB9c6Oti6snEFJiCQ67JLyW/E83/frzCmO5Ru6WjU4tmsmy8Ra
	 * Ud4APK0wZTGtfPXU7w+IBdG5Ez0kE1qzxGQaL4gINJ1zMyleDnbuS8UicjJijvqA
	 * 152Sq049ESDz+1rRGc2NVEqh1KaGXmtXvqxXcTB+Ljy5Bw2ke0v8iGngFBPqCTVB
	 * 3op5KBG3RjbF6RRSzwzuWfL7QErNC8WEy5yDVARzTA5+xmBc388v9Dm21HGfcC8O
	 * DD+gT9sSpssq0ascmvH49MOgjt1yoysLtdCtJW/9FZpoOypaHx0R+mJTLwPXVMrv
	 * DaVzWh5aiEx+idkSGMnX
	 * -----END CERTIFICATE-----
	 */
	webpki::types::TrustAnchor {
		subject: webpki::types::Der::from_slice(b"1\x0b0\t\x06\x03U\x04\x06\x13\x02US1\x0b0\t\x06\x03U\x04\x08\x0c\x02CA1\x140\x12\x06\x03U\x04\x07\x0c\x0bSanta Clara1\x1a0\x18\x06\x03U\x04\n\x0c\x11Intel Corporation100.\x06\x03U\x04\x03\x0c\'Intel SGX Attestation Report Signing CA"),
		subject_public_key_info: webpki::types::Der::from_slice(b"0\r\x06\t*\x86H\x86\xf7\r\x01\x01\x01\x05\x00\x03\x82\x01\x8f\x000\x82\x01\x8a\x02\x82\x01\x81\x00\x9f<d~\xb5w<\xbbQ-\'2\xc0\xd7A^\xbbU\xa0\xfa\x9e\xde.d\x91\x99\xe6\x82\x1d\xb9\x10\xd51w7\twFjj^G\x86\xcc\xd2\xdd\xeb\xd4\x14\x9dj/c%R\x9d\xd1\x0c\xc9\x877\xb0w\x9c\x1a\x07\xe2\x9cG\xa1\xae\x00IHGlH\x9fE\xa5\xa1]z\xc8\xec\xc6\xac\xc6E\xad\xb4=\x87g\x9d\xf5\x9c\t;\xc5\xa2\xe9ilTxT\x1b\x97\x9euKW9\x14\xbeU\xd3/\xf4\xc0\x9d\xdf\'!\x994\xcd\x99\x05\'\xb3\xf9.\xd7\x8f\xbf)$j\xbe\xcbq$\x0e\xf3\x9c-q\x07\xb4GTZ\x7f\xfb\x10\xeb\x06\nh\xa9\x85\x80!\x9e6\x91\tRh8\x92\xd6\xa5\xe2\xa8\x08\x03\x19>@u1@N6\xb3\x15b7\x99\xaa\x82Pt@\x97T\xa2\xdf\xe8\xf5\xaf\xd5\xfec\x1e\x1f\xc2\xaf8\x08\x90o(\xa7\x90\xd9\xdd\x9f\xe0`\x93\x9b\x12W\x90\xc5\x80]\x03}\xf5j\x99S\x1b\x96\xdei\xde3\xed\"l\xc1 }\x10B\xb5\xc9\xab\x7f@O\xc7\x11\xc0\xfeGi\xfb\x95x\xb1\xdc\x0e\xc4i\xea\x1a%\xe0\xff\x99\x14\x88n\xf2i\x9b#[\xb4\x84}\xd6\xff@\xb6\x06\xe6\x17\x07\x93\xc2\xfb\x98\xb3\x14X\x7f\x9c\xfd%sb\xdf\xea\xb1\x0b;\xd2\xd9vs\xa1\xa4\xbdD\xc4S\xaa\xf4\x7f\xc1\xf2\xd3\xd0\xf3\x84\xf7J\x06\xf8\x9c\x08\x9f\r\xa6\xcd\xb7\xfc\xee\xe8\xc9\x82\x1a\x8eT\xf2\\\x04\x16\xd1\x8cF\x83\x9a_\x80\x12\xfb\xdd=\xc7M%by\xad\xc2\xc0\xd5Z\xffo\x06\"B]\x1b\x02\x03\x01\x00\x01"),
		name_constraints: None
	},
];

#[cfg(test)]
mod test {
    use super::*;

    const SAMPLE: &str = include_str!("../sample/ias_attestation.json");
    const ATTESTATION_TIMESTAMP: u64 = 1631441180; // 2021-09-12T18:06:20.402478

    #[test]
    fn could_parse() {
        let report: SignedIasReport = pink_json::from_str(SAMPLE).unwrap();
        let now = core::time::Duration::from_secs(ATTESTATION_TIMESTAMP);
        report.verify(now).expect("verify failed");
        insta::assert_debug_snapshot!(report);
        insta::assert_debug_snapshot!(report.parse_report().unwrap());
        insta::assert_debug_snapshot!(report.parse_report().unwrap().decode_quote().unwrap());
    }
}

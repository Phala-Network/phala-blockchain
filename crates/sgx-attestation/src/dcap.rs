#[cfg(feature = "std")]
pub mod get_collateral;

mod quote;
mod tcb_info;
mod utils;

use alloc::borrow::ToOwned;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use scale::{Decode, Encode};
use scale_info::TypeInfo;

use crate::dcap::quote::{AttestationKeyType, EnclaveReport, Quote, QuoteAuthData, QuoteVersion};
use crate::dcap::tcb_info::TcbInfo;
use crate::dcap::utils::*;
use crate::Error;

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

#[allow(clippy::type_complexity)]
pub fn verify(
    raw_quote: &[u8],
    quote_collateral: &SgxV30QuoteCollateral,
    now: u64,
) -> Result<([u8; 64], Vec<u8>, String, Vec<String>), Error> {
    // Parse data

    let quote = Quote::parse(raw_quote).map_err(|_| Error::CodecError)?;

    let tcb_info = pink_json::from_str::<TcbInfo>(&quote_collateral.tcb_info)
        .map_err(|_| Error::CodecError)?;

    let next_update =
        chrono::DateTime::parse_from_rfc3339(&tcb_info.next_update).map_err(|_| {
            Error::InvalidFieldValue {
                field: "nextUpdate".to_owned(),
            }
        })?;
    if now > next_update.timestamp() as u64 {
        return Err(Error::TCBInfoExpired);
    }

    let now_in_milli = now * 1000;

    // Verify enclave

    // Seems we verify MR_ENCLAVE and MR_SIGNER is enough
    // skip verify_misc_select_field
    // skip verify_attributes_field

    // Verify integrity

    // Check TCB info cert chain and signature
    let leaf_certs = extract_certs(quote_collateral.tcb_info_issuer_chain.as_bytes());
    if leaf_certs.len() < 2 {
        return Err(Error::CertificateChainIsTooShort);
    }
    let leaf_cert: webpki::EndEntityCert = webpki::EndEntityCert::try_from(&leaf_certs[0])
        .map_err(|_| Error::LeafCertificateParsingError)?;
    let intermediate_certs = &leaf_certs[1..];
    verify_certificate_chain(&leaf_cert, intermediate_certs, now_in_milli)?;
    let asn1_signature = encode_as_der(&quote_collateral.tcb_info_signature)?;
    if leaf_cert
        .verify_signature(
            webpki::ECDSA_P256_SHA256,
            quote_collateral.tcb_info.as_bytes(),
            &asn1_signature,
        )
        .is_err()
    {
        return Err(Error::RsaSignatureIsInvalid);
    }

    // Check quote fields
    if quote.header.version != QuoteVersion::V3 {
        return Err(Error::UnsupportedDCAPQuoteVersion);
    }
    // We only support ECDSA256 with P256 curve
    if quote.header.attestation_key_type != AttestationKeyType::ECDSA256WithP256Curve {
        return Err(Error::UnsupportedDCAPAttestationKeyType);
    }

    // Extract Auth data from quote
    let QuoteAuthData::Ecdsa256Bit {
        signature,
        attestation_key,
        qe_report,
        qe_report_signature,
        qe_auth_data,
        certification_data,
    } = quote.signed_data
    else {
        return Err(Error::UnsupportedQuoteAuthData);
    };

    // We only support 5 -Concatenated PCK Cert Chain (PEM formatted).
    if certification_data.data_type != 5 {
        return Err(Error::UnsupportedDCAPPckCertFormat);
    }
    // Check certification_data
    let leaf_cert: webpki::EndEntityCert =
        webpki::EndEntityCert::try_from(&certification_data.certs[0])
            .map_err(|_| Error::LeafCertificateParsingError)?;
    let intermediate_certs = &certification_data.certs[1..];
    verify_certificate_chain(&leaf_cert, intermediate_certs, now_in_milli)?;

    // Check QE signature
    let asn1_signature = encode_as_der(&qe_report_signature)?;
    if leaf_cert
        .verify_signature(webpki::ECDSA_P256_SHA256, &qe_report, &asn1_signature)
        .is_err()
    {
        return Err(Error::RsaSignatureIsInvalid);
    }

    // Extract QE report from quote
    let parsed_qe_report =
        EnclaveReport::from_slice(&qe_report).map_err(|_err| Error::CodecError)?;

    // Check QE hash
    let mut qe_hash_data = [0u8; quote::QE_HASH_DATA_BYTE_LEN];
    qe_hash_data[0..quote::ATTESTATION_KEY_LEN].copy_from_slice(&attestation_key);
    qe_hash_data[quote::ATTESTATION_KEY_LEN..].copy_from_slice(&qe_auth_data);
    let qe_hash = ring::digest::digest(&ring::digest::SHA256, &qe_hash_data);
    if qe_hash.as_ref() != &parsed_qe_report.report_data[0..32] {
        return Err(Error::QEReportHashMismatch);
    }

    // Check signature from auth data
    let mut pub_key = [0x04u8; 65]; //Prepend 0x04 to specify uncompressed format
    pub_key[1..].copy_from_slice(&attestation_key);
    let peer_public_key =
        ring::signature::UnparsedPublicKey::new(&ring::signature::ECDSA_P256_SHA256_FIXED, pub_key);
    peer_public_key
        .verify(
            &raw_quote[..(quote::HEADER_BYTE_LEN + quote::ENCLAVE_REPORT_BYTE_LEN)],
            &signature,
        )
        .map_err(|_| Error::IsvEnclaveReportSignatureIsInvalid)?;

    // Extract information from the quote

    let extension_section = get_intel_extension(&certification_data.certs[0])?;
    let cpu_svn = get_cpu_svn(&extension_section)?;
    let pce_svn = get_pce_svn(&extension_section)?;
    let fmspc = get_fmspc(&extension_section)?;

    let tcb_fmspc = hex::decode(&tcb_info.fmspc).map_err(|_| Error::InvalidFieldValue {
        field: "fmspc".to_owned(),
    })?;
    if fmspc != tcb_fmspc[..] {
        return Err(Error::FmspcMismatch);
    }

    // TCB status and advisory ids
    let mut tcb_status = "Unknown".to_owned();
    let mut advisory_ids = Vec::<String>::new();
    for tcb_level in &tcb_info.tcb_levels {
        if pce_svn >= tcb_level.tcb.pce_svn {
            let selected = !cpu_svn.iter().zip(&tcb_level.tcb.components).any(|(a, b)| a < &b.svn);
            if !selected {
                continue;
            }

            tcb_status = tcb_level.tcb_status.clone();
            tcb_level
                .advisory_ids
                .iter()
                .for_each(|id| advisory_ids.push(id.clone()));

            break;
        }
    }

    let mut pruntime_hash = Vec::new();
    pruntime_hash.extend_from_slice(&quote.enclave_report.mr_enclave);
    pruntime_hash.extend_from_slice(&quote.enclave_report.isv_prod_id.to_be_bytes());
    pruntime_hash.extend_from_slice(&quote.enclave_report.isv_svn.to_be_bytes());
    pruntime_hash.extend_from_slice(&quote.enclave_report.mr_signer);

    Ok((
        quote.enclave_report.report_data,
        pruntime_hash,
        tcb_status.to_string(),
        advisory_ids,
    ))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn could_parse() {
        let raw_quote = include_bytes!("../sample/dcap_quote").to_vec();
        let raw_quote_collateral = include_bytes!("../sample/dcap_quote_collateral").to_vec();
        let now = 1699301000u64;

        let quote_collateral =
            SgxV30QuoteCollateral::decode(&mut raw_quote_collateral.as_slice()).expect("decodable");
        let (report_data, pruntime_hash, tcb_status, advisory_ids) =
            verify(&raw_quote, &quote_collateral, now).expect("verify");

        insta::assert_debug_snapshot!(report_data);
        insta::assert_debug_snapshot!(pruntime_hash);
        insta::assert_debug_snapshot!(tcb_status);
        insta::assert_debug_snapshot!(advisory_ids);
    }
}

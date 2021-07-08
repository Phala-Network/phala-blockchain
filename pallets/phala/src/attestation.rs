use crate::constants::*;

use sp_std::{
	borrow::ToOwned,
	convert::{TryFrom, TryInto},
	vec::Vec,
};

pub enum Error {
	InvalidIASSigningCert,
	InvalidReport,
	InvalidQuoteStatus,
	BadIASReport,
	OutdatedIASReport,
	UnknownQuoteBodyFormat,
}

pub struct IasFields {
	pub mr_enclave: [u8; 32],
	pub mr_signer: [u8; 32],
	pub isv_prod_id: [u8; 2],
	pub isv_svn: [u8; 2],
	pub report_data: [u8; 64],
	pub confidence_level: u8,
}

pub fn validate_ias_report(
	report: &Vec<u8>,
	signature: &Vec<u8>,
	raw_signing_cert: &Vec<u8>,
	now: u64,
) -> Result<IasFields, Error> {
	// Validate report
	let sig_cert = webpki::EndEntityCert::try_from(&raw_signing_cert[..]);
	let sig_cert = sig_cert.or(Err(Error::InvalidIASSigningCert))?;
	let verify_result =
		sig_cert.verify_signature(&webpki::RSA_PKCS1_2048_8192_SHA256, &report, &signature);
	verify_result.or(Err(Error::InvalidIASSigningCert))?;
	// Validate certificate
	let chain: Vec<&[u8]> = Vec::new();
	let time_now = webpki::Time::from_seconds_since_unix_epoch(now);
	let tls_server_cert_valid = sig_cert.verify_is_valid_tls_server_cert(
		SUPPORTED_SIG_ALGS,
		&IAS_SERVER_ROOTS,
		&chain,
		time_now,
	);
	tls_server_cert_valid.or(Err(Error::InvalidIASSigningCert))?;
	// Validate related fields
	let parsed_report: serde_json::Value =
		serde_json::from_slice(&report).or(Err(Error::InvalidReport))?;
	// Validate time
	let raw_report_timestamp = parsed_report["timestamp"]
		.as_str()
		.unwrap_or("UNKNOWN")
		.to_owned()
		+ "Z";
	let report_timestamp = chrono::DateTime::parse_from_rfc3339(&raw_report_timestamp)
		.or(Err(Error::BadIASReport))?
		.timestamp();
	if (now as i64 - report_timestamp) >= 7200 {
		return Err(Error::OutdatedIASReport);
	}
	// Filter valid `isvEnclaveQuoteStatus`
	let quote_status = &parsed_report["isvEnclaveQuoteStatus"]
		.as_str()
		.unwrap_or("UNKNOWN");
	let mut confidence_level: u8 = 128;
	if IAS_QUOTE_STATUS_LEVEL_1.contains(quote_status) {
		confidence_level = 1;
	} else if IAS_QUOTE_STATUS_LEVEL_2.contains(quote_status) {
		confidence_level = 2;
	} else if IAS_QUOTE_STATUS_LEVEL_3.contains(quote_status) {
		confidence_level = 3;
	} else if IAS_QUOTE_STATUS_LEVEL_5.contains(quote_status) {
		confidence_level = 5;
	}
	if confidence_level == 128 {
		return Err(Error::InvalidQuoteStatus);
	}
	if confidence_level < 5 {
		// Filter AdvisoryIDs. `advisoryIDs` is optional
		if let Some(advisory_ids) = parsed_report["advisoryIDs"].as_array() {
			for advisory_id in advisory_ids {
				let advisory_id = advisory_id.as_str().ok_or(Error::BadIASReport)?;
				if !IAS_QUOTE_ADVISORY_ID_WHITELIST.contains(&advisory_id) {
					confidence_level = 4;
				}
			}
		}
	}
	// Extract quote fields
	let raw_quote_body = parsed_report["isvEnclaveQuoteBody"]
		.as_str()
		.ok_or(Error::UnknownQuoteBodyFormat)?;
	let quote_body = base64::decode(&raw_quote_body).or(Err(Error::UnknownQuoteBodyFormat))?;
	// Check the following fields
	Ok(IasFields {
		mr_enclave: (&quote_body[112..144]).try_into().unwrap(),
		mr_signer: (&quote_body[176..208]).try_into().unwrap(),
		isv_prod_id: (&quote_body[304..306]).try_into().unwrap(),
		isv_svn: (&quote_body[306..308]).try_into().unwrap(),
		report_data: (&quote_body[368..432]).try_into().unwrap(),
		confidence_level,
	})
}

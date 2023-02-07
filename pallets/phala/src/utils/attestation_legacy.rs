pub use crate::attestation::Error;
use crate::constants::*;

use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::{
	borrow::ToOwned,
	convert::{TryFrom, TryInto},
	vec::Vec,
};

#[derive(Encode, Decode, TypeInfo, Debug, Clone, PartialEq, Eq)]
pub enum Attestation {
	SgxIas {
		ra_report: Vec<u8>,
		signature: Vec<u8>,
		raw_signing_cert: Vec<u8>,
	},
}

pub trait AttestationValidator {
	/// Validates the attestation as well as the user data hash it commits to.
	fn validate(
		attestation: &Attestation,
		user_data_hash: &[u8; 32],
		now: u64,
		verify_pruntime_hash: bool,
		pruntime_allowlist: Vec<Vec<u8>>,
	) -> Result<IasFields, Error>;
}

#[derive(Encode, Decode, TypeInfo, Debug, Clone, PartialEq, Eq)]
pub struct IasFields {
	pub mr_enclave: [u8; 32],
	pub mr_signer: [u8; 32],
	pub isv_prod_id: [u8; 2],
	pub isv_svn: [u8; 2],
	pub report_data: [u8; 64],
	pub confidence_level: u8,
}

impl IasFields {
	pub fn from_ias_report(report: &[u8]) -> Result<(IasFields, i64), Error> {
		// Validate related fields
		let parsed_report: serde_json::Value =
			serde_json::from_slice(report).or(Err(Error::InvalidReport))?;

		// Extract quote fields
		let raw_quote_body = parsed_report["isvEnclaveQuoteBody"]
			.as_str()
			.ok_or(Error::UnknownQuoteBodyFormat)?;
		let quote_body = base64::decode(raw_quote_body).or(Err(Error::UnknownQuoteBodyFormat))?;
		let mr_enclave = &quote_body[112..144];
		let mr_signer = &quote_body[176..208];
		let isv_prod_id = &quote_body[304..306];
		let isv_svn = &quote_body[306..308];
		let report_data = &quote_body[368..432];

		// Extract report time
		let raw_report_timestamp = parsed_report["timestamp"]
			.as_str()
			.unwrap_or("UNKNOWN")
			.to_owned() + "Z";
		let report_timestamp = chrono::DateTime::parse_from_rfc3339(&raw_report_timestamp)
			.or(Err(Error::BadIASReport))?
			.timestamp();

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
		// CL 1 means there is no known issue of the CPU
		// CL 2 means the worker's firmware up to date, and the worker has well configured to prevent known issues
		// CL 3 means the worker's firmware up to date, but needs to well configure its BIOS to prevent known issues
		// CL 5 means the worker's firmware is outdated
		// For CL 3, we don't know which vulnerable (aka SA) the worker not well configured, so we need to check the allow list
		if confidence_level == 3 {
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

		Ok((
			IasFields {
				mr_enclave: mr_enclave.try_into().unwrap(),
				mr_signer: mr_signer.try_into().unwrap(),
				isv_prod_id: isv_prod_id.try_into().unwrap(),
				isv_svn: isv_svn.try_into().unwrap(),
				report_data: report_data.try_into().unwrap(),
				confidence_level,
			},
			report_timestamp,
		))
	}

	pub fn extend_mrenclave(&self) -> Vec<u8> {
		let mut t_mrenclave = Vec::new();
		t_mrenclave.extend_from_slice(&self.mr_enclave);
		t_mrenclave.extend_from_slice(&self.isv_prod_id);
		t_mrenclave.extend_from_slice(&self.isv_svn);
		t_mrenclave.extend_from_slice(&self.mr_signer);
		t_mrenclave
	}
}

/// Attestation validator implementation for IAS
pub struct IasValidator;
impl AttestationValidator for IasValidator {
	fn validate(
		attestation: &Attestation,
		user_data_hash: &[u8; 32],
		now: u64,
		verify_pruntime: bool,
		pruntime_allowlist: Vec<Vec<u8>>,
	) -> Result<IasFields, Error> {
		let fields = match attestation {
			Attestation::SgxIas {
				ra_report,
				signature,
				raw_signing_cert,
			} => validate_ias_report(
				ra_report,
				signature,
				raw_signing_cert,
				now,
				verify_pruntime,
				pruntime_allowlist,
			),
		}?;
		let commit = &fields.report_data[..32];
		if commit != user_data_hash {
			Err(Error::InvalidUserDataHash)
		} else {
			Ok(fields)
		}
	}
}

pub fn validate_ias_report(
	report: &[u8],
	signature: &[u8],
	raw_signing_cert: &[u8],
	now: u64,
	verify_pruntime: bool,
	pruntime_allowlist: Vec<Vec<u8>>,
) -> Result<IasFields, Error> {
	// Validate report
	let sig_cert = webpki::EndEntityCert::try_from(raw_signing_cert);
	let sig_cert = sig_cert.or(Err(Error::InvalidIASSigningCert))?;
	let verify_result =
		sig_cert.verify_signature(&webpki::RSA_PKCS1_2048_8192_SHA256, report, signature);
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

	let (ias_fields, report_timestamp) = IasFields::from_ias_report(report)?;

	// Validate PRuntime
	if verify_pruntime {
		let t_mrenclave = ias_fields.extend_mrenclave();
		if !pruntime_allowlist.contains(&t_mrenclave) {
			return Err(Error::PRuntimeRejected);
		}
	}

	// Validate time
	if (now as i64 - report_timestamp) >= 7200 {
		return Err(Error::OutdatedIASReport);
	}

	Ok(ias_fields)
}

#[cfg(test)]
mod test {
	use super::*;
	use frame_support::assert_ok;

	pub const ATTESTATION_SAMPLE: &[u8] = include_bytes!("../../sample/ias_attestation.json");
	pub const ATTESTATION_TIMESTAMP: u64 = 1631441180; // 2021-09-12T18:06:20.402478
	pub const PRUNTIME_HASH: &str = "518422fa769d2d55982015a0e0417c6a8521fdfc7308f5ec18aaa1b6924bd0f300000000815f42f11cf64430c30bab7816ba596a1da0130c3b028b673133a66cf9a3e0e6";

	#[test]
	fn test_ias_validator() {
		let sample: serde_json::Value = serde_json::from_slice(ATTESTATION_SAMPLE).unwrap();

		let report = sample["raReport"].as_str().unwrap().as_bytes();
		let signature = hex::decode(sample["signature"].as_str().unwrap().as_bytes()).unwrap();
		let raw_signing_cert =
			hex::decode(sample["rawSigningCert"].as_str().unwrap().as_bytes()).unwrap();

		assert_eq!(
			validate_ias_report(
				report,
				&signature,
				&raw_signing_cert,
				ATTESTATION_TIMESTAMP + 10000000,
				false,
				vec![]
			),
			Err(Error::OutdatedIASReport)
		);

		assert_eq!(
			validate_ias_report(
				report,
				&signature,
				&raw_signing_cert,
				ATTESTATION_TIMESTAMP,
				true,
				vec![]
			),
			Err(Error::PRuntimeRejected)
		);

		assert_ok!(validate_ias_report(
			report,
			&signature,
			&raw_signing_cert,
			ATTESTATION_TIMESTAMP,
			true,
			vec![hex::decode(PRUNTIME_HASH).unwrap()]
		));
	}
}

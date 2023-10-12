pub use crate::attestation::Error;
use crate::constants::*;

use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::vec::Vec;

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
		use sgx_attestation::ias::RaReport;
		// Validate related fields
		let parsed_report: RaReport =
			serde_json::from_slice(report).or(Err(Error::InvalidReport))?;

		// Extract report time
		let raw_report_timestamp = parsed_report.timestamp.clone() + "Z";
		let report_timestamp = chrono::DateTime::parse_from_rfc3339(&raw_report_timestamp)
			.or(Err(Error::BadIASReport))?
			.timestamp();

		// Filter valid `isvEnclaveQuoteStatus`
		let quote_status = &parsed_report.isv_enclave_quote_status.as_str();
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
			for advisory_id in parsed_report.advisory_ids.iter() {
				if !IAS_QUOTE_ADVISORY_ID_WHITELIST.contains(&advisory_id.as_str()) {
					confidence_level = 4;
				}
			}
		}

		// Extract quote fields
		let quote = parsed_report
			.decode_quote()
			.or(Err(Error::UnknownQuoteBodyFormat))?;
		Ok((
			IasFields {
				mr_enclave: quote.mr_enclave,
				mr_signer: quote.mr_signer,
				isv_prod_id: quote.isv_prod_id,
				isv_svn: quote.isv_svn,
				report_data: quote.report_data,
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
	sgx_attestation::ias::verify_signature(
		report,
		signature,
		raw_signing_cert,
		core::time::Duration::from_secs(now),
	)
	.or(Err(Error::InvalidIASSigningCert))?;

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

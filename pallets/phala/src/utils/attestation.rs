use crate::constants::*;

use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::vec::Vec;

use phala_types::{AttestationProvider, AttestationReport, Collateral};
use sgx_attestation::dcap::SgxV30QuoteCollateral;

#[derive(Encode, Decode, TypeInfo, Debug, Clone, PartialEq, Eq)]
pub enum Error {
	PRuntimeRejected,
	InvalidIASSigningCert,
	InvalidReport,
	InvalidQuoteStatus,
	BadIASReport,
	OutdatedIASReport,
	UnknownQuoteBodyFormat,
	InvalidUserDataHash,
	NoneAttestationDisabled,
	UnsupportedAttestationType,
	InvalidDCAPQuote(sgx_attestation::Error),
}

#[derive(Encode, Decode, TypeInfo, Debug, Clone, PartialEq, Eq)]
pub struct ConfidentialReport {
	pub confidence_level: u8,
	pub provider: Option<AttestationProvider>,
	pub runtime_hash: Vec<u8>,
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
		let parsed_report: sgx_attestation::ias::RaReport =
			serde_json::from_slice(report).or(Err(Error::InvalidReport))?;

		// Extract report time
		let raw_report_timestamp = parsed_report.timestamp.clone() + "Z";
		let report_timestamp = chrono::DateTime::parse_from_rfc3339(&raw_report_timestamp)
			.or(Err(Error::BadIASReport))?
			.timestamp();

		// Filter valid `isvEnclaveQuoteStatus`
		let quote_status = &parsed_report.isv_enclave_quote_status.as_str();
		let mut confidence_level: u8 = 128;
		if SGX_QUOTE_STATUS_LEVEL_1.contains(quote_status) {
			confidence_level = 1;
		} else if SGX_QUOTE_STATUS_LEVEL_2.contains(quote_status) {
			confidence_level = 2;
		} else if SGX_QUOTE_STATUS_LEVEL_3.contains(quote_status) {
			confidence_level = 3;
		} else if SGX_QUOTE_STATUS_LEVEL_5.contains(quote_status) {
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
				let advisory_id = advisory_id.as_str();
				if !SGX_QUOTE_ADVISORY_ID_WHITELIST.contains(&advisory_id) {
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

pub fn validate(
	attestation: Option<AttestationReport>,
	user_data_hash: &[u8; 32],
	now: u64,
	verify_pruntime_hash: bool,
	pruntime_allowlist: Vec<Vec<u8>>,
	opt_out_enabled: bool,
) -> Result<ConfidentialReport, Error> {
	match attestation {
		Some(AttestationReport::SgxIas {
			ra_report,
			signature,
			raw_signing_cert,
		}) => validate_ias_report(
			user_data_hash,
			ra_report.as_slice(),
			signature.as_slice(),
			raw_signing_cert.as_slice(),
			now,
			verify_pruntime_hash,
			pruntime_allowlist,
		),
		Some(AttestationReport::SgxDcap { quote, collateral }) => {
			let Some(Collateral::SgxV30(collateral)) = collateral else {
				return Err(Error::UnsupportedAttestationType);
			};
			validate_dcap(
				&quote,
				&collateral,
				now,
				user_data_hash,
				verify_pruntime_hash,
				pruntime_allowlist,
			)
		}
		None => {
			if opt_out_enabled {
				Ok(ConfidentialReport {
					provider: None,
					runtime_hash: Vec::new(),
					confidence_level: 128u8,
				})
			} else {
				Err(Error::NoneAttestationDisabled)
			}
		}
	}
}

pub fn validate_ias_report(
	user_data_hash: &[u8],
	report: &[u8],
	signature: &[u8],
	raw_signing_cert: &[u8],
	now: u64,
	verify_pruntime_hash: bool,
	pruntime_allowlist: Vec<Vec<u8>>,
) -> Result<ConfidentialReport, Error> {
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
	let pruntime_hash = ias_fields.extend_mrenclave();
	if verify_pruntime_hash && !pruntime_allowlist.contains(&pruntime_hash) {
		return Err(Error::PRuntimeRejected);
	}

	// Validate time
	if (now as i64 - report_timestamp) >= 7200 {
		return Err(Error::OutdatedIASReport);
	}

	let commit = &ias_fields.report_data[..32];
	if commit != user_data_hash {
		return Err(Error::InvalidUserDataHash);
	}

	// Check the following fields
	Ok(ConfidentialReport {
		provider: Some(AttestationProvider::Ias),
		runtime_hash: pruntime_hash,
		confidence_level: ias_fields.confidence_level,
	})
}

pub fn validate_dcap(
	quote: &[u8],
	collateral: &SgxV30QuoteCollateral,
	now: u64,
	user_data_hash: &[u8],
	verify_pruntime_hash: bool,
	pruntime_allowlist: Vec<Vec<u8>>,
) -> Result<ConfidentialReport, Error> {
	// Validate report
	let (report_data, pruntime_hash, tcb_status, advisory_ids) = sgx_attestation::dcap::verify(
		quote,
		collateral,
		now,
	).map_err(Error::InvalidDCAPQuote)?;

	// Validate PRuntime
	if verify_pruntime_hash && !pruntime_allowlist.contains(&pruntime_hash) {
		return Err(Error::PRuntimeRejected);
	}

	let commit = &report_data[..32];
	if commit != user_data_hash {
		return Err(Error::InvalidUserDataHash);
	}

	let mut confidence_level: u8 = 128;
	if SGX_QUOTE_STATUS_LEVEL_1.contains(&tcb_status.as_str()) {
		confidence_level = 1;
	} else if SGX_QUOTE_STATUS_LEVEL_2.contains(&tcb_status.as_str()) {
		confidence_level = 2;
	} else if SGX_QUOTE_STATUS_LEVEL_3.contains(&tcb_status.as_str()) {
		confidence_level = 3;
	} else if SGX_QUOTE_STATUS_LEVEL_5.contains(&tcb_status.as_str()) {
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
		for advisory_id in advisory_ids.iter() {
			let advisory_id = advisory_id.as_str();
			if !SGX_QUOTE_ADVISORY_ID_WHITELIST.contains(&advisory_id) {
				confidence_level = 4;
			}
		}
	}

	// Check the following fields
	Ok(ConfidentialReport {
		provider: Some(AttestationProvider::Dcap),
		runtime_hash: pruntime_hash,
		confidence_level
	})
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
		let sample: sgx_attestation::ias::SignedIasReport =
			serde_json::from_slice(ATTESTATION_SAMPLE).unwrap();

		let report = sample.ra_report.as_bytes();
		let parsed_report = sample.parse_report().unwrap();
		let signature = hex::decode(sample.signature.as_str()).unwrap();
		let raw_signing_cert = hex::decode(sample.raw_signing_cert.as_str()).unwrap();

		let quote = parsed_report.decode_quote().unwrap();
		let commit = &quote.report_data[..32];

		assert_eq!(
			validate_ias_report(
				&[0u8],
				report,
				&signature,
				&raw_signing_cert,
				ATTESTATION_TIMESTAMP,
				false,
				vec![]
			),
			Err(Error::InvalidUserDataHash)
		);

		assert_eq!(
			validate_ias_report(
				commit,
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
				commit,
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
			commit,
			report,
			&signature,
			&raw_signing_cert,
			ATTESTATION_TIMESTAMP,
			true,
			vec![hex::decode(PRUNTIME_HASH).unwrap()]
		));
	}
}

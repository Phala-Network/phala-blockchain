pub const RANDOMNESS_SUBJECT: &[u8] = b"PhalaPoW";

pub const SGX_QUOTE_STATUS_LEVEL_1: &[&str] = &[
	// IAS
	"OK",
	// DCAP
	"UpToDate",
];
pub const SGX_QUOTE_STATUS_LEVEL_2: &[&str] = &[
	// IAS
	"SW_HARDENING_NEEDED",
	// DCAP
	"SWHardeningNeeded",
];
pub const SGX_QUOTE_STATUS_LEVEL_3: &[&str] = &[
	// IAS
	"CONFIGURATION_NEEDED",
	"CONFIGURATION_AND_SW_HARDENING_NEEDED",
	// DCAP
	"ConfigurationNeeded",
	"ConfigurationAndSWHardeningNeeded",
];
// LEVEL 4 is LEVEL 3 with advisors which not included in whitelist
pub const SGX_QUOTE_STATUS_LEVEL_5: &[&str] = &[
	// IAS
	"GROUP_OUT_OF_DATE",
	// DCAP
	"OutOfDate",
	"OutOfDateConfigurationNeeded",
];
pub const SGX_QUOTE_ADVISORY_ID_WHITELIST: &[&str] = &[
	"INTEL-SA-00334",
	"INTEL-SA-00219",
	"INTEL-SA-00381",
	"INTEL-SA-00389",
];

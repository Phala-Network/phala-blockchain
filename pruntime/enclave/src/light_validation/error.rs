use crate::std::string::String;
use derive_more::{Display, From};

/// Substrate Client error
#[derive(Debug, Display, From)]
pub enum JustificationError {
// /// Consensus Error
// #[display(fmt = "Consensus: {}", _0)]
// Consensus(sp_consensus::Error),
// /// Backend error.
// #[display(fmt = "Backend error: {}", _0)]
// #[from(ignore)]
// Backend(String),
// /// Unknown block.
// #[display(fmt = "UnknownBlock: {}", _0)]
// #[from(ignore)]
// UnknownBlock(String),
// /// The `apply_extrinsic` is not valid due to the given `TransactionValidityError`.
// #[display(fmt = "{:?}", _0)]
// ApplyExtrinsicFailed(ApplyExtrinsicFailed),
// /// Execution error.
// #[display(fmt = "Execution: {}", _0)]
// Execution(Box<dyn sp_state_machine::Error>),
// /// Blockchain error.
// #[display(fmt = "Blockchain: {}", _0)]
// Blockchain(Box<Error>),
// /// Invalid authorities set received from the runtime.
// #[display(fmt = "Current state of blockchain has invalid authorities set")]
// InvalidAuthoritiesSet,
// /// Could not get runtime version.
// #[display(fmt = "Failed to get runtime version: {}", _0)]
// #[from(ignore)]
// VersionInvalid(String),
// /// Genesis config is invalid.
// #[display(fmt = "Genesis config provided is invalid")]
// GenesisInvalid,
	/// Error decoding header justification.
	#[display(fmt = "error decoding justification for header")]
	JustificationDecode,
	/// Justification for header is correctly encoded, but invalid.
	#[display(fmt = "bad justification for header: {}", _0)]
	#[from(ignore)]
	BadJustification(String),
// /// Not available on light client.
// #[display(fmt = "This method is not currently available when running in light client mode")]
// NotAvailableOnLightClient,
// /// Invalid remote CHT-based proof.
// #[display(fmt = "Remote node has responded with invalid header proof")]
// InvalidCHTProof,
// /// Remote fetch has been cancelled.
// #[display(fmt = "Remote data fetch has been cancelled")]
// RemoteFetchCancelled,
// /// Remote fetch has been failed.
// #[display(fmt = "Remote data fetch has been failed")]
// RemoteFetchFailed,
// /// Error decoding call result.
// #[display(fmt = "Error decoding call result of {}: {}", _0, _1)]
// CallResultDecode(&'static str, CodecError),
// /// Error converting a parameter between runtime and node.
// #[display(fmt = "Error converting `{}` between runtime and node", _0)]
// #[from(ignore)]
// RuntimeParamConversion(String),
// /// Changes tries are not supported.
// #[display(fmt = "Changes tries are not supported by the runtime")]
// ChangesTriesNotSupported,
// /// Key changes query has failed.
// #[display(fmt = "Failed to check changes proof: {}", _0)]
// #[from(ignore)]
// ChangesTrieAccessFailed(String),
// /// Last finalized block not parent of current.
// #[display(fmt = "Did not finalize blocks in sequential order.")]
// #[from(ignore)]
// NonSequentialFinalization(String),
// /// Safety violation: new best block not descendent of last finalized.
// #[display(fmt = "Potential long-range attack: block not in finalized chain.")]
// NotInFinalizedChain,
// /// Hash that is required for building CHT is missing.
// #[display(fmt = "Failed to get hash of block for building CHT")]
// MissingHashRequiredForCHT,
// /// Invalid calculated state root on block import.
// #[display(fmt = "Calculated state root does not match.")]
// InvalidStateRoot,
// /// A convenience variant for String
// #[display(fmt = "{}", _0)]
// Msg(String),
}
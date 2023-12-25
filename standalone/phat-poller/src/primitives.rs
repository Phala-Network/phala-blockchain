use pallet_contracts::{ExecReturnValue, StorageDeposit};
use scale::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::{DispatchError, RuntimeDebug};
use sp_weights::Weight;

// Forked from pallet_contracts polkadot-v0.9.42

#[derive(Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ContractResult<R, Balance> {
	/// How much weight was consumed during execution.
	pub gas_consumed: Weight,
	/// How much weight is required as gas limit in order to execute this call.
	///
	/// This value should be used to determine the weight limit for on-chain execution.
	///
	/// # Note
	///
	/// This can only different from [`Self::gas_consumed`] when weight pre charging
	/// is used. Currently, only `seal_call_runtime` makes use of pre charging.
	/// Additionally, any `seal_call` or `seal_instantiate` makes use of pre-charging
	/// when a non-zero `gas_limit` argument is supplied.
	pub gas_required: Weight,
	/// How much balance was paid by the origin into the contract's deposit account in order to
	/// pay for storage.
	///
	/// The storage deposit is never actually charged from the origin in case of [`Self::result`]
	/// is `Err`. This is because on error all storage changes are rolled back including the
	/// payment of the deposit.
	pub storage_deposit: StorageDeposit<Balance>,
	/// An optional debug message. This message is only filled when explicitly requested
	/// by the code that calls into the contract. Otherwise it is empty.
	///
	/// The contained bytes are valid UTF-8. This is not declared as `String` because
	/// this type is not allowed within the runtime.
	///
	/// Clients should not make any assumptions about the format of the buffer.
	/// They should just display it as-is. It is **not** only a collection of log lines
	/// provided by a contract but a formatted buffer with different sections.
	///
	/// # Note
	///
	/// The debug message is never generated during on-chain execution. It is reserved for
	/// RPC calls.
	pub debug_message: Vec<u8>,
	/// The execution result of the wasm code.
	pub result: R,
}

pub type ContractExecResult<Balance> =
	ContractResult<Result<ExecReturnValue, DispatchError>, Balance>;

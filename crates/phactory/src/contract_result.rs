//! Types compatible with pallet-contracts.

use parity_scale_codec::{Decode, Encode};
use sp_runtime::DispatchError;

#[derive(Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Clone, Debug)]
pub enum StorageDeposit {
    Refund(u128),
    Charge(u128),
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Clone, Debug)]
pub struct Weight {
    #[codec(compact)]
    pub ref_time: u64,
    #[codec(compact)]
    pub proof_size: u64,
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Clone, Debug)]
pub struct ExecReturnValue {
    pub flags: u32,
    pub data: Vec<u8>,
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Clone, Debug)]
pub struct InstantiateReturnValue {
    pub flags: u32,
    pub data: Vec<u8>,
    pub account_id: [u8; 32],
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Clone, Debug)]
pub struct ContractResult<R> {
    pub gas_consumed: Weight,
    pub gas_required: Weight,
    pub storage_deposit: StorageDeposit,
    pub debug_message: Vec<u8>,
    pub result: R,
}

pub type ContractExecResult = ContractResult<Result<ExecReturnValue, DispatchError>>;
pub type ContractInstantiateResult = ContractResult<Result<InstantiateReturnValue, DispatchError>>;

use codec::{Decode, Encode};
use frame_support::BoundedVec;
use sp_core::ConstU32;
use sp_std::vec::Vec;

pub type ActionBytes = BoundedVec<u8, ConstU32<256>>;
pub type KeyBytes = BoundedVec<u8, ConstU32<128>>;
pub type ValueBytes = BoundedVec<u8, ConstU32<256>>;

// Almost copied from `phat-offchain-rollup/phat/src/lib.rs`.
#[derive(Debug, Default, PartialEq, Eq, Encode, Decode, Clone, scale_info::TypeInfo)]
pub struct RollupTx {
	pub conds: Vec<Cond>,
	pub actions: Vec<ActionBytes>,
	pub updates: Vec<(KeyBytes, Option<ValueBytes>)>,
}

#[derive(Debug, PartialEq, Eq, Encode, Decode, Clone, scale_info::TypeInfo)]
pub enum Cond {
	Eq(KeyBytes, Option<ValueBytes>),
}

// Defined for our own usage for now

#[derive(Debug, PartialEq, Eq, Encode, Decode, Clone, scale_info::TypeInfo)]
pub enum Action {
	Reply(ActionBytes),
	SetQueueHead(u32),
}

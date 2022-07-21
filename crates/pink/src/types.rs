use sp_runtime::{AccountId32, traits::BlakeTwo256};

pub use frame_support::weights::Weight;

pub type Hash = sp_core::H256;
pub type Hashing = BlakeTwo256;
pub type AccountId = AccountId32;
pub type Balance = u128;
pub type BlockNumber = u32;
pub type Index = u64;

pub const ENOUGH: Balance = Balance::MAX / 2;
pub const GAS_LIMIT: Weight = Weight::MAX / 2;

// The magic 9218808636700 is the gas required to start a 5MB sideprog.
pub const COMMAND_GAS_LIMIT: Weight = 9218808636700;

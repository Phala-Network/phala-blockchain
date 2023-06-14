use scale::{Decode, Encode};
use sp_runtime::{traits::BlakeTwo256, AccountId32};

pub use frame_support::weights::Weight;

use crate::runtime::SystemEvents;

pub type Hash = sp_core::H256;
pub type Hashing = BlakeTwo256;
pub type AccountId = AccountId32;
pub type Balance = u128;
pub type BlockNumber = u32;
pub type Index = u64;

#[derive(Encode, Decode, Clone)]
pub struct EventsBlock {
    pub parent_hash: Hash,
    pub number: u64,
    pub phala_block_number: BlockNumber,
    pub runtime_version: (u32, u32),
    // The fields below may be changed in the future. They should be decoded as types determined by
    // runtime_version.
    pub events: SystemEvents,
}

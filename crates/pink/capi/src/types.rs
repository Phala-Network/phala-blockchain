use scale::{Decode, Encode};
use sp_core::Hasher;
use sp_runtime::{traits::BlakeTwo256, AccountId32};

pub type Hash = <BlakeTwo256 as Hasher>::Out;
pub type Hashing = BlakeTwo256;
pub type AccountId = AccountId32;
pub type Balance = u128;
pub type BlockNumber = u32;
pub type Index = u64;
pub type Address = AccountId32;
pub type Weight = u64;

pub use pink_extension::{HookPoint, PinkEvent};

#[derive(Decode, Encode, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecMode {
    Query,
    Estimate,
    Transaction,
}

impl ExecMode {
    pub fn is_query(&self) -> bool {
        matches!(self, ExecMode::Query)
    }

    pub fn is_transaction(&self) -> bool {
        matches!(self, ExecMode::Transaction)
    }

    pub fn is_estimate(&self) -> bool {
        matches!(self, ExecMode::Estimate)
    }

    pub fn should_return_coarse_gas(&self) -> bool {
        match self {
            ExecMode::Query => true,
            ExecMode::Estimate => true,
            ExecMode::Transaction => false,
        }
    }

    pub fn deterministic_required(&self) -> bool {
        match self {
            ExecMode::Query => false,
            ExecMode::Estimate => true,
            ExecMode::Transaction => true,
        }
    }
}

#[derive(Decode, Encode)]
pub enum ExecSideEffects {
    V1 {
        pink_events: Vec<(AccountId, PinkEvent)>,
        ink_events: Vec<(AccountId, Vec<Hash>, Vec<u8>)>,
        instantiated: Vec<(AccountId, AccountId)>,
    },
}

impl ExecSideEffects {
    pub fn into_query_only_effects(self) -> Self {
        match self {
            ExecSideEffects::V1 {
                pink_events,
                ink_events: _,
                instantiated: _,
            } => Self::V1 {
                pink_events: pink_events
                    .into_iter()
                    .filter(|(_, event)| event.allowed_in_query())
                    .collect(),
                ink_events: vec![],
                instantiated: vec![],
            },
        }
    }
}

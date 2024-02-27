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

pub use pink::{HookPoint, PinkEvent};

/// The mode in which the runtime is currently executing.
#[derive(Decode, Encode, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ExecutionMode {
    /// In this mode, the runtime is executing an RPC query. Any state changes are discarded
    /// after execution. Indeterministic operations like HTTP requests are allowed in this mode.
    Query,
    /// In this mode, the runtime is simulating a transaction but state changes are discarded.
    #[default]
    Estimating,
    /// In this mode, the runtime is executing a real transaction. State changes will be committed.
    /// Indeterministic operations like HTTP requests aren't allowed in this mode.
    Transaction,
}

impl ExecutionMode {
    pub fn display(&self) -> &'static str {
        match self {
            ExecutionMode::Query => "query",
            ExecutionMode::Estimating => "estimating",
            ExecutionMode::Transaction => "transaction",
        }
    }

    /// Returns whether the execution mode is `Query`.
    pub fn is_query(&self) -> bool {
        matches!(self, ExecutionMode::Query)
    }

    /// Returns whether the execution mode is `Transaction`.
    pub fn is_transaction(&self) -> bool {
        matches!(self, ExecutionMode::Transaction)
    }

    /// Returns whether the execution mode is `Estimating`.
    pub fn is_estimating(&self) -> bool {
        matches!(self, ExecutionMode::Estimating)
    }

    /// Returns whether the execution mode should return coarse gas.
    ///
    /// Coarse gas is returned in `Query` and `Estimating` modes to help mitigate
    /// potential side-channel attacks.
    pub fn should_return_coarse_gas(&self) -> bool {
        match self {
            ExecutionMode::Query => true,
            ExecutionMode::Estimating => true,
            ExecutionMode::Transaction => false,
        }
    }

    /// Returns whether the execution mode requires deterministic execution.
    pub fn deterministic_required(&self) -> bool {
        match self {
            ExecutionMode::Query => false,
            ExecutionMode::Estimating => true,
            ExecutionMode::Transaction => true,
        }
    }
}

/// Events emitted by contracts which can potentially lead to further actions by the pruntime.
#[derive(Decode, Encode, Debug, Clone)]
pub enum ExecSideEffects {
    V1 {
        pink_events: Vec<(AccountId, PinkEvent)>,
        ink_events: Vec<(AccountId, Vec<Hash>, Vec<u8>)>,
        instantiated: Vec<(AccountId, AccountId)>,
    },
}

impl ExecSideEffects {
    /// Filters and retains only those events which are permissible in a query context.
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

    /// Returns true if there are no side effects inside.
    pub fn is_empty(&self) -> bool {
        match self {
            ExecSideEffects::V1 {
                pink_events,
                ink_events,
                instantiated,
            } => pink_events.is_empty() && ink_events.is_empty() && instantiated.is_empty(),
        }
    }
}

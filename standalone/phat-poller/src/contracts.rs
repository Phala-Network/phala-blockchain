use phaxt::AccountId;
use scale::{Decode, Encode};
// TODO: load from metadata?

type ContractId = sp_core::H256;

#[derive(Encode, Decode, Debug)]
pub enum Error {
    BadOrigin,
    NotConfigured,
    Deprecated,
    NoPollForTransaction,
    BadWorkflowSession,
    BadEvmSecretKey,
    BadUnsignedTransaction,
    WorkflowNotFound,
    WorkflowDisabled,
    NoAuthorizedExternalAccount,
    ExternalAccountNotFound,
    ExternalAccountDisabled,
    FailedToGetEthAccounts(String),
    FailedToSignTransaction(String),
}

pub const SELECTOR_POLL: u32 = 0x1e44dfc6;
pub const SELECTOR_GET_USER_PROFILES: u32 = 0xbcb18cc3;
pub const SELECTOR_WORKFLOW_COUNT: u32 = 0x198e532a;

pub type UserProfilesResponse = Result<Vec<(AccountId, ContractId)>, Error>;
pub type PollResponse = Result<(), Error>;

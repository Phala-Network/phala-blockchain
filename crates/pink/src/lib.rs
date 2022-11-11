extern crate alloc;

mod contract;
mod export_fixtures;

pub mod runtime;
pub mod storage;

pub mod types;

pub use contract::{transpose_contract_result, Contract, ContractFile, Storage};
pub use export_fixtures::load_test_wasm;
pub use pink_extension_runtime::local_cache;

pub mod predefined_accounts {
    use crate::types::AccountId;
    use pink_extension::predefined_accounts::ACCOUNT_PALLET;

    pub const fn pallet_account() -> AccountId {
        AccountId::new(ACCOUNT_PALLET)
    }
}

#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(dead_code)]
mod types;
pub use types::*;

pub trait CrossCall {
    fn cross_call(&self, id: u32, data: &[u8]) -> Vec<u8>;
}

pub trait CrossCallMut {
    fn cross_call_mut(&mut self, call_id: u32, data: &[u8]) -> Vec<u8>;
}

pub trait Executing {
    fn execute<T>(&self, f: impl FnOnce() -> T) -> T;
    fn execute_mut<T>(&mut self, f: impl FnOnce() -> T) -> T;
}

pub struct IdentExecute;
impl Executing for IdentExecute {
    fn execute<T>(&self, f: impl FnOnce() -> T) -> T {
        f()
    }

    fn execute_mut<T>(&mut self, f: impl FnOnce() -> T) -> T {
        f()
    }
}

pub trait ECall: CrossCall {}
pub trait OCall: CrossCall {}

pub mod ecall {
    use super::{CrossCallMut, ECall, Executing};
    use crate::types::{AccountId, Balance, ExecutionMode, Hash, Weight};
    use pink_macro::cross_call;
    use scale::{Decode, Encode};
    pub trait EventCallbacks {
        fn log_to_server(&self, contract: &AccountId, in_query: bool, level: u8, message: String);
    }

    #[derive(Encode, Decode, Clone, Debug)]
    pub struct TransactionArguments {
        pub origin: AccountId,
        pub transfer: Balance,
        pub gas_limit: Weight,
        pub gas_free: bool,
        pub storage_deposit_limit: Option<Balance>,
    }

    #[derive(Encode, Decode, Clone, Debug)]
    pub struct ClusterSetupConfig {
        pub cluster_id: Hash,
        pub owner: AccountId,
        pub deposit: Balance,
        pub gas_price: Balance,
        pub deposit_per_item: Balance,
        pub deposit_per_byte: Balance,
        pub treasury_account: AccountId,
        pub system_code: Vec<u8>,
    }

    #[cross_call(ECall)]
    pub trait ECalls {
        #[xcall(id = 1)]
        fn cluster_id(&self) -> Hash;
        #[xcall(id = 2)]
        fn setup(&mut self, config: ClusterSetupConfig) -> Result<(), String>;
        #[xcall(id = 3)]
        fn deposit(&mut self, who: AccountId, value: Balance);
        #[xcall(id = 5)]
        fn set_key(&mut self, key: [u8; 64]);
        #[xcall(id = 6)]
        fn get_key(&self) -> Option<[u8; 64]>;
        #[xcall(id = 7)]
        fn upload_code(
            &mut self,
            account: AccountId,
            code: Vec<u8>,
            deterministic: bool,
        ) -> Result<Hash, String>;
        #[xcall(id = 8)]
        fn upload_sidevm_code(&mut self, account: AccountId, code: Vec<u8>)
            -> Result<Hash, String>;
        #[xcall(id = 9)]
        fn get_sidevm_code(&self, hash: Hash) -> Option<Vec<u8>>;
        #[xcall(id = 11)]
        fn system_contract(&self) -> Option<AccountId>;
        #[xcall(id = 14)]
        fn free_balance(&self, account: AccountId) -> Balance;
        #[xcall(id = 15)]
        fn total_balance(&self, account: AccountId) -> Balance;
        #[xcall(id = 16)]
        fn code_hash(&self, account: AccountId) -> Option<Hash>;
        #[xcall(id = 19)]
        fn contract_instantiate(
            &mut self,
            code_hash: Hash,
            input_data: Vec<u8>,
            salt: Vec<u8>,
            mode: ExecutionMode,
            tx_args: TransactionArguments,
        ) -> Vec<u8>;
        #[xcall(id = 20)]
        fn contract_call(
            &mut self,
            contract: AccountId,
            input_data: Vec<u8>,
            mode: ExecutionMode,
            tx_args: TransactionArguments,
        ) -> Vec<u8>;
        #[xcall(id = 21)]
        fn git_revision(&self) -> String;
    }
}

pub mod ocall {
    use super::{CrossCallMut, Executing, OCall};
    use crate::types::{AccountId, BlockNumber, ExecSideEffects, ExecutionMode, Hash};
    use pink_macro::cross_call;
    use scale::{Decode, Encode};

    pub use pink_extension::chain_extension::{
        HttpRequest, HttpRequestError, HttpResponse, StorageQuotaExceeded,
    };
    pub type StorageChanges = Vec<(Vec<u8>, (Vec<u8>, i32))>;

    #[derive(Decode, Encode, Clone, Debug, Default)]
    pub struct ExecContext {
        pub mode: ExecutionMode,
        pub block_number: BlockNumber,
        pub now_ms: u64,
        pub req_id: Option<u64>,
    }

    impl ExecContext {
        pub fn new(
            mode: ExecutionMode,
            block_number: BlockNumber,
            now_ms: u64,
            req_id: Option<u64>,
        ) -> Self {
            Self {
                mode,
                block_number,
                now_ms,
                req_id,
            }
        }
    }

    #[cross_call(OCall)]
    pub trait OCalls {
        #[xcall(id = 1)]
        fn storage_root(&self) -> Option<Hash>;
        #[xcall(id = 2)]
        fn storage_get(&self, key: Vec<u8>) -> Option<Vec<u8>>;
        #[xcall(id = 3)]
        fn storage_commit(&mut self, root: Hash, changes: StorageChanges);
        #[xcall(id = 5)]
        fn log_to_server(&self, contract: AccountId, level: u8, message: String);
        #[xcall(id = 6)]
        fn emit_side_effects(&mut self, effects: ExecSideEffects);
        #[xcall(id = 7)]
        fn exec_context(&self) -> ExecContext;
        #[xcall(id = 8)]
        fn worker_pubkey(&self) -> [u8; 32];
        #[xcall(id = 9)]
        fn cache_get(&self, contract: Vec<u8>, key: Vec<u8>) -> Option<Vec<u8>>;
        #[xcall(id = 10)]
        fn cache_set(
            &self,
            contract: Vec<u8>,
            key: Vec<u8>,
            value: Vec<u8>,
        ) -> Result<(), StorageQuotaExceeded>;
        #[xcall(id = 11)]
        fn cache_set_expiration(&self, contract: Vec<u8>, key: Vec<u8>, expiration: u64);
        #[xcall(id = 12)]
        fn cache_remove(&self, contract: Vec<u8>, key: Vec<u8>) -> Option<Vec<u8>>;
        #[xcall(id = 13)]
        fn latest_system_code(&self) -> Vec<u8>;
        #[xcall(id = 14)]
        fn http_request(
            &self,
            contracr: AccountId,
            request: HttpRequest,
        ) -> Result<HttpResponse, HttpRequestError>;
    }
}

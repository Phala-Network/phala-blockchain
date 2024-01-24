#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(dead_code)]
mod types;
pub use types::*;

/// The trait that defines the basic cross-call interface.
pub trait CrossCall {
    fn cross_call(&self, id: u32, data: &[u8]) -> Vec<u8>;
}

/// The mutable version of `CrossCall`.
pub trait CrossCallMut {
    fn cross_call_mut(&mut self, call_id: u32, data: &[u8]) -> Vec<u8>;
}

/// A trait used by the generated ecall dispatcher for context setup
/// before invoking the ecall runtime implementations.
pub trait Executing {
    /// Executes a given function `f` immutably.
    fn execute<T>(&self, f: impl FnOnce() -> T) -> T;

    /// Executes a given function `f` mutably.
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
    use crate::types::{AccountId, Balance, BlockNumber, ExecutionMode, Hash, Weight};
    use pink_macro::cross_call;
    use scale::{Decode, Encode};

    /// Contains the parameters for a contract call or instantiation.
    #[derive(Encode, Decode, Clone, Debug)]
    pub struct TransactionArguments {
        /// The sender's AccountId.
        pub origin: AccountId,
        /// The amount of Balance transferred to the contract.
        pub transfer: Balance,
        /// The maximum gas amount for this transaction.
        pub gas_limit: Weight,
        /// Indicates whether the transaction requires gas. If `true`, no tokens are reserved for gas.
        pub gas_free: bool,
        /// The storage limit for this transaction. `None` indicates no limit.
        pub storage_deposit_limit: Option<Balance>,
        /// The balance to deposit to the caller address, used when the caller's balance is insufficient
        /// for a gas estimation.
        pub deposit: Balance,
    }

    /// Contains the full configuration for a cluster setup.
    #[derive(Encode, Decode, Clone, Debug)]
    pub struct ClusterSetupConfig {
        /// The unique identifier for the cluster.
        pub cluster_id: Hash,
        /// The cluster's `owner`.
        pub owner: AccountId,
        /// The balance deposited to the cluster's owner, to avoid needing an extra transfer operation
        /// while creating a cluster.
        pub deposit: Balance,
        /// The gas cost per unit of gas.
        pub gas_price: Balance,
        /// Storage price as per each item.
        pub deposit_per_item: Balance,
        /// Storage price as per byte.
        pub deposit_per_byte: Balance,
        /// The account amassing the gas and other fees in the cluster.
        pub treasury_account: AccountId,
        /// The code of the system contract.
        pub system_code: Vec<u8>,
    }

    /// The trait that defines the interface provided by the Pink runtime to the host.
    #[cross_call(ECall)]
    pub trait ECalls {
        /// Returns the ID of the cluster.
        #[xcall(id = 1)]
        fn cluster_id(&self) -> Hash;

        /// Initializes the cluster with the provided configuration.
        #[xcall(id = 2)]
        fn setup(&mut self, config: ClusterSetupConfig) -> Result<(), String>;

        /// Mint a specified amount of balance for account `who`.
        #[xcall(id = 3)]
        fn deposit(&mut self, who: AccountId, value: Balance);

        /// Sets the key of the cluster. Used for contract key derivation.
        #[xcall(id = 5)]
        fn set_key(&mut self, key: [u8; 64]);

        /// Returns the currently cluster key.
        #[xcall(id = 6)]
        fn get_key(&self) -> Option<[u8; 64]>;

        /// Uploads ink code WASM to the storage. When deterministic=true, it checks and
        /// forbids inderterministic instructions such as floating point ops in the code.
        #[xcall(id = 7)]
        fn upload_code(
            &mut self,
            account: AccountId,
            code: Vec<u8>,
            deterministic: bool,
        ) -> Result<Hash, String>;

        /// Uploads sidevm code to the account.
        #[xcall(id = 8)]
        fn upload_sidevm_code(&mut self, account: AccountId, code: Vec<u8>)
            -> Result<Hash, String>;

        /// Returns the sidevm code associated with a given hash.
        #[xcall(id = 9)]
        fn get_sidevm_code(&self, hash: Hash) -> Option<Vec<u8>>;

        /// Returns the address of the system contract.
        #[xcall(id = 11)]
        fn system_contract(&self) -> Option<AccountId>;

        /// Returns the free balance of the specified account.
        #[xcall(id = 14)]
        fn free_balance(&self, account: AccountId) -> Balance;

        /// Returns the total balance of the specified account.
        #[xcall(id = 15)]
        fn total_balance(&self, account: AccountId) -> Balance;

        /// Returns the hash of the code from the specified contract address.
        #[xcall(id = 16)]
        fn code_hash(&self, account: AccountId) -> Option<Hash>;

        /// Executes a contract instantiation with the provided arguments.
        #[xcall(id = 19)]
        fn contract_instantiate(
            &mut self,
            code_hash: Hash,
            input_data: Vec<u8>,
            salt: Vec<u8>,
            mode: ExecutionMode,
            tx_args: TransactionArguments,
        ) -> Vec<u8>;

        /// Executes a contract call with the specified parameters.
        #[xcall(id = 20)]
        fn contract_call(
            &mut self,
            contract: AccountId,
            input_data: Vec<u8>,
            mode: ExecutionMode,
            tx_args: TransactionArguments,
        ) -> Vec<u8>;

        /// Returns the git revision that compiled the runtime.
        #[xcall(id = 21)]
        fn git_revision(&self) -> String;

        /// Would be called when the cluster is created.
        #[xcall(id = 22)]
        fn on_genesis(&mut self);

        /// Would be called right after the runtime is upgraded. State migration
        /// should be done here.
        #[xcall(id = 23, since = "1.1")]
        fn on_runtime_upgrade(&mut self);

        /// Would be called once per block.
        #[xcall(id = 24, since = "1.2")]
        fn on_idle(&mut self, block_number: BlockNumber);
    }

    #[test]
    fn coverage() {
        dbg!(TransactionArguments {
            origin: [0u8; 32].into(),
            transfer: 0,
            gas_limit: 0,
            gas_free: false,
            storage_deposit_limit: None,
            deposit: 0,
        }
        .clone());
        dbg!(ClusterSetupConfig {
            cluster_id: [0u8; 32].into(),
            owner: [0u8; 32].into(),
            deposit: 0,
            gas_price: 0,
            deposit_per_item: 0,
            deposit_per_byte: 0,
            treasury_account: [0u8; 32].into(),
            system_code: vec![],
        }
        .clone());
    }
}

pub mod ocall {
    use super::{CrossCallMut, Executing, OCall};
    use crate::types::{AccountId, BlockNumber, ExecSideEffects, ExecutionMode, Hash};
    pub use pink::chain_extension::{JsCode, JsValue};
    use pink_macro::cross_call;
    use scale::{Decode, Encode};

    pub use pink::chain_extension::{
        BatchHttpResult, HttpRequest, HttpRequestError, HttpResponse, StorageQuotaExceeded,
    };
    pub type StorageChanges = Vec<(Vec<u8>, (Vec<u8>, i32))>;

    /// Information about the current execution context.
    #[derive(Decode, Encode, Clone, Default)]
    pub struct ExecContext {
        /// The execution mode.
        pub mode: ExecutionMode,
        /// The Phala block number which triggered the current execution.
        pub block_number: BlockNumber,
        /// The timestamp read from the Phala blockchain.
        pub now_ms: u64,
        /// The request ID if the current execution is triggered by a RPC request.
        pub req_id: Option<u64>,
    }

    /// The trait that defines the interface provided by the Pink runtime to the host for outwards bound calls.
    #[cross_call(OCall)]
    pub trait OCalls {
        /// Returns the storage root hash. This is used for building the storage trie.
        #[xcall(id = 1)]
        fn storage_root(&self) -> Option<Hash>;

        /// Fetches a value from storage.
        #[xcall(id = 2)]
        fn storage_get(&self, key: Vec<u8>) -> Option<Vec<u8>>;

        /// Commits changes to the storage and set the new storage root hash.
        #[xcall(id = 3)]
        fn storage_commit(&mut self, root: Hash, changes: StorageChanges);

        /// Sends a log message from a contract to the log collection server.
        #[xcall(id = 5)]
        fn log_to_server(&self, contract: AccountId, level: u8, message: String);

        /// Emits the side effects that occurred during contract execution.
        #[xcall(id = 6)]
        fn emit_side_effects(&mut self, effects: ExecSideEffects);

        /// Returns the current execution context.
        #[xcall(id = 7)]
        fn exec_context(&self) -> ExecContext;

        /// Returns the public key of the worker.
        #[xcall(id = 8)]
        fn worker_pubkey(&self) -> [u8; 32];

        /// Fetches a cache value that is associated with the specified contract and key.
        #[xcall(id = 9)]
        fn cache_get(&self, contract: Vec<u8>, key: Vec<u8>) -> Option<Vec<u8>>;

        /// Sets a cache value associated with the specified contract and key.
        /// Returns an error if the storage quota is exceeded.
        #[xcall(id = 10)]
        fn cache_set(
            &self,
            contract: Vec<u8>,
            key: Vec<u8>,
            value: Vec<u8>,
        ) -> Result<(), StorageQuotaExceeded>;

        /// Sets an expiration time (in seconds relative to present) for the cache value associated
        /// with the specified contract and key.
        #[xcall(id = 11)]
        fn cache_set_expiration(&self, contract: Vec<u8>, key: Vec<u8>, expiration: u64);

        /// Removes a cache value associated with a specified contract and key.
        /// Returns the previously associated value (if any).
        #[xcall(id = 12)]
        fn cache_remove(&self, contract: Vec<u8>, key: Vec<u8>) -> Option<Vec<u8>>;

        /// Returns the latest available system contract code.
        #[xcall(id = 13)]
        fn latest_system_code(&self) -> Vec<u8>;

        /// Performs a HTTP(S) request on behalf of the contract.
        /// Returns an HTTP response or an error.
        #[xcall(id = 14)]
        fn http_request(
            &self,
            contract: AccountId,
            request: HttpRequest,
        ) -> Result<HttpResponse, HttpRequestError>;

        /// Performs a batch of HTTP(S) requests on behalf of the contract within a specified timeout period.
        /// Returns the collective results of all HTTP requests.
        #[xcall(id = 15)]
        fn batch_http_request(
            &self,
            contract: AccountId,
            requests: Vec<HttpRequest>,
            timeout_ms: u64,
        ) -> BatchHttpResult;

        /// Emits a system event block that includes all events extracted from the current contract call.
        #[xcall(id = 16)]
        fn emit_system_event_block(&self, number: u64, encoded_block: Vec<u8>);

        /// Gets the nonce of the current contract call (if available).
        #[xcall(id = 17)]
        fn contract_call_nonce(&self) -> Option<Vec<u8>>;

        /// Returns the address of the entry contract if the execution was triggered by a contract call.
        #[xcall(id = 18)]
        fn entry_contract(&self) -> Option<AccountId>;

        /// Evaluates a set of JavaScript code using the QuickJS engine running in SideVM.
        /// Returns the output of the evaluation.
        #[xcall(id = 19)]
        fn js_eval(&self, contract: AccountId, codes: Vec<JsCode>, args: Vec<String>) -> JsValue;

        /// Get the origin of the transaction (if available).
        #[xcall(id = 20)]
        fn origin(&self) -> Option<AccountId>;
    }
}

#[test]
fn coverage_ctypes() {
    #[derive(Debug, Default, Clone)]
    struct Test {
        _fsid_t: types::__fsid_t,
        _imaxdiv_t: types::imaxdiv_t,
        _max_align_t: types::max_align_t,
        _ocalls_t: types::ocalls_t,
        _config_t: types::config_t,
        _ecalls_t: types::ecalls_t,
    }

    dbg!(Test::default().clone());
}

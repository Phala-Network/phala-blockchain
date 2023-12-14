use assert_matches::assert_matches;
use phala_crypto::sr25519::Persistence;
use pink_capi::v1::ecall::{ECalls, TransactionArguments};
use pink_runner::{
    local_cache,
    types::{AccountId, Balance, ExecutionMode, Weight},
};
use sp_runtime::{app_crypto::sr25519, AccountId32};

use test_cluster::TestCluster;

pub const ALICE: AccountId32 = AccountId32::new([1u8; 32]);
pub const BOB: AccountId32 = AccountId32::new([2u8; 32]);
pub const ENOUGH: Balance = u128::MAX / 2;

fn tx_args() -> TransactionArguments {
    TransactionArguments {
        origin: ALICE.clone(),
        transfer: 0,
        gas_limit: Weight::MAX,
        gas_free: true,
        storage_deposit_limit: None,
        deposit: 0,
    }
}

fn alice() -> AccountId {
    ALICE.clone()
}

#[test]
fn test_ink_flip() {
    let mut cluster = TestCluster::for_test();

    let code_hash = cluster
        .upload_code(
            alice(),
            include_bytes!("./fixtures/flip/flip.wasm").to_vec(),
            true,
        )
        .unwrap();

    let result = cluster.instantiate(
        code_hash,
        0x9bae9d5e,
        true,
        vec![],
        cluster.mode(),
        tx_args(),
    );

    insta::assert_debug_snapshot!(result);
    insta::assert_debug_snapshot!(cluster.effects.take());
    let contract = result.result.unwrap().account_id;

    let result = cluster.call_typed::<_, bool>(
        &contract,
        0x2f865bd9, // get
        (),
        cluster.mode(),
        tx_args(),
    );
    assert_matches!(result, Ok(true));

    let result = cluster.call_typed::<_, ()>(
        &contract,
        0x633aa551, // flip
        (),
        cluster.mode(),
        tx_args(),
    );
    assert_matches!(result, Ok(()));

    let result = cluster.call_typed::<_, bool>(
        &contract,
        0x2f865bd9, // get
        (),
        cluster.mode(),
        tx_args(),
    );
    assert_matches!(result, Ok(false));

    let result = cluster.call_typed::<_, (u32, u128)>(
        &contract,
        0xf7dff04c, // echo
        (42u32, 24u128),
        cluster.mode(),
        tx_args(),
    );
    assert_matches!(result, Ok((42, 24)));
}

#[test]
fn test_ink_cross_contract_instantiate() {
    let mut cluster = TestCluster::for_test();

    let _code_hash = cluster
        .upload_code(
            alice(),
            include_bytes!("./fixtures/flip/flip.wasm").to_vec(),
            true,
        )
        .unwrap();

    let code_hash = cluster
        .upload_code(
            alice(),
            include_bytes!("./fixtures/cross/cross.wasm").to_vec(),
            true,
        )
        .unwrap();

    let mut args = tx_args();
    args.transfer = ENOUGH / 100;
    let result = cluster.instantiate(code_hash, 0x9bae9d5e, true, vec![], cluster.mode(), args);
    insta::assert_debug_snapshot!(result);
    insta::assert_debug_snapshot!(cluster.effects.take());
    let contract = result.result.unwrap().account_id;

    let result = cluster.call_typed::<_, bool>(
        &contract,
        0xc3220014, // get
        (),
        cluster.mode(),
        tx_args(),
    );
    assert_matches!(result, Ok(true));
}

fn generate_key() -> [u8; 64] {
    let key = sr25519::Pair::restore_from_seed(&[42; 32]);
    key.dump_secret_key()
}

fn test_with_wasm(wasm: &[u8], constructor: u32, message: u32, query: bool) {
    let mut cluster = TestCluster::for_test();
    cluster.set_key(generate_key());
    if query {
        cluster.context.mode = ExecutionMode::Query;
    }

    let code_hash = cluster.upload_code(alice(), wasm.to_vec(), true).unwrap();

    let contract = cluster
        .instantiate_typed(
            code_hash,
            constructor,
            (),
            vec![],
            cluster.mode(),
            tx_args(),
        )
        .expect("Failed to instantiate contract");

    local_cache::apply_quotas([(contract.as_ref(), 1024)]);

    let result = cluster.call_typed::<_, ()>(&contract, message, (), cluster.mode(), tx_args());
    assert_matches!(result, Ok(()));
}

#[test]
fn test_signing() {
    test_with_wasm(
        include_bytes!("./fixtures/signing/signing.wasm"),
        0xed4b9d1b,
        0x928b2036,
        true,
    );
}

#[test]
fn test_logging() {
    env_logger::init();
    test_with_wasm(
        include_bytes!("./fixtures/logging.wasm"),
        0xed4b9d1b,
        0x928b2036,
        false,
    );
}

#[test]
fn test_use_cache() {
    test_with_wasm(
        include_bytes!("./fixtures/use_cache/use_cache.wasm"),
        0xed4b9d1b,
        0x928b2036,
        true,
    );
}

#[test]
#[ignore = "for dev"]
fn test_qjs() {
    use scale::{Decode, Encode};

    env_logger::init();

    let mut cluster = TestCluster::for_test();
    cluster.set_key(generate_key());

    let checker = [];
    let qjs = [];
    // let checker = include_bytes!("../../../../e2e/res/check_system/target/ink/check_system.wasm");
    // let qjs = include_bytes!("qjs.wasm");

    #[derive(Debug, Encode, Decode)]
    pub enum Output {
        String(String),
        Bytes(Vec<u8>),
        Undefined,
    }

    let checker_hash = cluster
        .upload_code(alice(), checker.to_vec(), true)
        .unwrap();
    let qjs_hash = cluster.upload_code(alice(), qjs.to_vec(), false).unwrap();

    let contract = cluster
        .instantiate_typed(
            checker_hash,
            0xed4b9d1b,
            (),
            vec![],
            cluster.mode(),
            tx_args(),
        )
        .expect("Failed to instantiate contract");

    local_cache::apply_quotas([(contract.as_ref(), 1024)]);

    let js = r#"
    (function(){
        console.log("Hello, World!");
        // return scriptArgs[1];
        return new Uint8Array([21, 31]);
    })()
    "#;
    let t0 = std::time::Instant::now();
    let args: Vec<String> = vec!["Hello".to_string(), "World".to_string()];
    cluster.context.mode = ExecutionMode::Query;
    type LangError = ();
    let result = cluster
        .call_typed::<_, Result<Result<Output, String>, LangError>>(
            &contract,
            0xf32e54c5,
            (qjs_hash, js, args),
            cluster.mode(),
            tx_args(),
        )
        .expect("Failed to call contract");
    println!("evaluate result={result:?}, dt={:?}", t0.elapsed());
}

mod test_cluster {
    use pink_capi::v1::{
        ecall::ECalls,
        ocall::{
            BatchHttpResult, ExecContext, HttpRequest, HttpRequestError, HttpResponse, JsCode,
            JsValue, OCalls, StorageChanges,
        },
        CrossCall, CrossCallMut, ECall,
    };
    use pink_runner::{
        local_cache::{self, StorageQuotaExceeded},
        runtimes::v1::{using_ocalls, Runtime},
        storage::ClusterStorage,
        types::{AccountId, ExecSideEffects, ExecutionMode, Hash, TransactionArguments},
    };
    use scale::{Decode, Encode};
    use sp_runtime::DispatchError;

    use crate::ENOUGH;

    use super::ALICE;

    pub type ContractExecResult = pink::ContractExecResult;
    pub type ContractInstantiateResult = pink::ContractInstantiateResult;

    pub struct TestCluster {
        pub storage: ClusterStorage,
        pub context: ExecContext,
        pub worker_pubkey: [u8; 32],
        pub runtime: Runtime,
        pub(crate) effects: Option<ExecSideEffects>,
    }

    impl Clone for TestCluster {
        fn clone(&self) -> Self {
            Self {
                storage: self.storage.clone(),
                context: self.context.clone(),
                runtime: unsafe { self.runtime.dup() },
                worker_pubkey: self.worker_pubkey,
                effects: None,
            }
        }
    }

    impl TestCluster {
        pub fn for_test() -> Self {
            let storage = ClusterStorage::default();
            let context = ExecContext {
                mode: ExecutionMode::Transaction,
                block_number: 1,
                now_ms: 1,
                req_id: None,
            };
            let runtime = Runtime::from_fn(
                pink::capi::__pink_runtime_init,
                std::ptr::null_mut(),
                pink::version(),
            );
            let mut me = Self {
                storage,
                context,
                runtime,
                effects: None,
                worker_pubkey: [1; 32],
            };
            me.deposit(ALICE.clone(), ENOUGH);
            me
        }
    }

    impl TestCluster {
        pub fn mode(&self) -> ExecutionMode {
            self.context.mode
        }

        fn execute_mut<T>(&mut self, f: impl FnOnce() -> T) -> T {
            using_ocalls(self, f)
        }

        fn execute<T>(&self, f: impl FnOnce() -> T) -> T {
            let mut tmp = self.clone();
            using_ocalls(&mut tmp, f)
        }

        pub fn instantiate<I: Encode>(
            &mut self,
            code_hash: Hash,
            selector: u32,
            input: I,
            salt: Vec<u8>,
            mode: ExecutionMode,
            tx_args: TransactionArguments,
        ) -> ContractInstantiateResult {
            let input_data = Encode::encode(&(selector.to_be_bytes(), input));
            let result = self.contract_instantiate(code_hash, input_data, salt, mode, tx_args);
            Decode::decode(&mut &result[..]).unwrap()
        }

        pub fn instantiate_typed<I: Encode>(
            &mut self,
            code_hash: Hash,
            selector: u32,
            input: I,
            salt: Vec<u8>,
            mode: ExecutionMode,
            tx_args: TransactionArguments,
        ) -> Result<AccountId, DispatchError> {
            let result = self.instantiate(code_hash, selector, input, salt, mode, tx_args);
            match result.result {
                Ok(ret) => {
                    if ret.result.did_revert() {
                        Err(DispatchError::Other("Contract instantiation reverted"))
                    } else {
                        Ok(ret.account_id)
                    }
                }
                Err(err) => Err(err),
            }
        }

        pub fn call<I: Encode>(
            &mut self,
            contract: &AccountId,
            selector: u32,
            input: I,
            mode: ExecutionMode,
            tx_args: TransactionArguments,
        ) -> ContractExecResult {
            let input_data = Encode::encode(&(selector.to_be_bytes(), input));
            let result = self.contract_call(contract.clone(), input_data, mode, tx_args);
            Decode::decode(&mut &result[..]).unwrap()
        }

        pub fn call_typed<I: Encode, T: Decode>(
            &mut self,
            contract: &AccountId,
            selector: u32,
            input: I,
            mode: ExecutionMode,
            tx_args: TransactionArguments,
        ) -> Result<T, DispatchError> {
            let result = self.call(contract, selector, input, mode, tx_args);
            match result.result {
                Ok(ret) => {
                    if ret.did_revert() {
                        Err(DispatchError::Other("Contract reverted"))
                    } else {
                        Decode::decode(&mut &ret.data[..]).map_err(|_| {
                            DispatchError::Other("Failed to decode contract return value")
                        })
                    }
                }
                Err(e) => Err(e),
            }
        }
    }

    impl OCalls for TestCluster {
        fn storage_root(&self) -> Option<Hash> {
            self.storage.root()
        }

        fn storage_get(&self, key: Vec<u8>) -> Option<Vec<u8>> {
            self.storage.get(&key).map(|(_rc, val)| val.clone())
        }

        fn storage_commit(&mut self, root: Hash, changes: StorageChanges) {
            self.storage.commit(root, changes);
        }

        fn log_to_server(&self, contract: AccountId, level: u8, message: String) {
            log::log!(
                match level {
                    0 => log::Level::Error,
                    1 => log::Level::Warn,
                    2 => log::Level::Info,
                    3 => log::Level::Debug,
                    4 => log::Level::Trace,
                    _ => log::Level::Info,
                },
                "log_server: contract={:?}, message={}",
                contract,
                message
            )
        }

        fn emit_side_effects(&mut self, effects: ExecSideEffects) {
            self.effects = Some(effects)
        }

        fn exec_context(&self) -> ExecContext {
            self.context.clone()
        }

        fn worker_pubkey(&self) -> [u8; 32] {
            self.worker_pubkey
        }

        fn cache_get(&self, contract: Vec<u8>, key: Vec<u8>) -> Option<Vec<u8>> {
            local_cache::get(&contract, &key)
        }

        fn cache_set(
            &self,
            contract: Vec<u8>,
            key: Vec<u8>,
            value: Vec<u8>,
        ) -> Result<(), StorageQuotaExceeded> {
            local_cache::set(&contract, &key, &value)
        }

        fn cache_set_expiration(&self, contract: Vec<u8>, key: Vec<u8>, expiration: u64) {
            local_cache::set_expiration(&contract, &key, expiration)
        }

        fn cache_remove(&self, contract: Vec<u8>, key: Vec<u8>) -> Option<Vec<u8>> {
            local_cache::remove(&contract, &key)
        }

        fn latest_system_code(&self) -> Vec<u8> {
            vec![]
        }

        fn http_request(
            &self,
            _contract: AccountId,
            request: HttpRequest,
        ) -> Result<HttpResponse, HttpRequestError> {
            pink_extension_runtime::http_request(request, 10 * 1000)
        }

        fn batch_http_request(
            &self,
            _: AccountId,
            requests: Vec<HttpRequest>,
            timeout_ms: u64,
        ) -> BatchHttpResult {
            pink_extension_runtime::batch_http_request(requests, timeout_ms)
        }

        fn emit_system_event_block(&self, number: u64, _encoded_block: Vec<u8>) {
            log::info!("emit_system_event_block: number={}", number,);
        }

        fn contract_call_nonce(&self) -> Option<Vec<u8>> {
            None
        }

        fn entry_contract(&self) -> Option<AccountId> {
            None
        }

        fn js_eval(
            &self,
            _contract: AccountId,
            _codes: Vec<JsCode>,
            _args: Vec<String>,
        ) -> JsValue {
            JsValue::Exception("Not implemented".to_string())
        }

        fn origin(&self) -> Option<AccountId> {
            None
        }
    }

    impl CrossCall for TestCluster {
        fn cross_call(&self, call_id: u32, data: &[u8]) -> Vec<u8> {
            self.execute(move || self.runtime.ecall(call_id, data))
        }
    }

    impl CrossCallMut for TestCluster {
        fn cross_call_mut(&mut self, call_id: u32, data: &[u8]) -> Vec<u8> {
            let runtime = unsafe { self.runtime.dup() };
            self.execute_mut(move || runtime.ecall(call_id, data))
        }
    }

    impl ECall for TestCluster {}
}

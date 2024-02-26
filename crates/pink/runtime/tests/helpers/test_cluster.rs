use std::collections::BTreeMap;

use super::{
    storage::ClusterStorage,
    xcalls::{using_ocalls, Runtime},
    ALICE, ENOUGH, TREASURY,
};
use log::{error, info};
use phala_crypto::sr25519::Persistence;
use pink_capi::{
    types::{AccountId, ExecSideEffects, ExecutionMode, Hash},
    v1::{
        ecall::{ClusterSetupConfig, ECalls, TransactionArguments},
        ocall::{
            BatchHttpResult, ExecContext, HttpRequest, HttpRequestError, HttpResponse, JsCode,
            JsValue, OCalls, SgxQuote, StorageChanges,
        },
        CrossCall, CrossCallMut, ECall,
    },
};
use pink_extension::ConvertTo;
use pink_extension::PinkEvent;
use pink_extension_runtime::local_cache::{self, StorageQuotaExceeded};
use scale::{Decode, Encode};
use sp_core::Pair;
use sp_runtime::DispatchError;

pub type ContractExecResult = pink::ContractExecResult;
pub type ContractInstantiateResult = pink::ContractInstantiateResult;

fn test_key() -> [u8; 64] {
    let pair = sp_core::sr25519::Pair::from_seed(&[42u8; 32]);
    pair.dump_secret_key()
}

pub struct TestCluster {
    pub storage: ClusterStorage,
    pub worker_pubkey: [u8; 32],
    pub runtime: Runtime,
    pub effects: Option<ExecSideEffects>,
    pub other_operations: BTreeMap<AccountId, Vec<&'static str>>,
    pub n_ink_events: usize,
}

impl Clone for TestCluster {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            runtime: self.runtime.dup(),
            worker_pubkey: self.worker_pubkey,
            effects: self.effects.clone(),
            other_operations: self.other_operations.clone(),
            n_ink_events: self.n_ink_events,
        }
    }
}

impl TestCluster {
    pub fn bare() -> Self {
        let storage = ClusterStorage::default();
        let runtime = Runtime::from_fn(pink::capi::__pink_runtime_init);
        Self {
            storage,
            runtime,
            worker_pubkey: [1; 32],
            effects: None,
            other_operations: BTreeMap::new(),
            n_ink_events: 0,
        }
    }

    pub fn for_test() -> Self {
        pink_extension_runtime::mock_ext::mock_all_ext();
        let test_key = test_key();
        let mut me = Self::bare();
        me.tx().on_genesis();
        me.tx().on_idle(1);
        me.tx().set_key(test_key);
        assert_eq!(me.tx().get_key(), Some(test_key));
        me.tx()
            .setup(ClusterSetupConfig {
                cluster_id: Default::default(),
                owner: ALICE.clone(),
                deposit: ENOUGH,
                gas_price: 1,
                deposit_per_item: 1,
                deposit_per_byte: 1,
                treasury_account: TREASURY.clone(),
                system_code: include_bytes!("../fixtures/system/system.wasm").to_vec(),
            })
            .expect("Failed to setup cluster");
        me.tx().on_runtime_upgrade();
        me.tx()
            .upload_sidevm_code(
                ALICE.clone(),
                include_bytes!("../fixtures/logging.wasm").to_vec(),
            )
            .expect("Failed to upload logger code");
        me
    }
}

impl TestCluster {
    pub fn query(&mut self) -> Exec {
        self.use_mode(ExecutionMode::Query)
    }

    pub fn estimate(&mut self) -> Exec {
        self.use_mode(ExecutionMode::Estimating)
    }

    pub fn tx(&mut self) -> Exec {
        self.use_mode(ExecutionMode::Transaction)
    }

    pub fn use_mode(&mut self, mode: ExecutionMode) -> Exec {
        Exec {
            cluster: self,
            context: context(mode),
        }
    }

    fn push_operation(&mut self, contract: impl ConvertTo<AccountId>, operation: &'static str) {
        self.other_operations
            .entry(contract.convert_to())
            .or_default()
            .push(operation);
    }

    pub fn stat_ink_events(&mut self, n: usize) {
        self.n_ink_events += n;
    }

    pub fn apply_pink_events(&mut self, pink_events: Vec<(AccountId, PinkEvent)>) {
        for (origin, event) in pink_events {
            let event_name = event.name();
            macro_rules! ensure_system {
                () => {
                    if Some(&origin) != self.query().system_contract().as_ref() {
                        error!(
                            "Unpermitted operation from {:?}, operation={:?}",
                            &origin, &event_name
                        );
                        continue;
                    }
                };
            }
            match event {
                PinkEvent::SetHook {
                    contract: target_contract,
                    ..
                } => {
                    ensure_system!();
                    self.push_operation(target_contract, "Set hook");
                }
                PinkEvent::DeploySidevmTo {
                    contract: target_contract,
                    ..
                } => {
                    ensure_system!();
                    self.push_operation(target_contract, "Deploy sidevm");
                }
                PinkEvent::SidevmMessage(_) => self.push_operation(origin.clone(), "Push message"),
                PinkEvent::CacheOp(op) => {
                    local_cache::apply_cache_op(&origin, op);
                }
                PinkEvent::StopSidevm => {
                    self.push_operation(origin.clone(), "Stop sidevm");
                }
                PinkEvent::ForceStopSidevm {
                    contract: target_contract,
                } => {
                    ensure_system!();
                    self.push_operation(target_contract, "Force stop sidevm");
                }
                PinkEvent::SetLogHandler(handler) => {
                    ensure_system!();
                    info!("Set logger to {handler:?}");
                    self.push_operation(origin.clone(), "Set logger");
                }
                PinkEvent::SetContractWeight { contract, weight } => {
                    ensure_system!();
                    info!("Set contract weight for {contract:?} to {weight:?}");
                    local_cache::apply_quotas([(contract.as_ref(), weight as usize)]);
                }
                PinkEvent::UpgradeRuntimeTo { version } => {
                    ensure_system!();
                    info!("Upgrade runtime to {version:?}");
                    self.push_operation(origin.clone(), "Upgrade runtime");
                }
                PinkEvent::SidevmOperation(event) => {
                    ensure_system!();
                    info!("Sidevm operation: {:?}", event);
                    self.push_operation(origin.clone(), "Sidevm operation");
                }
                PinkEvent::SetJsRuntime(_code_hash) => {
                    ensure_system!();
                    self.push_operation(origin.clone(), "Set JsRuntime");
                }
            }
        }
    }
}

pub struct Exec<'a> {
    cluster: &'a mut TestCluster,
    context: ExecContext,
}

fn context(mode: ExecutionMode) -> ExecContext {
    ExecContext {
        mode,
        block_number: 1,
        now_ms: 1,
        req_id: None,
    }
}

impl Exec<'_> {
    pub fn mode(&self) -> ExecutionMode {
        self.context.mode
    }

    fn execute_mut<T>(&mut self, f: impl FnOnce() -> T) -> T {
        using_ocalls(self, f)
    }

    fn execute<T>(&self, f: impl FnOnce() -> T) -> T {
        let mut cluster = self.cluster.clone();
        let mut tmp = Exec {
            cluster: &mut cluster,
            context: self.context.clone(),
        };
        using_ocalls(&mut tmp, f)
    }
}

impl OCalls for Exec<'_> {
    fn storage_root(&self) -> Option<Hash> {
        self.cluster.storage.root()
    }

    fn storage_get(&self, key: Vec<u8>) -> Option<Vec<u8>> {
        self.cluster.storage.get(&key).map(|(_rc, val)| val.clone())
    }

    fn storage_commit(&mut self, root: Hash, changes: StorageChanges) {
        if self.mode().is_estimating() {
            return;
        }
        self.cluster.storage.commit(root, changes);
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
        self.cluster.effects = Some(effects)
    }

    fn exec_context(&self) -> ExecContext {
        self.context.clone()
    }

    fn worker_pubkey(&self) -> [u8; 32] {
        self.cluster.worker_pubkey
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
        if self.mode().is_estimating() {
            return Ok(());
        }
        local_cache::set(&contract, &key, &value)
    }

    fn cache_set_expiration(&self, contract: Vec<u8>, key: Vec<u8>, expiration: u64) {
        if self.mode().is_estimating() {
            return;
        }
        local_cache::set_expiration(&contract, &key, expiration)
    }

    fn cache_remove(&self, contract: Vec<u8>, key: Vec<u8>) -> Option<Vec<u8>> {
        if self.mode().is_estimating() {
            return None;
        }
        local_cache::remove(&contract, &key)
    }

    fn latest_system_code(&self) -> Vec<u8> {
        include_bytes!("../fixtures/system/system-0xffff.wasm").to_vec()
    }

    fn http_request(
        &self,
        _contract: AccountId,
        request: HttpRequest,
    ) -> Result<HttpResponse, HttpRequestError> {
        if request.url.ends_with("/timeout") {
            return Err(HttpRequestError::Timeout);
        }
        if request.url.ends_with("/not_allowed") {
            return Err(HttpRequestError::NotAllowed);
        }
        if request.url.ends_with("/404") {
            return Ok(HttpResponse::not_found());
        }
        if request.url.ends_with("/large") {
            return Ok(HttpResponse::ok(vec![0; 8 * 1024 * 1024]));
        }
        Ok(HttpResponse::ok(request.url.into_bytes()))
    }

    fn batch_http_request(
        &self,
        _: AccountId,
        requests: Vec<HttpRequest>,
        timeout_ms: u64,
    ) -> BatchHttpResult {
        if timeout_ms == 0 {
            return Err(HttpRequestError::Timeout);
        }
        let responses = requests
            .into_iter()
            .map(|request| self.http_request(ALICE.clone(), request))
            .collect();
        Ok(responses)
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

    fn js_eval(&self, _contract: AccountId, _codes: Vec<JsCode>, _args: Vec<String>) -> JsValue {
        JsValue::Exception("Not implemented".to_string())
    }

    fn origin(&self) -> Option<AccountId> {
        None
    }

    fn worker_sgx_quote(&self) -> Option<SgxQuote> {
        None
    }
}

impl CrossCall for Exec<'_> {
    fn cross_call(&self, call_id: u32, data: &[u8]) -> Vec<u8> {
        self.execute(move || self.cluster.runtime.ecall(call_id, data))
    }
}

impl CrossCallMut for Exec<'_> {
    fn cross_call_mut(&mut self, call_id: u32, data: &[u8]) -> Vec<u8> {
        let runtime = self.cluster.runtime.dup();
        self.execute_mut(move || runtime.ecall(call_id, data))
    }
}

impl ECall for Exec<'_> {}

impl Exec<'_> {
    pub fn instantiate<I: Encode>(
        &mut self,
        code_hash: Hash,
        selector: u32,
        input: I,
        salt: Vec<u8>,
        tx_args: TransactionArguments,
    ) -> ContractInstantiateResult {
        let mode = self.mode();
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
        tx_args: TransactionArguments,
    ) -> Result<AccountId, DispatchError> {
        let result = self.instantiate(code_hash, selector, input, salt, tx_args);
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
        tx_args: TransactionArguments,
    ) -> ContractExecResult {
        let mode = self.mode();
        let input_data = Encode::encode(&(selector.to_be_bytes(), input));
        let result = self.contract_call(contract.clone(), input_data, mode, tx_args);
        Decode::decode(&mut &result[..]).unwrap()
    }

    pub fn call_typed<I: Encode, T: Decode>(
        &mut self,
        contract: &AccountId,
        selector: u32,
        input: I,
        tx_args: TransactionArguments,
    ) -> Result<T, DispatchError> {
        let result = self.call(contract, selector, input, tx_args);
        match result.result {
            Ok(ret) => {
                if ret.did_revert() {
                    Err(DispatchError::Other("Contract reverted"))
                } else {
                    Decode::decode(&mut &ret.data[..])
                        .map_err(|_| DispatchError::Other("Failed to decode contract return value"))
                }
            }
            Err(e) => Err(e),
        }
    }
}

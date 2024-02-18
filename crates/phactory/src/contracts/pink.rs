use once_cell::sync::Lazy;
use sgx_attestation::SgxQuote;
use std::sync::Mutex;
use std::{convert::TryInto, time::Duration};

use crate::{
    contracts::{self, block_on_run_module, QueryContext, TransactionContext},
    system::{TransactionError, TransactionResult},
};
use anyhow::{Context, Result};
use parity_scale_codec::Encode;
use phala_crypto::sr25519::Persistence;
use phala_mq::{ContractClusterId, MessageOrigin};
use phala_types::{contract::messaging::ResourceType, SignedContentType};
use pink::{
    capi::v1::{
        ecall::{ECalls, ECallsRo},
        ocall::{
            BatchHttpResult, ExecContext, HttpRequest, HttpRequestError, HttpResponse, OCalls,
            StorageChanges,
        },
    },
    local_cache::{self, StorageQuotaExceeded},
    runtimes::v1::{get_runtime, using_ocalls},
    types::{BlockNumber, ExecutionMode},
};
use pink_extension::chain_extension::{JsCode, JsValue};
use serde::{Deserialize, Serialize};
use sidevm::{
    service::{Command as SidevmCommand, CommandSender, Metric, SystemMessage},
    WasmEngine, WasmModule,
};
use sp_core::{blake2_256, sr25519, twox_64};

use ::pink::{
    capi::v1,
    constants::WEIGHT_REF_TIME_PER_SECOND,
    types::{AccountId, Balance, ExecSideEffects, Hash, TransactionArguments},
};
use tracing::info;

pub use phactory_api::contracts::{Query, QueryError, Response};
pub use phala_types::contract::InkCommand;

use super::ContractsKeeper;

pub(crate) mod http_counters;

#[derive(Serialize, Deserialize, Default, Clone, ::scale_info::TypeInfo)]
pub struct ClusterConfig {
    pub log_handler: Option<AccountId>,
    pub runtime_version: (u32, u32),
    pub secret_salt: [u8; 32],
    #[serde(default)]
    pub js_runtime: Option<Hash>,
}

#[derive(Serialize, Deserialize, Clone, ::scale_info::TypeInfo)]
pub struct Cluster {
    pub id: ContractClusterId,
    pub config: ClusterConfig,
    pub storage: pink::storage::ClusterStorage,
}

pub struct RuntimeHandleMut<'a> {
    cluster: &'a mut Cluster,
    logger: Option<CommandSender>,
    pub(crate) effects: Option<ExecSideEffects>,
}

impl RuntimeHandleMut<'_> {
    fn readonly(&self) -> RuntimeHandle {
        RuntimeHandle {
            cluster: self.cluster,
            logger: self.logger.clone(),
        }
    }
    fn execute_mut<T>(&mut self, f: impl FnOnce((u32, u32)) -> T) -> T {
        let ver = self.readonly().runtime_version();
        using_ocalls(self, || f(ver))
    }

    pub fn call(
        &mut self,
        contract: AccountId,
        input_data: Vec<u8>,
        mode: ExecutionMode,
        tx_args: TransactionArguments,
    ) -> Vec<u8> {
        context::using_entry(contract.clone(), tx_args.origin.clone(), move || {
            self.contract_call(contract, input_data, mode, tx_args)
        })
    }

    pub fn instantiate(
        &mut self,
        code_hash: Hash,
        instantiate_data: Vec<u8>,
        salt: Vec<u8>,
        mode: ExecutionMode,
        tx_args: TransactionArguments,
    ) -> Vec<u8> {
        let buf = phala_types::contract::contract_id_preimage(
            tx_args.origin.as_ref(),
            code_hash.as_ref(),
            self.readonly().cluster_id().as_ref(),
            &salt,
        );
        let entry = AccountId::from(blake2_256(&buf));
        context::using_entry(entry, tx_args.origin.clone(), move || {
            self.contract_instantiate(code_hash, instantiate_data, salt, mode, tx_args)
        })
    }
}

pub struct RuntimeHandle<'a> {
    cluster: &'a Cluster,
    logger: Option<CommandSender>,
}

impl RuntimeHandle<'_> {
    fn dup(&self) -> RuntimeHandle {
        RuntimeHandle {
            cluster: self.cluster,
            logger: self.logger.clone(),
        }
    }
    fn execute<T>(&self, f: impl FnOnce((u32, u32)) -> T) -> T {
        using_ocalls(&mut self.dup(), || f(self.runtime_version()))
    }

    fn runtime_version(&self) -> (u32, u32) {
        self.cluster.config.runtime_version
    }
}

pub(crate) mod context {
    use std::time::{Duration, Instant};

    use anyhow::{anyhow, Result};
    use phala_types::{wrap_content_to_sign, AttestationProvider, SignedContentType};
    use pink::{
        capi::v1::ocall::ExecContext,
        types::{AccountId, BlockNumber, ExecutionMode},
    };
    use sgx_attestation::SgxQuote;
    use sidevm::OutgoingRequestChannel;
    use sp_core::Pair;

    use crate::{contracts::ContractsKeeper, system::WorkerIdentityKey, ChainStorage};

    environmental::environmental!(exec_context: trait GetContext);

    pub trait GetContext {
        fn chain_storage(&self) -> &ChainStorage;
        fn exec_context(&self) -> ExecContext;
        fn worker_pubkey(&self) -> [u8; 32];
        fn worker_identity_key(&self) -> &WorkerIdentityKey;
        fn call_elapsed(&self) -> Duration;
        fn sidevm_query(
            &self,
            origin: [u8; 32],
            vmid: [u8; 32],
            input: Vec<u8>,
            timeout: Duration,
        ) -> Result<Vec<u8>>;
        fn sidevm_event_tx(&self) -> OutgoingRequestChannel;
        fn worker_sgx_quote(&self) -> Option<SgxQuote>;
    }

    pub struct ContractExecContext {
        pub mode: ExecutionMode,
        pub now_ms: u64,
        pub block_number: BlockNumber,
        pub worker_identity_key: WorkerIdentityKey,
        pub chain_storage: ChainStorage,
        pub contracts: ContractsKeeper,
        pub start_at: Instant,
        pub req_id: u64,
        pub sidevm_event_tx: OutgoingRequestChannel,
        pub attestation_provider: Option<AttestationProvider>,
    }

    impl ContractExecContext {
        #[allow(clippy::too_many_arguments)]
        pub fn new(
            mode: ExecutionMode,
            now_ms: u64,
            block_number: BlockNumber,
            worker_identity_key: WorkerIdentityKey,
            chain_storage: ChainStorage,
            req_id: u64,
            contracts: ContractsKeeper,
            sidevm_event_tx: OutgoingRequestChannel,
            attestation_provider: Option<AttestationProvider>,
        ) -> Self {
            Self {
                mode,
                now_ms,
                block_number,
                worker_identity_key,
                chain_storage,
                start_at: Instant::now(),
                req_id,
                contracts,
                sidevm_event_tx,
                attestation_provider,
            }
        }
    }

    impl GetContext for ContractExecContext {
        fn chain_storage(&self) -> &ChainStorage {
            &self.chain_storage
        }

        fn exec_context(&self) -> ExecContext {
            ExecContext {
                mode: self.mode,
                block_number: self.block_number,
                now_ms: self.now_ms,
                req_id: Some(self.req_id),
            }
        }

        fn worker_pubkey(&self) -> [u8; 32] {
            self.worker_identity_key.public().0
        }

        fn worker_identity_key(&self) -> &WorkerIdentityKey {
            &self.worker_identity_key
        }

        fn call_elapsed(&self) -> Duration {
            self.start_at.elapsed()
        }

        fn sidevm_query(
            &self,
            origin: [u8; 32],
            vmid: [u8; 32],
            payload: Vec<u8>,
            timeout: Duration,
        ) -> Result<Vec<u8>> {
            let contract_id = AccountId::new(vmid);
            let contract = self
                .contracts
                .get(&contract_id)
                .ok_or(anyhow!("Contract not found: {contract_id:?}"))?;
            let tx = contract
                .sidevm_handle()
                .ok_or(anyhow!("Sidevm not found: {contract_id:?}"))?
                .cmd_sender()
                .ok_or(anyhow!("Sidevm stopped: {contract_id:?}"))?;
            tokio::runtime::Runtime::new()?.block_on(async move {
                tokio::time::timeout(timeout, async {
                    let (reply_tx, rx) = tokio::sync::oneshot::channel();
                    tx.send(sidevm::service::Command::PushQuery {
                        origin: Some(origin),
                        payload,
                        reply_tx,
                    })
                    .await
                    .or(Err(anyhow!("Sidevm send query failed")))?;
                    rx.await.or(Err(anyhow!("Broken pipe")))
                })
                .await
                .or(Err(anyhow!("Sidevm query timeout")))?
            })
        }

        fn sidevm_event_tx(&self) -> OutgoingRequestChannel {
            self.sidevm_event_tx.clone()
        }

        fn worker_sgx_quote(&self) -> Option<SgxQuote> {
            use AttestationProvider::*;
            let Some(Ias | Dcap) = self.attestation_provider else {
                return None;
            };
            sgx_attestation::gramine::create_quote(&self.worker_pubkey())
        }
    }

    pub fn get() -> ExecContext {
        exec_context::with(|ctx| ctx.exec_context()).unwrap_or_default()
    }

    pub fn using<T>(ctx: &mut impl GetContext, f: impl FnOnce() -> T) -> T {
        exec_context::using(ctx, f)
    }

    pub fn with<T>(f: impl FnOnce(&dyn GetContext) -> T) -> T {
        exec_context::with(|ctx| f(ctx)).expect("exec_context not set")
    }

    pub fn worker_pubkey() -> [u8; 32] {
        exec_context::with(|ctx| ctx.worker_pubkey()).unwrap_or_default()
    }

    pub fn sign_with_worker_identity_key(message: &[u8], sig_type: SignedContentType) -> [u8; 64] {
        let wrapped = wrap_content_to_sign(message, sig_type);
        exec_context::with(|ctx| ctx.worker_identity_key().sign(&wrapped))
            .map(|sig| sig.0)
            .unwrap_or([0; 64])
    }

    pub fn call_elapsed() -> Duration {
        exec_context::with(|ctx| ctx.call_elapsed()).unwrap_or_else(|| Duration::from_secs(0))
    }

    pub fn time_remaining_ms() -> u64 {
        time_remaining().as_millis() as u64
    }

    pub fn time_remaining() -> Duration {
        const MAX_QUERY_TIME: Duration = Duration::from_secs(10);
        MAX_QUERY_TIME.saturating_sub(call_elapsed())
    }

    pub fn sidevm_query(origin: [u8; 32], vmid: [u8; 32], payload: Vec<u8>) -> Result<Vec<u8>> {
        let timeout = Duration::from_millis(time_remaining_ms());
        exec_context::with(|ctx| ctx.sidevm_query(origin, vmid, payload, timeout))
            .ok_or(anyhow!("sidevm_query called outside of contract execution"))?
    }

    pub fn sidevm_event_tx() -> OutgoingRequestChannel {
        exec_context::with(|ctx| ctx.sidevm_event_tx())
            .expect("sidevm_event_tx called outside of contract execution")
    }

    pub use entry::{get_entry_contract, get_origin, using_entry};
    mod entry {
        use super::*;

        struct EntryInfo {
            contract: AccountId,
            origin: AccountId,
        }

        environmental::environmental!(entry: EntryInfo);

        pub fn get_entry_contract() -> Option<AccountId> {
            entry::with(|entry| entry.contract.clone())
        }

        pub fn get_origin() -> Option<AccountId> {
            entry::with(|entry| entry.origin.clone())
        }

        pub fn using_entry<T>(contract: AccountId, origin: AccountId, f: impl FnOnce() -> T) -> T {
            let mut entry = EntryInfo { contract, origin };
            entry::using(&mut entry, f)
        }
    }

    pub(super) use call_nonce::{get_call_nonce, using_call_nonce};
    mod call_nonce {
        environmental::environmental!(contract_call_nonce: Vec<u8>);

        pub fn get_call_nonce() -> Option<Vec<u8>> {
            contract_call_nonce::with(|a| a.clone())
        }

        pub fn using_call_nonce<T>(mut call_nonce: Vec<u8>, f: impl FnOnce() -> T) -> T {
            contract_call_nonce::using(&mut call_nonce, f)
        }
    }
}

impl OCalls for RuntimeHandle<'_> {
    fn storage_root(&self) -> Option<Hash> {
        self.cluster.storage.root()
    }

    fn storage_get(&self, key: Vec<u8>) -> Option<Vec<u8>> {
        self.cluster.storage.get(&key).map(|(_rc, val)| val.clone())
    }

    fn storage_commit(&mut self, _root: Hash, _changes: StorageChanges) {
        panic!("storage_commit called on readonly cluster");
    }

    fn log_to_server(&self, contract: AccountId, level: u8, message: String) {
        let Some(log_handler) = self.logger.as_ref() else {
            return;
        };
        let Some(entry) = context::get_entry_contract() else {
            error!("Pink log called outside of contract execution");
            return;
        };
        let context = self.exec_context();
        let msg = SidevmCommand::PushSystemMessage(SystemMessage::PinkLog {
            block_number: context.block_number,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as _,
            exec_mode: context.mode.display().into(),
            contract: contract.into(),
            entry: entry.into(),
            level,
            message,
        });
        if log_handler.try_send(msg).is_err() {
            error!("Pink send log to server failed");
        }
    }

    fn emit_side_effects(&mut self, _effects: ExecSideEffects) {}

    fn exec_context(&self) -> ExecContext {
        context::get()
    }

    fn worker_pubkey(&self) -> [u8; 32] {
        context::worker_pubkey()
    }

    fn cache_get(&self, contract: Vec<u8>, key: Vec<u8>) -> Option<Vec<u8>> {
        if !context::get().mode.is_query() {
            return None;
        }
        local_cache::get(&contract, &key)
    }

    fn cache_set(
        &self,
        contract: Vec<u8>,
        key: Vec<u8>,
        value: Vec<u8>,
    ) -> Result<(), StorageQuotaExceeded> {
        if context::get().mode.is_estimating() {
            return Ok(());
        }
        local_cache::set(&contract, &key, &value)
    }

    fn cache_set_expiration(&self, contract: Vec<u8>, key: Vec<u8>, expiration: u64) {
        if context::get().mode.is_estimating() {
            return;
        }
        local_cache::set_expiration(&contract, &key, expiration)
    }

    fn cache_remove(&self, contract: Vec<u8>, key: Vec<u8>) -> Option<Vec<u8>> {
        if context::get().mode.is_estimating() {
            return None;
        }
        local_cache::remove(&contract, &key)
    }

    fn latest_system_code(&self) -> Vec<u8> {
        context::with(|ctx| ctx.chain_storage().pink_system_code().1)
    }

    fn http_request(
        &self,
        contract: AccountId,
        request: HttpRequest,
    ) -> Result<HttpResponse, HttpRequestError> {
        if request.url.starts_with("sidevm://") {
            let dest = hex::decode(request.url.trim_start_matches("sidevm://"))
                .or(Err(HttpRequestError::InvalidUrl))?
                .try_into()
                .or(Err(HttpRequestError::InvalidUrl))?;
            let origin = contract.into();
            let result = context::sidevm_query(origin, dest, request.body);
            return match result {
                Ok(r) => Ok(HttpResponse {
                    status_code: 200,
                    reason_phrase: "OK".into(),
                    headers: vec![],
                    body: r,
                }),
                Err(err) => {
                    error!("sidevm query failed: {:?}", err);
                    Ok(HttpResponse {
                        status_code: 500,
                        reason_phrase: "Internal Server Error".into(),
                        headers: vec![],
                        body: err.to_string().into_bytes(),
                    })
                }
            };
        }
        let result = pink_extension_runtime::http_request(request, context::time_remaining_ms());
        match &result {
            Ok(response) => {
                http_counters::add(contract, response.status_code);
            }
            Err(_) => {
                http_counters::add(contract, 0);
            }
        }
        result
    }

    fn batch_http_request(
        &self,
        contract: AccountId,
        requests: Vec<HttpRequest>,
        timeout_ms: u64,
    ) -> BatchHttpResult {
        let results = pink_extension_runtime::batch_http_request(
            requests,
            context::time_remaining_ms().min(timeout_ms),
        )?;
        for result in &results {
            match result {
                Ok(r) => {
                    http_counters::add(contract.clone(), r.status_code);
                }
                Err(_) => {
                    http_counters::add(contract.clone(), 0);
                }
            }
        }
        Ok(results)
    }

    fn emit_system_event_block(&self, _number: u64, _encoded_block: Vec<u8>) {
        error!("emit_system_event_block called on readonly calls");
    }

    fn contract_call_nonce(&self) -> Option<Vec<u8>> {
        context::get_call_nonce()
    }

    fn entry_contract(&self) -> Option<AccountId> {
        context::get_entry_contract()
    }

    fn js_eval(&self, caller: AccountId, codes: Vec<JsCode>, js_args: Vec<String>) -> JsValue {
        info!("evaluating js from {caller:?}");
        let Some(js_runtime) = self.cluster.config.js_runtime else {
            return JsValue::Exception("No js runtime".into());
        };
        let timeout = Duration::from_millis(context::time_remaining_ms());
        let mut args = vec!["phatjs".into()];
        for code in codes {
            match code {
                JsCode::Source(src) => {
                    args.push("-c".into());
                    args.push(src);
                }
                JsCode::Bytecode(code) => {
                    args.push("-b".into());
                    args.push(hex::encode(code));
                }
            }
        }
        args.push("--".into());
        args.extend(js_args);

        let result = load_module(&js_runtime, || {
            self.cluster
                .get_resource(ResourceType::SidevmCode, &js_runtime)
        });
        let module = match result {
            Err(err) => {
                let err = format!("Failed to load Javascript runtime module: {:?}", err);
                error!("{}", err);
                return JsValue::Exception(err);
            }
            Ok(module) => module,
        };
        let result = block_on_run_module(
            caller.into(),
            &module,
            args,
            timeout,
            context::sidevm_event_tx(),
            |vmid, level, message| self.log_to_server(vmid.into(), level, message),
        );
        match result {
            Ok(value) => value,
            Err(err) => {
                let err = format!("Failed to run Javascript: {:?}", err);
                error!("{}", err);
                JsValue::Exception(err)
            }
        }
    }

    fn origin(&self) -> Option<AccountId> {
        context::get_origin()
    }

    fn worker_sgx_quote(&self) -> Option<SgxQuote> {
        context::with(|ctx| ctx.worker_sgx_quote())
    }
}

pub fn load_module(code_hash: &Hash, init: impl FnOnce() -> Option<Vec<u8>>) -> Result<WasmModule> {
    struct Cache {
        engine: WasmEngine,
        cached_module: Option<(Hash, WasmModule)>,
    }
    static WASM_CACHE: Lazy<Mutex<Cache>> = Lazy::new(|| {
        Mutex::new(Cache {
            engine: WasmEngine::new(),
            cached_module: None,
        })
    });
    let mut cache = WASM_CACHE.lock().unwrap();
    if let Some((hash, module)) = &cache.cached_module {
        if hash == code_hash {
            return Ok(module.clone());
        }
    }
    let code = init().ok_or_else(|| anyhow::anyhow!("Js runtime code not found"))?;
    let module = cache
        .engine
        .compile(&code)
        .context("Failed to compile js runtime")?;
    cache.cached_module = Some((*code_hash, module.clone()));
    Ok(module)
}

impl OCalls for RuntimeHandleMut<'_> {
    fn storage_root(&self) -> Option<Hash> {
        self.readonly().storage_root()
    }

    fn storage_get(&self, key: Vec<u8>) -> Option<Vec<u8>> {
        self.readonly().storage_get(key)
    }

    fn storage_commit(&mut self, root: Hash, changes: StorageChanges) {
        self.cluster.storage.commit(root, changes);
    }

    fn log_to_server(&self, contract: AccountId, level: u8, message: String) {
        self.readonly().log_to_server(contract, level, message)
    }

    fn emit_side_effects(&mut self, effects: ExecSideEffects) {
        self.effects = Some(effects);
    }

    fn exec_context(&self) -> ExecContext {
        self.readonly().exec_context()
    }

    fn worker_pubkey(&self) -> [u8; 32] {
        self.readonly().worker_pubkey()
    }

    fn cache_get(&self, contract: Vec<u8>, key: Vec<u8>) -> Option<Vec<u8>> {
        self.readonly().cache_get(contract, key)
    }

    fn cache_set(
        &self,
        contract: Vec<u8>,
        key: Vec<u8>,
        value: Vec<u8>,
    ) -> Result<(), StorageQuotaExceeded> {
        self.readonly().cache_set(contract, key, value)
    }

    fn cache_set_expiration(&self, contract: Vec<u8>, key: Vec<u8>, expiration: u64) {
        self.readonly()
            .cache_set_expiration(contract, key, expiration)
    }

    fn cache_remove(&self, contract: Vec<u8>, key: Vec<u8>) -> Option<Vec<u8>> {
        self.readonly().cache_remove(contract, key)
    }

    fn latest_system_code(&self) -> Vec<u8> {
        self.readonly().latest_system_code()
    }

    fn http_request(
        &self,
        contract: AccountId,
        request: HttpRequest,
    ) -> Result<HttpResponse, HttpRequestError> {
        self.readonly().http_request(contract, request)
    }

    fn batch_http_request(
        &self,
        contract: AccountId,
        requests: Vec<HttpRequest>,
        timeout_ms: u64,
    ) -> BatchHttpResult {
        self.readonly()
            .batch_http_request(contract, requests, timeout_ms)
    }

    fn emit_system_event_block(&self, number: u64, encoded_block: Vec<u8>) {
        if !tracing::enabled!(target: "phactory::event_chain", tracing::Level::INFO) {
            return;
        }
        if !context::get().mode.is_transaction() {
            return;
        }
        let signature = hex_fmt::HexFmt(context::sign_with_worker_identity_key(
            &encoded_block,
            SignedContentType::EventChainBlock,
        ));
        let pubkey = hex_fmt::HexFmt(context::worker_pubkey());
        let payload = hex_fmt::HexFmt(encoded_block);
        info!(target: "phactory::event_chain", number, %payload, %signature, %pubkey);
    }

    fn contract_call_nonce(&self) -> Option<Vec<u8>> {
        self.readonly().contract_call_nonce()
    }

    fn entry_contract(&self) -> Option<AccountId> {
        self.readonly().entry_contract()
    }

    fn js_eval(&self, caller: AccountId, codes: Vec<JsCode>, args: Vec<String>) -> JsValue {
        self.readonly().js_eval(caller, codes, args)
    }

    fn origin(&self) -> Option<AccountId> {
        self.readonly().origin()
    }

    fn worker_sgx_quote(&self) -> Option<SgxQuote> {
        self.readonly().worker_sgx_quote()
    }
}

impl v1::CrossCall for RuntimeHandle<'_> {
    fn cross_call(&self, call_id: u32, data: &[u8]) -> Vec<u8> {
        self.execute(move |version| get_runtime(version).ecall(call_id, data))
    }
}

impl v1::CrossCall for RuntimeHandleMut<'_> {
    fn cross_call(&self, call_id: u32, data: &[u8]) -> Vec<u8> {
        self.readonly()
            .execute(move |version| get_runtime(version).ecall(call_id, data))
    }
}

impl v1::CrossCallMut for RuntimeHandleMut<'_> {
    fn cross_call_mut(&mut self, call_id: u32, data: &[u8]) -> Vec<u8> {
        self.execute_mut(move |version| get_runtime(version).ecall(call_id, data))
    }
}

impl v1::ECall for RuntimeHandle<'_> {}
impl v1::ECall for RuntimeHandleMut<'_> {}

impl Cluster {
    pub fn new(
        id: &ContractClusterId,
        cluster_key: &sr25519::Pair,
        runtime_version: (u32, u32),
    ) -> Self {
        let secret_key = cluster_key.dump_secret_key();
        let secret_salt = blake2_256(&secret_key);
        let mut cluster = Cluster {
            id: *id,
            storage: Default::default(),
            config: ClusterConfig {
                runtime_version,
                secret_salt,
                ..Default::default()
            },
        };
        let mut runtime = cluster.default_runtime_mut();
        runtime.set_key(secret_key);
        cluster
    }

    pub fn default_runtime(&self) -> RuntimeHandle {
        self.runtime(None)
    }

    pub fn default_runtime_mut(&mut self) -> RuntimeHandleMut {
        self.runtime_mut(None)
    }

    pub fn runtime(&self, logger: Option<CommandSender>) -> RuntimeHandle {
        RuntimeHandle {
            cluster: self,
            logger,
        }
    }

    pub fn runtime_mut(&mut self, logger: Option<CommandSender>) -> RuntimeHandleMut {
        RuntimeHandleMut {
            cluster: self,
            logger,
            effects: None,
        }
    }

    pub fn key(&self) -> sr25519::Pair {
        let raw_key = self
            .default_runtime()
            .get_key()
            .expect("cluster key not set");
        sr25519::Pair::restore_from_secret_key(&raw_key)
    }

    pub fn system_contract(&self) -> Option<AccountId> {
        self.default_runtime().system_contract()
    }

    pub fn code_hash(&self, address: &AccountId) -> Option<Hash> {
        self.default_runtime().code_hash(address.clone())
    }

    pub fn upload_resource(
        &mut self,
        origin: &AccountId,
        resource_type: ResourceType,
        resource_data: Vec<u8>,
    ) -> Result<Hash, String> {
        match resource_type {
            ResourceType::InkCode => {
                self.default_runtime_mut()
                    .upload_code(origin.clone(), resource_data, true)
            }
            ResourceType::SidevmCode => self
                .default_runtime_mut()
                .upload_sidevm_code(origin.clone(), resource_data),
            ResourceType::IndeterministicInkCode => {
                self.default_runtime_mut()
                    .upload_code(origin.clone(), resource_data, false)
            }
        }
    }

    pub fn get_resource(&self, resource_type: ResourceType, hash: &Hash) -> Option<Vec<u8>> {
        match resource_type {
            ResourceType::InkCode => None,
            ResourceType::SidevmCode => self.default_runtime().get_sidevm_code(*hash),
            ResourceType::IndeterministicInkCode => None,
        }
    }

    pub fn deposit(&mut self, who: &::pink::types::AccountId, amount: Balance) {
        self.default_runtime_mut().deposit(who.clone(), amount)
    }

    pub(crate) async fn handle_query(
        mut self,
        contract_id: &AccountId,
        origin: Option<&AccountId>,
        req: Query,
        context: QueryContext,
        contracts: ContractsKeeper,
    ) -> Result<(Response, Option<ExecSideEffects>), QueryError> {
        match req {
            Query::InkMessage {
                payload: input_data,
                deposit,
                transfer,
                estimating,
            } => {
                let origin = origin.cloned().ok_or(QueryError::BadOrigin)?;
                let _guard = context
                    .query_scheduler
                    .acquire(contract_id.clone(), context.weight)
                    .await
                    .or(Err(QueryError::ServiceUnavailable))?;

                if let Some(logger) = &context.log_handler {
                    let fp = twox_64(&(&origin, &self.config.secret_salt).encode());
                    if let Err(_err) = logger.try_send(SidevmCommand::PushSystemMessage(
                        SystemMessage::Metric(Metric::PinkQueryIn(fp)),
                    )) {
                        error!("Failed to send metric to log_server");
                    }
                }

                let mode = if estimating {
                    ExecutionMode::Estimating
                } else {
                    ExecutionMode::Query
                };
                let mut ctx = context::ContractExecContext::new(
                    mode,
                    context.now_ms,
                    context.block_number,
                    context.worker_identity_key.clone(),
                    context.chain_storage,
                    context.req_id,
                    contracts,
                    context.sidevm_event_tx.clone(),
                    context.attestation_provider,
                );
                let log_handler = context.log_handler.clone();
                let contract_id = contract_id.clone();
                let span = tracing::trace_span!("blocking");
                tokio::task::spawn_blocking(move || {
                    let _guard = span.enter();
                    context::using(&mut ctx, move || {
                        let mut runtime = self.runtime_mut(log_handler);
                        let args = TransactionArguments {
                            origin,
                            transfer,
                            gas_limit: WEIGHT_REF_TIME_PER_SECOND * 10,
                            gas_free: true,
                            storage_deposit_limit: None,
                            deposit,
                        };
                        let ink_result = runtime.call(contract_id, input_data, mode, args);
                        let effects = if mode.is_estimating() {
                            None
                        } else {
                            runtime.effects.take().map(|e| e.into_query_only_effects())
                        };
                        Ok((Response::Payload(ink_result), effects))
                    })
                })
                .await
                .map_err(|_| QueryError::RuntimeError("Failed to spawn blocking task".into()))?
            }
            Query::SidevmQuery(payload) => {
                let handle = context
                    .sidevm_handle
                    .as_ref()
                    .ok_or(QueryError::SidevmNotFound)?;
                let cmd_sender = match handle {
                    contracts::SidevmHandle::Stopped(_) => return Err(QueryError::SidevmNotFound),
                    contracts::SidevmHandle::Running { cmd_sender, .. } => cmd_sender,
                };
                let origin = origin.cloned().map(Into::into);

                let (reply_tx, rx) = tokio::sync::oneshot::channel();

                const SIDEVM_QUERY_TIMEOUT: Duration = Duration::from_secs(60 * 5);

                tokio::time::timeout(SIDEVM_QUERY_TIMEOUT, async move {
                    cmd_sender
                        .send(SidevmCommand::PushQuery {
                            origin,
                            payload,
                            reply_tx,
                        })
                        .await
                        .or(Err(QueryError::ServiceUnavailable))?;
                    rx.await
                        .or(Err(QueryError::NoResponse))
                        .map(Response::Payload)
                })
                .await
                .or(Err(QueryError::Timeout))?
                .map(|response| (response, None))
            }
            Query::InkInstantiate {
                code_hash,
                salt,
                instantiate_data,
                deposit,
                transfer,
            } => {
                let _guard = context
                    .query_scheduler
                    .acquire(contract_id.clone(), context.weight)
                    .await
                    .or(Err(QueryError::ServiceUnavailable))?;

                let origin = origin.cloned().ok_or(QueryError::BadOrigin)?;
                let mut ctx = context::ContractExecContext::new(
                    ExecutionMode::Estimating,
                    context.now_ms,
                    context.block_number,
                    context.worker_identity_key.clone(),
                    context.chain_storage,
                    context.req_id,
                    contracts,
                    context.sidevm_event_tx.clone(),
                    context.attestation_provider,
                );
                let log_handler = context.log_handler.clone();
                context::using(&mut ctx, move || {
                    let mut runtime = self.runtime_mut(log_handler);
                    let args = TransactionArguments {
                        origin,
                        transfer,
                        gas_limit: WEIGHT_REF_TIME_PER_SECOND * 10,
                        gas_free: true,
                        storage_deposit_limit: None,
                        deposit,
                    };
                    let ink_result = runtime.instantiate(
                        code_hash,
                        instantiate_data,
                        salt,
                        ExecutionMode::Estimating,
                        args,
                    );
                    Ok((Response::Payload(ink_result), None))
                })
            }
        }
    }

    pub(crate) fn handle_command(
        &mut self,
        contract_id: &AccountId,
        origin: MessageOrigin,
        cmd: InkCommand,
        context: &mut TransactionContext,
    ) -> TransactionResult {
        match cmd {
            InkCommand::InkMessage {
                nonce,
                message,
                transfer,
                gas_limit,
                storage_deposit_limit,
            } => {
                let mut gas_free = false;
                let origin: runtime::AccountId = match origin {
                    MessageOrigin::AccountId(origin) => origin.0.into(),
                    MessageOrigin::Pallet(_) => {
                        // The caller will be set to the system contract if it's from a pallet call
                        // and without charging for gas
                        gas_free = true;
                        self.system_contract()
                            .expect("BUG: system contract missing")
                    }
                    _ => return Err(TransactionError::BadOrigin),
                };

                let args = TransactionArguments {
                    origin: origin.clone(),
                    transfer,
                    gas_limit,
                    gas_free,
                    storage_deposit_limit,
                    deposit: 0,
                };

                let selector = head4(&message);
                let mut runtime = self.runtime_mut(context.log_handler.clone());
                let output = context::using_call_nonce(nonce.clone().into(), || {
                    runtime.call(
                        contract_id.clone(),
                        message,
                        ExecutionMode::Transaction,
                        args,
                    )
                });

                if let Some(log_handler) = &context.log_handler {
                    let mut output = output;
                    output.extend_from_slice(&selector);
                    let msg = SidevmCommand::PushSystemMessage(SystemMessage::PinkMessageOutput {
                        origin: origin.into(),
                        contract: *contract_id.as_ref(),
                        block_number: context.block.block_number,
                        nonce: nonce.into_inner(),
                        output,
                    });
                    if log_handler.try_send(msg).is_err() {
                        error!("Pink emit message output to log handler failed");
                    }
                }
                Ok(runtime.effects)
            }
        }
    }

    pub(crate) fn instantiate(
        &mut self,
        logger: Option<CommandSender>,
        code_hash: Hash,
        instantiate_data: Vec<u8>,
        salt: Vec<u8>,
        mode: ExecutionMode,
        tx_args: TransactionArguments,
    ) -> (Vec<u8>, Option<ExecSideEffects>) {
        let mut runtime = self.runtime_mut(logger);
        let result = context::using_call_nonce(salt.clone(), || {
            runtime.instantiate(code_hash, instantiate_data, salt, mode, tx_args)
        });
        (result, runtime.effects.take())
    }

    pub(crate) fn snapshot(&self) -> Self {
        self.clone()
    }

    pub(crate) fn upgrade_runtime(&mut self, version: (u32, u32)) {
        let current = self.config.runtime_version;
        info!("Try to upgrade runtime to {version:?}, current version is {current:?}");
        if version <= current {
            info!("Runtime version is already {current:?}");
            return;
        }
        if current == (1, 0) {
            // The 1.0 runtime didn't call on_genesis on cluster setup, so we need to call it
            // manually to make sure the storage_versions are correct before migration.
            self.default_runtime_mut().on_genesis();
        }
        self.config.runtime_version = version;
        self.default_runtime_mut().on_runtime_upgrade();
        info!("Runtime upgraded to {version:?}");
    }

    pub(crate) fn on_idle(&mut self, block_number: BlockNumber) {
        if pink::types::ECallsAvailable::on_idle(self.config.runtime_version) {
            self.default_runtime_mut().on_idle(block_number);
        }
    }
}

fn head4(bytes: &[u8]) -> [u8; 4] {
    let mut buf = [0u8; 4];
    if buf.len() >= 4 {
        buf.copy_from_slice(&bytes[..4]);
    }
    buf
}

pub trait ClusterContainer {
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn len(&self) -> usize;
    fn get_cluster(&self, cluster_id: &ContractClusterId) -> Option<&Cluster>;
    fn get_cluster_mut(&mut self, cluster_id: &ContractClusterId) -> Option<&mut Cluster>;
    fn remove_cluster(&mut self, cluster_id: &ContractClusterId) -> Option<Cluster>;
}

impl ClusterContainer for Option<Cluster> {
    fn len(&self) -> usize {
        if self.is_some() {
            1
        } else {
            0
        }
    }
    fn get_cluster(&self, cluster_id: &ContractClusterId) -> Option<&Cluster> {
        self.as_ref().filter(|c| c.id == *cluster_id)
    }
    fn get_cluster_mut(&mut self, cluster_id: &ContractClusterId) -> Option<&mut Cluster> {
        self.as_mut().filter(|c| c.id == *cluster_id)
    }
    fn remove_cluster(&mut self, cluster_id: &ContractClusterId) -> Option<Cluster> {
        _ = self.get_cluster(cluster_id)?;
        self.take()
    }
}

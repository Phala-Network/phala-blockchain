use frame_support::{traits::Currency, weights::constants::WEIGHT_REF_TIME_PER_SECOND};
use log::info;
use pallet_contracts::{AddressGenerator, Determinism};
use phala_crypto::sr25519::Sr25519SecretKey;
use pink_capi::{
    types::{AccountId, Balance, ExecutionMode, Hash, Weight},
    v1::{
        ecall::{self, ClusterSetupConfig, TransactionArguments},
        ocall::OCalls,
        Executing,
    },
};
use scale::Encode;

use crate::{
    contract::check_instantiate_result,
    runtime::{
        on_genesis, on_idle, on_runtime_upgrade, Balances as PalletBalances,
        Contracts as PalletContracts, Pink as PalletPink,
    },
    types::BlockNumber,
};

use super::OCallImpl;

pub struct ECallImpl;

pub(crate) fn storage() -> crate::storage::ExternalStorage {
    crate::storage::ExternalStorage::instantiate()
}

/// Set spans for blocknumber and request id
macro_rules! instrument_context {
    ($context: expr) => {
        let span = $context.req_id.map(|id| tracing::info_span!("prpc", id));
        let _enter = span.as_ref().map(|span| span.enter());
        let span = tracing::info_span!("pink", blk = $context.block_number);
        let _enter = span.enter();
    };
}

impl Executing for crate::storage::ExternalStorage {
    fn execute<T>(&self, f: impl FnOnce() -> T) -> T {
        let context = OCallImpl.exec_context();
        instrument_context!(&context);
        let (rv, _effects, _) = self.execute_with(&context, f);
        rv
    }

    fn execute_mut<T>(&mut self, f: impl FnOnce() -> T) -> T {
        let context = OCallImpl.exec_context();
        instrument_context!(&context);
        let (rv, effects) = self.execute_mut(&context, f);
        OCallImpl.emit_side_effects(effects);
        rv
    }
}

impl ecall::ECalls for ECallImpl {
    fn cluster_id(&self) -> Hash {
        PalletPink::cluster_id()
    }
    fn setup(&mut self, config: ClusterSetupConfig) -> Result<(), String> {
        on_genesis();
        let ClusterSetupConfig {
            cluster_id,
            owner,
            deposit,
            gas_price,
            deposit_per_item,
            deposit_per_byte,
            treasury_account,
            system_code,
        } = config;
        PalletPink::set_cluster_id(cluster_id);
        PalletPink::set_gas_price(gas_price);
        PalletPink::set_deposit_per_item(deposit_per_item);
        PalletPink::set_deposit_per_byte(deposit_per_byte);
        PalletPink::set_treasury_account(&treasury_account);

        self.deposit(owner.clone(), deposit);
        let code_hash = self
            .upload_code(owner.clone(), system_code, true)
            .map_err(|err| format!("FailedToUploadSystemCode: {err:?}"))?;
        info!("Worker: pink system code hash {:?}", code_hash);
        let selector = vec![0xed, 0x4b, 0x9d, 0x1b]; // The default() constructor
        let args = TransactionArguments {
            origin: owner,
            transfer: 0,
            gas_limit: Weight::MAX,
            gas_free: true,
            storage_deposit_limit: None,
            deposit: 0,
        };
        let result = crate::contract::instantiate(
            code_hash,
            selector,
            vec![],
            ExecutionMode::Transaction,
            args,
        );
        let address = match check_instantiate_result(&result) {
            Err(err) => {
                info!("Worker: failed to deploy system contract: {:?}", err);
                return Err("FailedToDeploySystemContract".into());
            }
            Ok(address) => address,
        };
        PalletPink::set_system_contract(&address);
        info!(
            "Cluster deployed, id={:?}, system={:?}",
            cluster_id, address
        );
        Ok(())
    }

    fn deposit(&mut self, who: AccountId, value: Balance) {
        let _ = PalletBalances::deposit_creating(&who, value);
    }

    fn set_key(&mut self, key: Sr25519SecretKey) {
        PalletPink::set_key(key);
    }

    fn get_key(&self) -> Option<Sr25519SecretKey> {
        PalletPink::key()
    }

    fn upload_code(
        &mut self,
        account: AccountId,
        code: Vec<u8>,
        deterministic: bool,
    ) -> Result<Hash, String> {
        /*
        According to the cost estimation in ink tests: https://github.com/paritytech/substrate/pull/12993/files#diff-70e9723e9db62816e35f6f885b6770a8449c75a6c2733e9fa7a245fe52c4656cR423
        If we set max code len to 2MB, the max memory cost for a single call stack would be calculated as:
        cost = (MaxCodeLen * 4 + MAX_STACK_SIZE + max_heap_size) * max_call_depth
             = (2MB * 4 + 1MB + 4MB) * 6
             = 78MB
        If we allow 8 concurrent calls, the total memory cost would be 78MB * 8 = 624MB.
        */
        let info =
            phala_wasm_checker::wasm_info(&code).map_err(|err| format!("Invalid wasm: {err:?}"))?;
        let max_wasmi_cost = crate::runtime::MaxCodeLen::get() as usize * 4;
        if info.estimate_wasmi_memory_cost() > max_wasmi_cost {
            return Err("DecompressedCodeTooLarge".into());
        }
        crate::runtime::Contracts::bare_upload_code(
            account,
            code,
            None,
            if deterministic {
                Determinism::Enforced
            } else {
                Determinism::Relaxed
            },
        )
        .map(|v| v.code_hash)
        .map_err(|err| format!("{err:?}"))
    }

    fn upload_sidevm_code(&mut self, account: AccountId, code: Vec<u8>) -> Result<Hash, String> {
        PalletPink::put_sidevm_code(account, code).map_err(|err| format!("{err:?}"))
    }

    fn get_sidevm_code(&self, hash: Hash) -> Option<Vec<u8>> {
        PalletPink::sidevm_codes(hash).map(|v| v.code)
    }

    fn system_contract(&self) -> Option<AccountId> {
        PalletPink::system_contract()
    }

    fn free_balance(&self, account: AccountId) -> Balance {
        PalletBalances::free_balance(account)
    }

    fn total_balance(&self, account: AccountId) -> Balance {
        PalletBalances::total_balance(&account)
    }

    fn code_hash(&self, account: AccountId) -> Option<Hash> {
        PalletContracts::code_hash(&account)
    }

    fn contract_instantiate(
        &mut self,
        code_hash: Hash,
        input_data: Vec<u8>,
        salt: Vec<u8>,
        mode: ExecutionMode,
        tx_args: TransactionArguments,
    ) -> Vec<u8> {
        let tx_args = sanitize_args(tx_args, mode);
        handle_deposit(&tx_args);
        let address = PalletPink::contract_address(&tx_args.origin, &code_hash, &input_data, &salt);
        let result = crate::contract::instantiate(code_hash, input_data, salt, mode, tx_args);
        if !result.debug_message.is_empty() {
            let message = String::from_utf8_lossy(&result.debug_message).into_owned();
            OCallImpl.log_to_server(
                address.clone(),
                log::Level::Debug as usize as _,
                message.clone(),
            );
            log::debug!("[{address:?}][{mode:?}] debug_message: {message:?}");
        }
        match &result.result {
            Err(err) => {
                log::error!("[{address:?}][{mode:?}] instantiate error: {err:?}");
                OCallImpl.log_to_server(
                    address,
                    log::Level::Error as usize as _,
                    format!("instantiate failed: {err:?}"),
                );
            }
            Ok(ret) if ret.result.did_revert() => {
                log::error!("[{address:?}][{mode:?}] instantiate reverted");
                OCallImpl.log_to_server(
                    address,
                    log::Level::Error as usize as _,
                    "instantiate reverted".into(),
                );
            }
            Ok(_) => {
                log::info!("[{address:?}][{mode:?}] instantiated");
                OCallImpl.log_to_server(
                    address,
                    log::Level::Info as usize as _,
                    "instantiated".to_owned(),
                );
            }
        }
        result.encode()
    }

    fn contract_call(
        &mut self,
        address: AccountId,
        input_data: Vec<u8>,
        mode: ExecutionMode,
        tx_args: TransactionArguments,
    ) -> Vec<u8> {
        let tx_args = sanitize_args(tx_args, mode);
        handle_deposit(&tx_args);
        let result = crate::contract::bare_call(address.clone(), input_data, mode, tx_args);
        if !result.debug_message.is_empty() {
            let message = String::from_utf8_lossy(&result.debug_message).into_owned();
            OCallImpl.log_to_server(
                address.clone(),
                log::Level::Debug as usize as _,
                message.clone(),
            );
            log::debug!("[{address:?}][{mode:?}] debug_message: {:?}", message);
        }
        match &result.result {
            Err(err) => {
                log::error!("[{address:?}][{mode:?}] command exec error: {:?}", err);
                OCallImpl.log_to_server(
                    address,
                    log::Level::Error as usize as _,
                    format!("contract call failed: {err:?}"),
                );
            }
            Ok(ret) if ret.did_revert() => {
                log::error!("[{address:?}][{mode:?}] contract reverted: {:?}", ret);
                OCallImpl.log_to_server(
                    address,
                    log::Level::Error as usize as _,
                    "contract call reverted".into(),
                );
            }
            Ok(_) => {}
        }
        result.encode()
    }

    fn git_revision(&self) -> String {
        phala_git_revision::git_revision().to_string()
    }

    fn on_genesis(&mut self) {
        on_genesis();
    }

    fn on_runtime_upgrade(&mut self) {
        on_runtime_upgrade();
    }

    fn on_idle(&mut self, block_number: BlockNumber) {
        on_idle(block_number);
    }
}

/// Clip gas limit to 0.5 second for tx, 10 seconds for query
fn sanitize_args(mut args: TransactionArguments, mode: ExecutionMode) -> TransactionArguments {
    let gas_limit = match mode {
        ExecutionMode::Transaction | ExecutionMode::Estimating => WEIGHT_REF_TIME_PER_SECOND / 2,
        ExecutionMode::Query => WEIGHT_REF_TIME_PER_SECOND * 10,
    };
    args.gas_limit = args.gas_limit.min(gas_limit);
    args
}

fn handle_deposit(args: &TransactionArguments) {
    if args.deposit > 0 {
        let _ = PalletBalances::deposit_creating(&args.origin, args.deposit);
    }
}

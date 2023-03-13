use frame_support::traits::Currency;
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
    runtime::{Balances as PalletBalances, Contracts as PalletContracts, Pink as PalletPink},
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
            .or(Err("FailedToUploadResourceToCluster"))?;
        info!("Worker: pink system code hash {:?}", code_hash);
        let selector = vec![0xed, 0x4b, 0x9d, 0x1b]; // The default() constructor
        let args = TransactionArguments {
            origin: owner,
            transfer: 0,
            gas_limit: Weight::MAX,
            gas_free: true,
            storage_deposit_limit: None,
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
        crate::runtime::Contracts::bare_upload_code(
            account,
            code,
            None,
            if deterministic {
                Determinism::Deterministic
            } else {
                Determinism::AllowIndeterminism
            },
        )
        .map(|v| v.code_hash)
        .map_err(|err| format!("{err:?}"))
    }

    fn upload_sidevm_code(&mut self, account: AccountId, code: Vec<u8>) -> Result<Hash, String> {
        PalletPink::put_sidevm_code(account, code).map_err(|err| format!("{err:?}"))
    }

    fn get_sidevm_code(&self, hash: Hash) -> Option<Vec<u8>> {
        PalletPink::sidevm_codes(&hash).map(|v| v.code)
    }

    fn system_contract(&self) -> Option<AccountId> {
        PalletPink::system_contract()
    }

    fn free_balance(&self, account: AccountId) -> Balance {
        PalletBalances::free_balance(&account)
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
        let address = PalletPink::generate_address(&tx_args.origin, &code_hash, &input_data, &salt);
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
}

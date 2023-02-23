use frame_support::traits::Currency;
use log::info;
use pallet_contracts::Determinism;
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

impl Executing for crate::storage::ExternalStorage {
    fn execute<T>(&self, f: impl FnOnce() -> T) -> T {
        let todo = "fill query and callbacks";
        let context = OCallImpl.exec_context();
        let (rv, _effects, _) = self.execute_with(&context, None, f);
        rv
    }

    fn execute_mut<T>(&mut self, f: impl FnOnce() -> T) -> T {
        let todo = "fill query and callbacks";
        let context = OCallImpl.exec_context();
        let (rv, effects) = self.execute_mut(&context, None, f);
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
            origin: owner.clone(),
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
    ) -> Result<Hash, Vec<u8>> {
        crate::runtime::Contracts::bare_upload_code(
            account.clone(),
            code,
            None,
            if deterministic {
                Determinism::Deterministic
            } else {
                Determinism::AllowIndeterminism
            },
        )
        .map(|v| v.code_hash)
        .map_err(|err| {
            let todo = "log error";
            err.encode()
        })
    }

    fn upload_sidevm_code(&mut self, account: AccountId, code: Vec<u8>) -> Result<Hash, Vec<u8>> {
        let todo = "log error";
        PalletPink::put_sidevm_code(account, code).map_err(|err| err.encode())
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

    fn code_exists(&self, code_hash: Hash, sidevm: bool) -> bool {
        let todo = "";
        todo!()
    }

    fn contract_instantiate(
        &mut self,
        code_hash: Hash,
        input_data: Vec<u8>,
        salt: Vec<u8>,
        mode: ExecutionMode,
        tx_args: TransactionArguments,
    ) -> Vec<u8> {
        let result = crate::contract::instantiate(code_hash, input_data, salt, mode, tx_args);
        let todo = "log error and clear effects on error";
        // // Send the reault to the log server
        // if let Some(log_handler) = &log_handler {
        //     macro_rules! send_log {
        //         ($level: expr, $msg: expr) => {
        //             let result = log_handler.try_send(
        //                 SidevmCommand::PushSystemMessage(SystemMessage::PinkLog {
        //                     block_number: block.block_number,
        //                     contract: contract_id,
        //                     in_query: false,
        //                     timestamp_ms: block.now_ms,
        //                     level: $level as usize as u8,
        //                     message: $msg,
        //                 }),
        //             );
        //             if result.is_err() {
        //                 error!("Failed to send log to log handler");
        //             }
        //         };
        //     }
        //     match &result {
        //         Ok(_) => {
        //             send_log!(log::Level::Info, "Instantiated".to_owned());
        //         }
        //         Err(err) => {
        //             send_log!(
        //                 log::Level::Error,
        //                 format!("Instantiating failed: {err:?}")
        //             );
        //         }
        //     }
        // }
        result.encode()
    }

    fn contract_call(
        &mut self,
        address: AccountId,
        input_data: Vec<u8>,
        mode: ExecutionMode,
        tx_args: TransactionArguments,
    ) -> Vec<u8> {
        let result = crate::contract::bare_call(address, input_data, mode, tx_args);
        let todo = "log error and clear effects on error";
        //     if !result.debug_message.is_empty() {
        //         ContractEventCallback::new(log_handler.clone(), context.block.block_number)
        //             .emit_log(
        //                 &self.instance.address,
        //                 false,
        //                 log::Level::Debug as usize as _,
        //                 String::from_utf8_lossy(&result.debug_message).into_owned(),
        //             );
        //     }
        // if let Err(err) = result.result {
        //     log::error!("Pink [{:?}] command exec error: {:?}", self.id(), err);
        //     if !result.debug_message.is_empty() {
        //         let message = String::from_utf8_lossy(&result.debug_message);
        //         log::error!("Pink [{:?}] buffer: {:?}", self.id(), message);
        //     }
        //     return Err(TransactionError::Other(format!(
        //         "Call contract method failed: {err:?}"
        //     )));
        // }
        result.encode()
    }

    fn git_revision(&self) -> String {
        phala_git_revision::git_revision().to_string()
    }
}

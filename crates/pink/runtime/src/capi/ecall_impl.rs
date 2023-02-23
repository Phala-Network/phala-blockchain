use frame_support::traits::Currency;
use pallet_contracts::Determinism;
use phala_crypto::sr25519::Sr25519SecretKey;
use pink_capi::{
    types::{AccountId, Balance, BlockNumber, ExecutionMode, ExecSideEffects, Hash, Weight},
    v1::{ecall, ocall::OCalls, CrossCall, Executing},
};
use scale::Encode;

use crate::runtime::{
    Balances as PalletBalances, Contracts as PalletContracts, Pink as PalletPink,
};

use super::OCallImpl;

pub struct ECallImpl;

pub(crate) fn storage() -> crate::storage::ExternalStorage {
    crate::storage::ExternalStorage::instantiate()
}

impl Executing for crate::storage::ExternalStorage {
    fn execute<T>(&self, f: impl FnOnce() -> T) -> T {
        // todo! fill query and callbacks
        let mode = OCallImpl.exec_mode();
        let (rv, _effects, _) = self.execute_with(mode, None, f);
        rv
    }

    fn execute_mut<T>(&mut self, f: impl FnOnce() -> T) -> T {
        // todo! fill query and callbacks
        let mode = OCallImpl.exec_mode();
        let (rv, effects) = self.execute_mut(mode, None, f);
        OCallImpl.emit_side_effects(effects);
        rv
    }
}

impl ecall::ECalls for ECallImpl {
    fn set_cluster_id(&mut self, cluster_id: Hash) {
        PalletPink::set_cluster_id(cluster_id);
    }
    fn cluster_id(&self) -> Hash {
        PalletPink::cluster_id()
    }
    fn setup(
        &mut self,
        gas_price: Balance,
        deposit_per_item: Balance,
        deposit_per_byte: Balance,
        treasury_account: AccountId,
    ) {
        PalletPink::set_gas_price(gas_price);
        PalletPink::set_deposit_per_item(deposit_per_item);
        PalletPink::set_deposit_per_byte(deposit_per_byte);
        PalletPink::set_treasury_account(&treasury_account);
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
            //todo! log error
            err.encode()
        })
    }

    fn upload_sidevm_code(&mut self, account: AccountId, code: Vec<u8>) -> Result<Hash, Vec<u8>> {
        // todo: log error
        PalletPink::put_sidevm_code(account, code).map_err(|err| err.encode())
    }

    fn get_sidevm_code(&self, hash: Hash) -> Option<Vec<u8>> {
        PalletPink::sidevm_codes(&hash).map(|v| v.code)
    }

    fn set_system_contract(&mut self, address: AccountId) {
        PalletPink::set_system_contract(address);
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
        todo!()
    }

    fn contract_instantiate(
        &mut self,
        code_hash: Hash,
        input_data: Vec<u8>,
        salt: Vec<u8>,
        mode: ExecutionMode,
        tx_args: ecall::TransactionArguments,
    ) -> Vec<u8> {
        let result = crate::contract::instantiate(code_hash, input_data, salt, mode, tx_args);
        result.encode()
    }

    fn contract_call(
        &mut self,
        address: AccountId,
        input_data: Vec<u8>,
        mode: ExecutionMode,
        tx_args: ecall::TransactionArguments,
    ) -> Vec<u8> {
        let result = crate::contract::bare_call(address, input_data, mode, tx_args);
        let todo = "log error";
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
        todo!()
    }

    fn block_number(&self) -> BlockNumber {
        todo!()
    }
}

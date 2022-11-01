use frame_support::weights::Weight;
use scale::{Decode, Encode};
use sp_runtime::DispatchError;

use crate::{
    runtime::{
        BoxedEventCallbacks, Contracts, ExecSideEffects, Pink as PalletPink, System, Timestamp,
    },
    storage,
    types::{AccountId, Balance, BlockNumber, Hash},
};

type ContractExecResult = pallet_contracts_primitives::ContractExecResult<Balance>;
type ContractInstantiateResult =
    pallet_contracts_primitives::ContractInstantiateResult<AccountId, Balance>;

type ContractResult<T> =
    pallet_contracts_primitives::ContractResult<Result<T, DispatchError>, Balance>;

pub type Storage = storage::Storage<storage::InMemoryBackend>;

fn _compilation_hint_for_kvdb(db: Storage) {
    // TODO.kevin: Don't forget to clean up the disk space on cluster destroying when we switch to
    // a KVDB backend.
    let _dont_forget_to_clean_up_disk: storage::Storage<storage::InMemoryBackend> = db;
}

impl Default for Storage {
    fn default() -> Self {
        Self::new(storage::new_in_memory_backend())
    }
}

#[derive(Debug, Default, Encode, Decode, Clone)]
struct HookSelectors {
    on_block_end: Option<(u32, Weight)>,
}

#[derive(Debug, Encode, Decode, Clone)]
pub struct Contract {
    pub address: AccountId,
    hooks: HookSelectors,
}

pub struct TransactionArguments<'a> {
    pub origin: AccountId,
    pub now: u64,
    pub block_number: BlockNumber,
    pub storage: &'a mut Storage,
    pub transfer: Balance,
    pub gas_limit: Weight,
    pub gas_free: bool,
    pub storage_deposit_limit: Option<Balance>,
    pub callbacks: Option<BoxedEventCallbacks>,
}

impl Contract {
    /// Create a new contract instance from existing address.
    pub fn from_address(address: AccountId) -> Self {
        Contract {
            address,
            hooks: Default::default(),
        }
    }

    /// Create a new contract instance.
    ///
    /// # Parameters
    ///
    /// * `origin`: The owner of the created contract instance.
    /// * `code_hash`: The hash of contract code which has been uploaded.
    /// * `input_data`: The input data to pass to the contract constructor.
    /// * `salt`: Used for the address derivation.
    pub fn new(
        code_hash: Hash,
        input_data: Vec<u8>,
        salt: Vec<u8>,
        args: TransactionArguments,
    ) -> Result<(Self, ExecSideEffects), DispatchError> {
        let (result, effects) = Self::instantiate(code_hash, input_data, salt, args);
        let result = result.result?;
        Ok((Self::from_address(result.account_id), effects))
    }

    pub fn instantiate(
        code_hash: Hash,
        input_data: Vec<u8>,
        salt: Vec<u8>,
        args: TransactionArguments,
    ) -> (ContractInstantiateResult, ExecSideEffects) {
        let TransactionArguments {
            origin,
            block_number,
            now,
            storage,
            transfer,
            gas_limit,
            storage_deposit_limit,
            callbacks,
            gas_free,
        } = args;
        storage.execute_with(false, callbacks, move || {
            let result = contract_tx(
                origin.clone(),
                block_number,
                now,
                gas_limit,
                gas_free,
                move || {
                    Contracts::bare_instantiate(
                        origin,
                        transfer,
                        gas_limit,
                        storage_deposit_limit,
                        pallet_contracts_primitives::Code::Existing(code_hash),
                        input_data,
                        salt,
                        false,
                    )
                },
            );
            log::info!("Contract instantiation result: {:?}", &result.result);
            result
        })
    }

    pub fn new_with_selector(
        code_hash: Hash,
        selector: [u8; 4],
        args: impl Encode,
        salt: Vec<u8>,
        tx_args: TransactionArguments,
    ) -> Result<(Self, ExecSideEffects), DispatchError> {
        let mut input_data = vec![];
        selector.encode_to(&mut input_data);
        args.encode_to(&mut input_data);
        Self::new(code_hash, input_data, salt, tx_args)
    }

    /// Call a contract method
    ///
    /// # Parameters
    /// * `input_data`: The SCALE encoded arguments including the 4-bytes selector as prefix.
    /// # Return
    /// Returns the SCALE encoded method return value.
    pub fn bare_call(
        &self,
        input_data: Vec<u8>,
        in_query: bool,
        tx_args: TransactionArguments,
    ) -> (ContractExecResult, ExecSideEffects) {
        let TransactionArguments {
            origin,
            now,
            block_number,
            storage,
            transfer,
            gas_limit,
            gas_free,
            callbacks,
            storage_deposit_limit,
        } = tx_args;
        let addr = self.address.clone();
        storage.execute_with(in_query, callbacks, move || {
            contract_tx(
                origin.clone(),
                block_number,
                now,
                gas_limit,
                gas_free,
                move || {
                    Contracts::bare_call(
                        origin,
                        addr,
                        transfer,
                        gas_limit,
                        storage_deposit_limit,
                        input_data,
                        false,
                    )
                },
            )
        })
    }

    /// Call a contract method given it's selector
    pub fn call_with_selector<RV: Decode>(
        &self,
        selector: [u8; 4],
        args: impl Encode,
        in_query: bool,
        tx_args: TransactionArguments,
    ) -> (Result<RV, DispatchError>, ExecSideEffects) {
        let mut input_data = vec![];
        selector.encode_to(&mut input_data);
        args.encode_to(&mut input_data);
        let (result, effects) = self.bare_call(input_data, in_query, tx_args);
        let result = result.result.and_then(|ret| {
            Decode::decode(&mut &ret.data[..])
                .map_err(|_| DispatchError::Other("Decode result failed"))
        });

        (result, effects)
    }

    /// Called by on each block end by the runtime
    pub fn on_block_end(
        &self,
        storage: &mut Storage,
        block_number: BlockNumber,
        now: u64,
        callbacks: Option<BoxedEventCallbacks>,
    ) -> Result<ExecSideEffects, DispatchError> {
        if let Some((selector, gas_limit)) = self.hooks.on_block_end {
            let mut input_data = vec![];
            selector.to_be_bytes().encode_to(&mut input_data);

            let (result, effects) = self.bare_call(
                input_data,
                false,
                TransactionArguments {
                    origin: self.address.clone(),
                    now,
                    block_number,
                    storage,
                    transfer: 0,
                    gas_limit,
                    gas_free: false,
                    storage_deposit_limit: None,
                    callbacks,
                },
            );
            let _ = result.result?;
            Ok(effects)
        } else {
            Ok(Default::default())
        }
    }

    pub fn set_on_block_end_selector(&mut self, selector: u32, gas_limit: u64) {
        self.hooks.on_block_end = Some((selector, Weight::from_ref_time(gas_limit)));
    }

    pub fn code_hash(&self, storage: &Storage) -> Option<Hash> {
        #[derive(Encode, Decode)]
        struct ContractInfo {
            trie_id: Vec<u8>,
            code_hash: Hash,
        }
        // The pallet-contracts doesn't export an API the get the code hash. So we dig it out from the storage.
        let key = storage_map_prefix_twox_64_concat(b"Contracts", b"ContractInfoOf", &self.address);
        let value = storage.get(&key)?;
        let info = ContractInfo::decode(&mut &value[..]).ok()?;
        Some(info.code_hash)
    }
}

fn contract_tx<T>(
    origin: AccountId,
    block_number: BlockNumber,
    now: u64,
    gas_limit: Weight,
    gas_free: bool,
    tx_fn: impl FnOnce() -> ContractResult<T>,
) -> ContractResult<T> {
    System::set_block_number(block_number);
    Timestamp::set_timestamp(now);
    if !gas_free && PalletPink::pay_for_gas(&origin, gas_limit).is_err() {
        return ContractResult {
            gas_consumed: 0,
            gas_required: 0,
            storage_deposit: Default::default(),
            debug_message: Default::default(),
            result: Err(DispatchError::Other("InsufficientBalance")),
        };
    }
    let result = tx_fn();
    if !gas_free {
        let refund = gas_limit
            .checked_sub(&Weight::from_ref_time(result.gas_consumed))
            .expect("BUG: consumed gas more than the gas limit");
        PalletPink::refund_gas(&origin, refund).expect("BUG: failed to refund gas");
    }
    result
}

/// Calculates the Substrate storage key prefix for a StorageMap
pub fn storage_map_prefix_twox_64_concat(
    module: &[u8],
    storage_item: &[u8],
    key: &impl Encode,
) -> Vec<u8> {
    let mut bytes = sp_core::twox_128(module).to_vec();
    bytes.extend(&sp_core::twox_128(storage_item)[..]);
    let encoded = key.encode();
    bytes.extend(sp_core::twox_64(&encoded));
    bytes.extend(&encoded);
    bytes
}

pub fn transpose_contract_result(result: ContractExecResult) -> Result<Vec<u8>, DispatchError> {
    result.result.map(|v| v.data.0)
}

pub use contract_file::ContractFile;

mod contract_file {
    use impl_serde::serialize as bytes;
    use serde::Deserialize;
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ContractFile {
        pub metadata_version: String,
        pub source: Source,
        pub contract: Contract,
    }

    #[derive(Debug, Deserialize)]
    pub struct Source {
        #[serde(with = "bytes")]
        pub wasm: Vec<u8>,
        #[serde(with = "bytes")]
        pub hash: Vec<u8>,
        pub language: String,
        pub compiler: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct Contract {
        pub name: String,
        pub version: String,
    }

    impl ContractFile {
        pub fn load(json_contract: &[u8]) -> serde_json::Result<Self> {
            serde_json::from_slice(json_contract)
        }
    }
}

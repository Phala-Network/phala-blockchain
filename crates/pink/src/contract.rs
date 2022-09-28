use pallet_contracts_primitives::StorageDeposit;
use phala_types::contract::contract_id_preimage;
use pink_extension::predefined_accounts::ACCOUNT_RUNTIME;
use scale::{Decode, Encode};
use sp_core::hashing;
use sp_runtime::DispatchError;

use crate::{
    runtime::{BoxedEventCallbacks, Contracts, ExecSideEffects, System, Timestamp},
    storage,
    types::{
        AccountId, BlockNumber, Hash, COMMAND_GAS_LIMIT, INSTANTIATE_GAS_LIMIT, QUERY_GAS_LIMIT,
    },
};

type ContractExecResult = pallet_contracts_primitives::ContractExecResult<crate::types::Balance>;

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

#[derive(Debug)]
pub struct ExecError {
    pub source: DispatchError,
    pub message: String,
}

#[derive(Debug, Default, Encode, Decode, Clone)]
struct HookSelectors {
    on_block_end: Option<u32>,
}

#[derive(Debug, Encode, Decode, Clone)]
pub struct Contract {
    pub address: AccountId,
    hooks: HookSelectors,
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
        storage: &mut Storage,
        origin: AccountId,
        code_hash: Hash,
        input_data: Vec<u8>,
        cluster_id: Vec<u8>,
        salt: Vec<u8>,
        block_number: BlockNumber,
        now: u64,
        callbacks: Option<BoxedEventCallbacks>,
    ) -> Result<(Self, ExecSideEffects), ExecError> {
        if origin == AccountId::new(ACCOUNT_RUNTIME) {
            return Err(ExecError {
                source: DispatchError::BadOrigin,
                message: "Default account is not allowed to create contracts".to_string(),
            });
        }

        let (address, effects) =
            storage.execute_with(false, callbacks, move || -> Result<_, ExecError> {
                System::set_block_number(block_number);
                Timestamp::set_timestamp(now);

                let result = Contracts::bare_instantiate(
                    origin.clone(),
                    0,
                    INSTANTIATE_GAS_LIMIT,
                    None,
                    pallet_contracts_primitives::Code::Existing(code_hash),
                    input_data,
                    salt.clone(),
                    false,
                );
                log::info!("Contract instantiation result: {:?}", &result);
                match result.result {
                    Err(err) => {
                        return Err(ExecError {
                            source: err,
                            message: String::from_utf8_lossy(&result.debug_message).to_string(),
                        });
                    }
                    Ok(rv) => {
                        if rv.result.did_revert() {
                            return Err(ExecError {
                                source: DispatchError::Other("Contract reverted"),
                                message: String::from_utf8_lossy(&result.debug_message).to_string(),
                            });
                        }
                    }
                }
                let preimage = contract_id_preimage(
                    origin.as_ref(),
                    code_hash.as_ref(),
                    cluster_id.as_ref(),
                    salt.as_ref(),
                );
                Ok(AccountId::from(hashing::blake2_256(&preimage)))
            });
        Ok((Self::from_address(address?), effects))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_selector(
        storage: &mut Storage,
        origin: AccountId,
        code_hash: Hash,
        selector: [u8; 4],
        args: impl Encode,
        cluster_id: Vec<u8>,
        salt: Vec<u8>,
        block_number: BlockNumber,
        now: u64,
    ) -> Result<(Self, ExecSideEffects), ExecError> {
        let mut input_data = vec![];
        selector.encode_to(&mut input_data);
        args.encode_to(&mut input_data);
        Self::new(
            storage,
            origin,
            code_hash,
            input_data,
            cluster_id,
            salt,
            block_number,
            now,
            None,
        )
    }

    /// Call a contract method
    ///
    /// # Parameters
    /// * `input_data`: The SCALE encoded arguments including the 4-bytes selector as prefix.
    /// # Return
    /// Returns the SCALE encoded method return value.
    pub fn bare_call(
        &self,
        storage: &mut Storage,
        origin: AccountId,
        input_data: Vec<u8>,
        rollback: bool,
        block_number: BlockNumber,
        now: u64,
        callbacks: Option<BoxedEventCallbacks>,
    ) -> (ContractExecResult, ExecSideEffects) {
        if origin == AccountId::new(ACCOUNT_RUNTIME) {
            return (
                ContractExecResult {
                    gas_consumed: 0,
                    gas_required: 0,
                    debug_message: b"Default account is not allowed to call contracts".to_vec(),
                    result: Err(DispatchError::BadOrigin),
                    storage_deposit: StorageDeposit::Charge(0),
                },
                ExecSideEffects::default(),
            );
        }
        self.unchecked_bare_call(
            storage,
            origin,
            input_data,
            rollback,
            block_number,
            now,
            callbacks,
        )
    }

    fn unchecked_bare_call(
        &self,
        storage: &mut Storage,
        origin: AccountId,
        input_data: Vec<u8>,
        rollback: bool,
        block_number: BlockNumber,
        now: u64,
        callbacks: Option<BoxedEventCallbacks>,
    ) -> (ContractExecResult, ExecSideEffects) {
        let addr = self.address.clone();
        storage.execute_with(rollback, callbacks, move || {
            System::set_block_number(block_number);
            Timestamp::set_timestamp(now);
            let gas_limit = if rollback {
                QUERY_GAS_LIMIT
            } else {
                COMMAND_GAS_LIMIT
            };
            Contracts::bare_call(origin, addr, 0, gas_limit, None, input_data, false)
        })
    }

    /// Call a contract method given it's selector
    #[allow(clippy::too_many_arguments)]
    pub fn call_with_selector<RV: Decode>(
        &self,
        storage: &mut Storage,
        origin: AccountId,
        selector: [u8; 4],
        args: impl Encode,
        rollback: bool,
        block_number: BlockNumber,
        now: u64,
    ) -> Result<(RV, ExecSideEffects), ExecError> {
        let mut input_data = vec![];
        selector.encode_to(&mut input_data);
        args.encode_to(&mut input_data);
        let (result, effects) = self.bare_call(
            storage,
            origin,
            input_data,
            rollback,
            block_number,
            now,
            None,
        );
        let mut rv = transpose_contract_result(&result)?;
        Ok((
            Decode::decode(&mut rv).map_err(|_| ExecError {
                source: DispatchError::Other("Decode result failed"),
                message: Default::default(),
            })?,
            effects,
        ))
    }

    /// Called by on each block end by the runtime
    pub fn on_block_end(
        &self,
        storage: &mut Storage,
        block_number: BlockNumber,
        now: u64,
        callbacks: Option<BoxedEventCallbacks>,
    ) -> Result<ExecSideEffects, ExecError> {
        if let Some(selector) = self.hooks.on_block_end {
            let mut input_data = vec![];
            selector.to_be_bytes().encode_to(&mut input_data);

            let (result, effects) = self.unchecked_bare_call(
                storage,
                AccountId::new(ACCOUNT_RUNTIME),
                input_data,
                false,
                block_number,
                now,
                callbacks,
            );
            let _ = transpose_contract_result(&result)?;
            Ok(effects)
        } else {
            Ok(Default::default())
        }
    }

    pub fn set_on_block_end_selector(&mut self, selector: u32) {
        self.hooks.on_block_end = Some(selector)
    }
}

pub fn transpose_contract_result(result: &ContractExecResult) -> Result<&[u8], ExecError> {
    result
        .result
        .as_ref()
        .map(|v| &*v.data.0)
        .map_err(|err| ExecError {
            source: *err,
            message: String::from_utf8_lossy(&result.debug_message).to_string(),
        })
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

use scale::{Decode, Encode};
use sp_runtime::DispatchError;
use sp_core::Hasher as _;

use crate::{
    storage::{InMemoryBackend, Storage},
    runtime::Contracts,
    types::{AccountId, Hashing, ENOUGH, GAS_LIMIT},
};

pub struct Contract {
    storage: Storage<InMemoryBackend>,
    pub address: AccountId,
}

impl Contract {
    /// Create a new contract instance.
    ///
    /// # Parameters
    ///
    /// * `origin`: The owner of the created contract instance.
    /// * `code`: The contract code to deploy in raw bytes.
    /// * `input_data`: The input data to pass to the contract constructor.
    /// * `salt`: Used for the address derivation.
    pub fn new(
        origin: AccountId,
        code: Vec<u8>,
        input_data: Vec<u8>,
        salt: Vec<u8>,
    ) -> Result<Self, DispatchError> {
        let mut storage = Storage::new(InMemoryBackend::default());
        let code_hash = Hashing::hash(&code);

        let address = storage.execute_with(move || -> Result<_, DispatchError> {
            let _ = Contracts::bare_instantiate(
                origin.clone(),
                ENOUGH,
                GAS_LIMIT,
                pallet_contracts_primitives::Code::Upload(code.into()),
                input_data,
                salt.clone(),
                false,
                false,
            )
            .result?;
            Result::<_, DispatchError>::Ok(Contracts::contract_address(&origin, &code_hash, &salt))
        })?;

        Ok(Self { storage, address })
    }

    pub fn new_with_selector(
        origin: AccountId,
        code: Vec<u8>,
        selector: [u8; 4],
        args: impl Encode,
        salt: Vec<u8>,
    ) -> Result<Self, DispatchError> {
        let mut input_data = vec![];
        selector.encode_to(&mut input_data);
        args.encode_to(&mut input_data);
        Self::new(origin, code, input_data, salt)
    }

    /// Call a contract method
    ///
    /// # Parameters
    /// * `input_data`: The SCALE encoded arguments including the 4-bytes selector as prefix.
    /// # Return
    /// Returns the SCALE encoded method return value.
    pub fn bare_call(
        &mut self,
        origin: AccountId,
        input_data: Vec<u8>,
    ) -> Result<Vec<u8>, DispatchError> {
        let addr = self.address.clone();
        let rv = self
            .storage
            .execute_with(move || -> Result<_, DispatchError> {
                let result =
                    Contracts::bare_call(origin, addr, 0, GAS_LIMIT * 2, input_data, false);
                result.result
            })?;
        Ok(rv.data.0)
    }

    /// Call a contract method given it's selector
    pub fn call_with_selector<RV: Decode>(
        &mut self,
        origin: AccountId,
        selector: [u8; 4],
        args: impl Encode,
    ) -> Result<RV, DispatchError> {
        let mut input_data = vec![];
        selector.encode_to(&mut input_data);
        args.encode_to(&mut input_data);
        let rv = self.bare_call(origin, input_data)?;
        Ok(Decode::decode(&mut &rv[..]).or(Err(DispatchError::Other("Decode result failed")))?)
    }

    pub fn commit_storage_changes(&mut self) {
        let (root, transaction) = self.storage.changes_transaction();
        self.storage.commit_changes(root, transaction);
        self.storage.clear_changes();
    }
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

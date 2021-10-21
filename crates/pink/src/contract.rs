use scale::{Decode, Encode};
use sp_core::Hasher as _;
use sp_runtime::DispatchError;

use crate::{
    runtime::{Contracts, PinkEgressMessages, System},
    storage,
    types::{AccountId, BlockNumber, Hashing, ENOUGH, GAS_LIMIT},
};

pub type Storage = storage::Storage<storage::InMemoryBackend>;

#[derive(Debug)]
pub struct ExecError {
    source: DispatchError,
    message: String,
}

pub struct Contract {
    pub address: AccountId,
}

impl Contract {
    pub fn new_storage() -> Storage {
        Storage::new(Default::default())
    }

    /// Create a new contract instance.
    ///
    /// # Parameters
    ///
    /// * `origin`: The owner of the created contract instance.
    /// * `code`: The contract code to deploy in raw bytes.
    /// * `input_data`: The input data to pass to the contract constructor.
    /// * `salt`: Used for the address derivation.
    pub fn new(
        storage: &mut Storage,
        origin: AccountId,
        code: Vec<u8>,
        input_data: Vec<u8>,
        salt: Vec<u8>,
        block_number: BlockNumber,
    ) -> Result<Self, ExecError> {
        let code_hash = Hashing::hash(&code);

        let address = storage
            .execute_with(false, move || -> Result<_, ExecError> {
                System::set_block_number(block_number);

                let result = Contracts::bare_instantiate(
                    origin.clone(),
                    ENOUGH,
                    GAS_LIMIT,
                    pallet_contracts_primitives::Code::Upload(code.into()),
                    input_data,
                    salt.clone(),
                    false,
                    true,
                );
                match result.result {
                    Err(err) => {
                        return Err(ExecError {
                            source: err,
                            message: String::from_utf8_lossy(&result.debug_message).to_string(),
                        });
                    }
                    Ok(_) => (),
                }
                Ok(Contracts::contract_address(&origin, &code_hash, &salt))
            })
            .0?;

        Ok(Self { address })
    }

    pub fn new_with_selector(
        storage: &mut Storage,
        origin: AccountId,
        code: Vec<u8>,
        selector: [u8; 4],
        args: impl Encode,
        salt: Vec<u8>,
        block_number: BlockNumber,
    ) -> Result<Self, ExecError> {
        let mut input_data = vec![];
        selector.encode_to(&mut input_data);
        args.encode_to(&mut input_data);
        Self::new(storage, origin, code, input_data, salt, block_number)
    }

    /// Call a contract method
    ///
    /// # Parameters
    /// * `input_data`: The SCALE encoded arguments including the 4-bytes selector as prefix.
    /// # Return
    /// Returns the SCALE encoded method return value.
    pub fn bare_call(
        &mut self,
        storage: &mut Storage,
        origin: AccountId,
        input_data: Vec<u8>,
        rollback: bool,
        block_number: BlockNumber,
    ) -> Result<(Vec<u8>, PinkEgressMessages), ExecError> {
        let addr = self.address.clone();
        let (rv, messages) = storage.execute_with(rollback, move || -> Result<_, ExecError> {
            System::set_block_number(block_number);
            let result = Contracts::bare_call(origin, addr, 0, GAS_LIMIT * 2, input_data, true);
            match result.result {
                Err(err) => {
                    return Err(ExecError {
                        source: err,
                        message: String::from_utf8_lossy(&result.debug_message).to_string(),
                    });
                }
                Ok(rv) => Ok(rv),
            }
        });
        Ok((rv?.data.0, messages))
    }

    /// Call a contract method given it's selector
    pub fn call_with_selector<RV: Decode>(
        &mut self,
        storage: &mut Storage,
        origin: AccountId,
        selector: [u8; 4],
        args: impl Encode,
        rollback: bool,
        block_number: BlockNumber,
    ) -> Result<(RV, PinkEgressMessages), ExecError> {
        let mut input_data = vec![];
        selector.encode_to(&mut input_data);
        args.encode_to(&mut input_data);
        let (rv, messages) = self.bare_call(storage, origin, input_data, rollback, block_number)?;
        Ok((
            Decode::decode(&mut &rv[..]).or(Err(ExecError {
                source: DispatchError::Other("Decode result failed"),
                message: Default::default(),
            }))?,
            messages,
        ))
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

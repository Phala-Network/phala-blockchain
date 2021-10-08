mod runtime;
pub mod types;

use codec::{Decode, Encode};
use runtime::Contracts;
use sp_core::Hasher;
use sp_runtime::DispatchError;
use sp_state_machine::{
    disabled_changes_trie_state, Backend as StorageBackend, Ext, OverlayedChanges,
    StorageTransactionCache,
};
use types::{AccountId, Hashing, ENOUGH, GAS_LIMIT};

pub use sp_runtime::DispatchResultWithInfo as DispatchResult;

type InMemoryBackend = sp_state_machine::InMemoryBackend<Hashing>;

#[derive(Default)]
pub struct Storage<Backend> {
    backend: Backend,
    changes: OverlayedChanges,
}

impl<Backend> Storage<Backend>
where
    Backend: StorageBackend<Hashing>,
{
    pub fn new(backend: Backend) -> Self {
        Self {
            backend,
            changes: Default::default(),
        }
    }

    fn execute_with<R>(&mut self, f: impl FnOnce() -> R) -> R {
        let backend = self.backend.as_trie_backend().expect("No trie backend?");

        self.changes.start_transaction();
        let mut cache = StorageTransactionCache::default();
        let mut ext = Ext::new(
            &mut self.changes,
            &mut cache,
            backend,
            disabled_changes_trie_state::<_, u64>(),
            None,
        );
        let r = sp_externalities::set_and_run_with_externalities(&mut ext, f);
        self.changes
            .commit_transaction()
            .expect("BUG: mis-paired transaction");
        r
    }
}

pub struct PinkContract {
    storage: Storage<InMemoryBackend>,
    pub address: AccountId,
}

impl PinkContract {
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
}

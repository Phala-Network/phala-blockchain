mod runtime;
pub mod types;

use codec::Encode;
use runtime::{Contracts, Origin, PinkRuntime};
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

// TODO: generic HASHING
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
    address: AccountId,
}

impl PinkContract {
    pub fn new(wasm_binary: Vec<u8>, owner: AccountId) -> Result<Self, DispatchError> {
        let mut storage = Storage::new(InMemoryBackend::default());
        let code_hash = Hashing::hash(&wasm_binary);

        let address = storage.execute_with(move || {
            let _ = Contracts::instantiate_with_code(
                Origin::signed(owner.clone()),
                ENOUGH,
                GAS_LIMIT,
                wasm_binary,
                vec![],
                vec![],
            )
            .map_err(|e| e.error)?;
            Result::<_, DispatchError>::Ok(Contracts::contract_address(&owner, &code_hash, &[]))
        })?;

        Ok(Self { storage, address })
    }

    pub fn call(
        &mut self,
        origin: AccountId,
        method: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, DispatchError> {
        let addr = self.address.clone();
        self.storage.execute_with(move || {
            Contracts::call(
                Origin::signed(origin),
                addr,
                0,
                GAS_LIMIT * 2,
                <PinkRuntime as pallet_contracts::Config>::Schedule::get()
                    .limits
                    .payload_len
                    .encode(),
            )
            .unwrap();
        });

        todo!()
    }
}

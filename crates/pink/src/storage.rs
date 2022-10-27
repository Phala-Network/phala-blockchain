use crate::{
    runtime::{BoxedEventCallbacks, ExecSideEffects, Pink as PalletPink},
    types::{AccountId, Balance, Hash, Hashing},
};
use phala_crypto::sr25519::Sr25519SecretKey;
use phala_trie_storage::{deserialize_trie_backend, serialize_trie_backend, MemoryDB};
use serde::{Deserialize, Serialize};
use sp_runtime::DispatchError;
use sp_state_machine::backend::AsTrieBackend;
use sp_state_machine::{Backend as StorageBackend, Ext, OverlayedChanges, StorageTransactionCache};

mod backend;

pub type InMemoryBackend = phala_trie_storage::InMemoryBackend<Hashing>;

pub fn new_in_memory_backend() -> InMemoryBackend {
    let db = MemoryDB::default();
    // V1 is same as V0 for an empty trie.
    sp_state_machine::TrieBackendBuilder::new(
        db,
        sp_trie::empty_trie_root::<sp_state_machine::LayoutV1<Hashing>>(),
    )
    .build()
}

pub trait CommitTransaction: StorageBackend<Hashing> {
    fn commit_transaction(&mut self, root: Hash, transaction: Self::Transaction);
}

impl CommitTransaction for InMemoryBackend {
    fn commit_transaction(&mut self, root: Hash, transaction: Self::Transaction) {
        let mut storage = sp_std::mem::replace(self, new_in_memory_backend()).into_storage();
        storage.consolidate(transaction);
        *self = sp_state_machine::TrieBackendBuilder::new(storage, root).build();
    }
}

pub trait Snapshot {
    fn snapshot(&self) -> Self;
}

pub struct Storage<Backend> {
    backend: Backend,
}

impl<Backend> Storage<Backend> {
    pub fn new(backend: Backend) -> Self {
        Self { backend }
    }
}

impl<Backend> Storage<Backend>
where
    Backend: Snapshot,
{
    pub fn snapshot(&self) -> Self {
        Self {
            backend: self.backend.snapshot(),
        }
    }
}

impl<Backend> Storage<Backend>
where
    Backend: StorageBackend<Hashing> + CommitTransaction + AsTrieBackend<Hashing>,
{
    pub fn execute_with<R>(
        &mut self,
        rollback: bool,
        callbacks: Option<BoxedEventCallbacks>,
        f: impl FnOnce() -> R,
    ) -> (R, ExecSideEffects) {
        let backend = self.backend.as_trie_backend();

        let mut overlay = OverlayedChanges::default();
        overlay.start_transaction();
        let mut cache = StorageTransactionCache::default();
        let mut ext = Ext::new(&mut overlay, &mut cache, backend, None);
        let r = sp_externalities::set_and_run_with_externalities(&mut ext, move || {
            crate::runtime::System::reset_events();
            let mode = if rollback {
                crate::runtime::CallMode::Query
            } else {
                crate::runtime::CallMode::Command
            };
            let r = crate::runtime::using_mode(mode, callbacks, f);
            (r, crate::runtime::get_side_effects())
        });
        overlay
            .commit_transaction()
            .expect("BUG: mis-paired transaction");
        if !rollback {
            self.commit_changes(overlay);
        }
        r
    }

    pub fn changes_transaction(&self, changes: OverlayedChanges) -> (Hash, Backend::Transaction) {
        let delta = changes
            .changes()
            .map(|(k, v)| (&k[..], v.value().map(|v| &v[..])));
        let child_delta = changes.children().map(|(changes, info)| {
            (
                info,
                changes.map(|(k, v)| (&k[..], v.value().map(|v| &v[..]))),
            )
        });

        self.backend
            .full_storage_root(delta, child_delta, sp_core::storage::StateVersion::V0)
    }

    pub fn commit_changes(&mut self, changes: OverlayedChanges) {
        let (root, transaction) = self.changes_transaction(changes);
        self.backend.commit_transaction(root, transaction)
    }

    pub fn set_cluster_id(&mut self, cluster_id: &[u8]) {
        self.execute_with(false, None, || {
            PalletPink::set_cluster_id(cluster_id);
        });
    }
    pub fn config_price(
        &mut self,
        gas_price: Balance,
        deposit_per_item: Balance,
        deposit_per_byte: Balance,
    ) {
        self.execute_with(false, None, || {
            PalletPink::set_gas_price(gas_price);
            PalletPink::set_deposit_per_item(deposit_per_item);
            PalletPink::set_deposit_per_byte(deposit_per_byte);
        });
    }

    pub fn set_key_seed(&mut self, seed: Sr25519SecretKey) {
        self.execute_with(false, None, || {
            PalletPink::set_key_seed(seed);
        });
    }

    pub fn upload_code(
        &mut self,
        account: AccountId,
        code: Vec<u8>,
    ) -> Result<Hash, DispatchError> {
        self.execute_with(false, None, || {
            crate::runtime::Contracts::bare_upload_code(account, code, None)
        })
        .0
        .map(|v| v.code_hash)
    }

    pub fn upload_sidevm_code(
        &mut self,
        account: AccountId,
        code: Vec<u8>,
    ) -> Result<Hash, DispatchError> {
        Ok(self
            .execute_with(false, None, || PalletPink::put_sidevm_code(account, code))
            .0)
    }

    pub fn get_sidevm_code(&mut self, hash: &Hash) -> Option<Vec<u8>> {
        self.execute_with(false, None, || {
            PalletPink::sidevm_codes(&hash).map(|v| v.code)
        })
        .0
    }

    pub fn set_system_contract(&mut self, address: AccountId) {
        self.execute_with(false, None, move || {
            PalletPink::set_system_contract(address);
        });
    }

    pub fn system_contract(&mut self) -> Option<AccountId> {
        self.execute_with(true, None, PalletPink::system_contract).0
    }

    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.backend.storage(key).ok().flatten()
    }

    pub fn root(&self) -> Hash {
        *self.backend.as_trie_backend().root()
    }
}

impl Serialize for Storage<InMemoryBackend> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let trie = self.backend.as_trie_backend();
        serialize_trie_backend(trie, serializer)
    }
}

impl<'de> Deserialize<'de> for Storage<InMemoryBackend> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::new(deserialize_trie_backend(deserializer)?))
    }
}

use crate::{
    runtime::{BoxedEventCallbacks, ExecSideEffects},
    types::{Hash, Hashing},
};
pub use external_backend::ExternalStorage;
pub use in_memory_backend::InMemoryStorage;
use phala_trie_storage::{deserialize_trie_backend, serialize_trie_backend, MemoryDB};
use pink_capi::types::ExecutionMode;
use serde::{Deserialize, Serialize};
use sp_state_machine::{
    backend::AsTrieBackend, Backend as StorageBackend, Ext, OverlayedChanges,
    StorageTransactionCache,
};

pub mod external_backend;
pub mod in_memory_backend;

pub trait CommitTransaction: StorageBackend<Hashing> {
    fn commit_transaction(&mut self, root: Hash, transaction: Self::Transaction);
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
        &self,
        mode: ExecutionMode,
        callbacks: Option<BoxedEventCallbacks>,
        f: impl FnOnce() -> R,
    ) -> (R, ExecSideEffects, OverlayedChanges) {
        let backend = self.backend.as_trie_backend();

        let mut overlay = OverlayedChanges::default();
        overlay.start_transaction();
        let mut cache = StorageTransactionCache::default();
        let mut ext = Ext::new(&mut overlay, &mut cache, backend, None);
        let (rv, effects) = sp_externalities::set_and_run_with_externalities(&mut ext, move || {
            let todo = "set block number";
            // System::set_block_number(block_number);
            // Timestamp::set_timestamp(now);
            crate::runtime::System::reset_events();
            let r = crate::runtime::using_mode(mode, callbacks, f);
            (r, crate::runtime::get_side_effects())
        });
        overlay
            .commit_transaction()
            .expect("BUG: mis-paired transaction");
        (rv, effects, overlay)
    }

    pub fn execute_mut<R>(
        &mut self,
        mode: ExecutionMode,
        callbacks: Option<BoxedEventCallbacks>,
        f: impl FnOnce() -> R,
    ) -> (R, ExecSideEffects) {
        let (rv, effects, overlay) = self.execute_with(mode, callbacks, f);
        self.commit_changes(overlay);
        (rv, effects)
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
}

impl Serialize for InMemoryStorage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let trie = self.backend.as_trie_backend();
        serialize_trie_backend(trie, serializer)
    }
}

impl<'de> Deserialize<'de> for InMemoryStorage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::new(deserialize_trie_backend(deserializer)?))
    }
}

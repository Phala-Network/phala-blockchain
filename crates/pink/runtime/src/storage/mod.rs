use crate::{
    runtime::{ExecSideEffects, Pink, System, Timestamp},
    types::{Hash, Hashing},
};
use pink_capi::v1::ocall::ExecContext;
use sp_state_machine::{
    backend::AsTrieBackend, Backend as StorageBackend, Ext, OverlayedChanges,
    StorageTransactionCache,
};

pub use external_backend::ExternalStorage;

pub trait CommitTransaction: StorageBackend<Hashing> {
    fn commit_transaction(&mut self, root: Hash, transaction: Self::Transaction);
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
    Backend: StorageBackend<Hashing> + CommitTransaction + AsTrieBackend<Hashing>,
{
    pub fn execute_with<R>(
        &self,
        exec_context: &ExecContext,
        f: impl FnOnce() -> R,
    ) -> (R, ExecSideEffects, OverlayedChanges) {
        let backend = self.backend.as_trie_backend();

        let mut overlay = OverlayedChanges::default();
        overlay.start_transaction();
        let mut cache = StorageTransactionCache::default();
        let mut ext = Ext::new(&mut overlay, &mut cache, backend, None);
        let (rv, effects) = sp_externalities::set_and_run_with_externalities(&mut ext, move || {
            Timestamp::set_timestamp(exec_context.now_ms);
            System::set_block_number(exec_context.block_number);
            System::reset_events();
            let result = f();
            let events = crate::runtime::get_events();
            Pink::apply_code_changes(&events.code_changes);
            (result, events.side_effects)
        });
        overlay
            .commit_transaction()
            .expect("BUG: mis-paired transaction");
        (rv, effects, overlay)
    }

    pub fn execute_mut<R>(
        &mut self,
        context: &ExecContext,
        f: impl FnOnce() -> R,
    ) -> (R, ExecSideEffects) {
        let (rv, effects, overlay) = self.execute_with(context, f);
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

    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.backend
            .as_trie_backend()
            .storage(key)
            .expect("Failed to get storage key")
    }
}

pub mod external_backend;

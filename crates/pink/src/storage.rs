use crate::types::{BlockNumber, Hash, Hashing};
use sp_state_machine::{
    disabled_changes_trie_state, Backend as StorageBackend, Ext, OverlayedChanges,
    StorageTransactionCache,
};

pub type InMemoryBackend = sp_state_machine::InMemoryBackend<Hashing>;

pub trait CommitTransaction: StorageBackend<Hashing> {
    fn commit_transaction(&mut self, root: Hash, transaction: Self::Transaction);
}

impl CommitTransaction for InMemoryBackend {
    fn commit_transaction(&mut self, root: Hash, transaction: Self::Transaction) {
        self.apply_transaction(root, transaction);
    }
}

#[derive(Default)]
pub struct Storage<Backend> {
    backend: Backend,
    overlay: OverlayedChanges,
}

impl<Backend> Storage<Backend>
where
    Backend: StorageBackend<Hashing> + CommitTransaction,
{
    pub fn new(backend: Backend) -> Self {
        Self {
            backend,
            overlay: Default::default(),
        }
    }

    pub fn execute_with<R>(&mut self, f: impl FnOnce() -> R) -> R {
        let backend = self.backend.as_trie_backend().expect("No trie backend?");

        self.overlay.start_transaction();
        let mut cache = StorageTransactionCache::default();
        let mut ext = Ext::new(
            &mut self.overlay,
            &mut cache,
            backend,
            disabled_changes_trie_state::<_, BlockNumber>(),
            None,
        );
        let r = sp_externalities::set_and_run_with_externalities(&mut ext, f);
        self.overlay
            .commit_transaction()
            .expect("BUG: mis-paired transaction");
        r
    }

    pub fn changes_transaction(&self) -> (Hash, Backend::Transaction) {
        let delta = self
            .overlay
            .changes()
            .map(|(k, v)| (&k[..], v.value().map(|v| &v[..])));
        let child_delta = self.overlay.children().map(|(changes, info)| {
            (
                info,
                changes.map(|(k, v)| (&k[..], v.value().map(|v| &v[..]))),
            )
        });

        self.backend.full_storage_root(delta, child_delta)
    }

    pub fn commit_changes(&mut self, root: Hash, transaction: Backend::Transaction) {
        self.backend.commit_transaction(root, transaction)
    }

    pub fn clear_changes(&mut self) {
        self.overlay = Default::default();
    }
}

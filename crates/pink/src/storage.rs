use crate::{
    runtime::{BoxedEventCallbacks, ExecSideEffects},
    types::{AccountId, Hash, Hashing},
};
use phala_crypto::sr25519::Sr25519SecretKey;
use phala_trie_storage::{deserialize_trie_backend, serialize_trie_backend, KeyValueDB};
use serde::{Deserialize, Serialize};
use sp_runtime::DispatchError;
use sp_state_machine::{Backend as StorageBackend, Ext, OverlayedChanges, StorageTransactionCache};

mod backend;

pub type KeyValueDBBackend = phala_trie_storage::KeyValueDBBackend<Hashing>;

pub fn new_backend() -> KeyValueDBBackend {
    let db = KeyValueDB::new();
    // V1 is same as V0 for an empty trie.
    sp_state_machine::TrieBackend::new(
        db,
        sp_trie::empty_trie_root::<sp_state_machine::LayoutV1<Hashing>>(),
    )
}

pub trait CommitTransaction: StorageBackend<Hashing> {
    fn commit_transaction(&mut self, root: Hash, transaction: Self::Transaction);
}

impl CommitTransaction for KeyValueDBBackend {
    fn commit_transaction(&mut self, root: Hash, transaction: Self::Transaction) {
        let storage = sp_std::mem::replace(self, new_backend()).into_storage();
        storage.consolidate(transaction);
        *self = sp_state_machine::TrieBackend::new(storage, root);
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
    Backend: StorageBackend<Hashing> + CommitTransaction,
{
    pub fn execute_with<R>(
        &mut self,
        rollback: bool,
        callbacks: Option<BoxedEventCallbacks>,
        f: impl FnOnce() -> R,
    ) -> (R, ExecSideEffects) {
        let backend = self.backend.as_trie_backend().expect("No trie backend?");

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
            crate::runtime::Pink::set_cluster_id(cluster_id);
        });
    }

    pub fn set_key_seed(&mut self, seed: Sr25519SecretKey) {
        self.execute_with(false, None, || {
            crate::runtime::Pink::set_key_seed(seed);
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
        Ok(self.execute_with(false, None, || {
            crate::runtime::Pink::put_sidevm_code(account, code)
        }).0)
    }

    pub fn get_sidevm_code(&mut self, hash: &Hash) -> Option<Vec<u8>> {
        self.execute_with(false, None, || {
            crate::runtime::Pink::sidevm_codes(&hash).map(|v| v.code)
        })
        .0
    }
}

impl Serialize for Storage<KeyValueDBBackend> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let trie = self.backend.as_trie_backend().unwrap();
        serialize_trie_backend(trie, serializer)
    }
}

impl<'de> Deserialize<'de> for Storage<KeyValueDBBackend> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::new(deserialize_trie_backend(deserializer)?))
    }
}

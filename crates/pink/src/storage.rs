use crate::{
    runtime::ExecSideEffects,
    types::{AccountId, Hash, Hashing},
};
use phala_crypto::sr25519::Sr25519SecretKey;
use pkvdb::leveldb::Kvdb;
use pkvdb::snapshot::SnapshotDB;
use pkvdb::trie::CommitTransaction as TrieCommitTransaction;
use pkvdb::trie::DBValue;
use sp_runtime::DispatchError;
use sp_state_machine::TrieBackend;
use sp_state_machine::{Ext, OverlayedChanges, StorageTransactionCache};

// used in pink to driven the contract running
// FIXME: should use the phala trie storage type for this usage
pub type PinkTrieBackend = TrieBackend<Kvdb<Hashing, DBValue>, Hashing>;
pub type PinkSnapshotBackend = TrieBackend<SnapshotDB<Hashing, DBValue>, Hashing>;

pub struct Storage<Backend> {
    pub(crate) backend: Backend,
}

impl<Backend> Storage<Backend> {
    pub fn new(backend: Backend) -> Self {
        Self { backend }
    }
}

impl<Backend> Storage<Backend>
where
    Backend: TrieCommitTransaction<Hashing>,
{
    // execute the contract over current storage object
    // if rollback is true no changes will be submit to the storage
    pub fn execute_with<R>(
        &mut self,
        rollback: bool,
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
                // Note: makesure the ext carried the PinkSnapshotBackend
                crate::runtime::CallMode::Query
            } else {
                // Note: makesure the ext carried the PinkTrieBackend
                crate::runtime::CallMode::Command
            };
            let r = crate::runtime::using_mode(mode, f);
            (r, crate::runtime::get_side_effects())
        });
        overlay
            .commit_transaction()
            .expect("BUG: mis-paired transaction");
        if !rollback {
            // just directly commit overlay changes 
            self.commit(overlay);
        }
        r
    }

    // FIXME: should make the method as private?
    pub fn commit(&mut self, changes: OverlayedChanges) {
        let delta = changes
            .changes()
            .map(|(k, v)| (&k[..], v.value().map(|v| &v[..])));
        let child_deltas = changes.children().map(|(changes, info)| {
            (
                info,
                changes.map(|(k, v)| (&k[..], v.value().map(|v| &v[..]))),
            )
        });
        let (root, transaction) =
            self.backend
                .full_storage_root(delta, child_deltas, sp_core::storage::StateVersion::V0);
        self.backend.commit_transaction(root, transaction);
    }

    pub fn set_cluster_id(&mut self, cluster_id: &[u8]) {
        self.execute_with(false, || {
            crate::runtime::Pink::set_cluster_id(cluster_id);
        });
    }

    pub fn set_key_seed(&mut self, seed: Sr25519SecretKey) {
        self.execute_with(false, || {
            crate::runtime::Pink::set_key_seed(seed);
        });
    }

    pub fn upload_code(
        &mut self,
        account: AccountId,
        code: Vec<u8>,
    ) -> Result<Hash, DispatchError> {
        self.execute_with(false, || {
            crate::runtime::Contracts::bare_upload_code(account, code, None)
        })
        .0
        .map(|v| v.code_hash)
    }
}

pub mod helper {
    use super::*;
    use crate::types::Hashing;
    use sp_state_machine::InMemoryBackend;

    // only useful in test
    pub fn new_in_memory() -> Storage<InMemoryBackend<Hashing>> {
        Storage::new(Default::default())
    }
}

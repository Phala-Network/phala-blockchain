use crate::{
    runtime::ExecSideEffects,
    types::{AccountId, Hash, Hashing},
};
use phala_crypto::sr25519::Sr25519SecretKey;
use pkvdb::PhalaTrieBackend;
use pkvdb::TransactionalBackend;
use sp_state_machine::Backend;
use sp_runtime::DispatchError;
use sp_state_machine::{Ext, OverlayedChanges, StorageTransactionCache};

pub type PinkTrieBackend = PhalaTrieBackend<Hashing>;

pub struct Storage{
    // should use the ref instead the instance 
    pub(crate) backend: PinkTrieBackend,
}

impl Storage{
    pub fn new(backend: PinkTrieBackend) -> Self {
        Self { backend }
    }
}

impl Storage
{
    // execute the contract over current storage object
    // if rollback is true no changes will be submit to the storage
    pub fn execute_with<R>(
        &mut self,
        rollback: bool,
        f: impl FnOnce() -> R,
    ) -> (R, ExecSideEffects) {
        let backend = self.backend.0.as_trie_backend().expect("No trie backend?");
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
            self.backend.0
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
    use pkvdb::new_memory_backend;

    // only useful in test
    pub fn new_in_memory() -> Storage {
        Storage::new(new_memory_backend::<Hashing>())
    }
}

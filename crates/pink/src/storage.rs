use crate::{
    runtime::ExecSideEffects,
    types::{AccountId, Hash, Hashing},
};
use phala_crypto::sr25519::Sr25519SecretKey;
use pkvdb::PhalaTrieBackend;
use pkvdb::TransactionalBackend;
use sp_runtime::DispatchError;
use sp_state_machine::Backend;
use sp_state_machine::{Ext, OverlayedChanges, StorageTransactionCache};

// use the atomic reference for inner backend
pub type PinkTrieBackend = PhalaTrieBackend<Hashing>;

pub enum MixinPinkTrieBackend<'a> {
    Move(PhalaTrieBackend<Hashing>),
    RefMut(&'a mut PhalaTrieBackend<Hashing>),
}

impl<'a> AsMut<PinkTrieBackend> for MixinPinkTrieBackend<'a> {
    fn as_mut(&mut self) -> &mut PinkTrieBackend{
        match self {
            Self::Move(p) => p,
            Self::RefMut( p) => p,
        }
    }
}

pub struct Storage<'a> {
    pub(crate) backend: MixinPinkTrieBackend<'a>,
}

impl Storage<'_> {
    pub fn new(backend: PinkTrieBackend) -> Self {
        Self { backend: MixinPinkTrieBackend::Move(backend) }
    }
}

impl<'a> Storage<'a> {
    pub fn new_with_mut(backend: &'a mut PinkTrieBackend) -> Self {
        Self { backend: MixinPinkTrieBackend::RefMut(backend) }
    }
}

impl<'a> Storage<'a> {
    // execute the contract over current storage object
    // if rollback is true no changes will be submit to the storage
    pub fn execute_with<R>(
        &mut self,
        rollback: bool,
        f: impl FnOnce() -> R,
    ) -> (R, ExecSideEffects) {
        let backend = self.backend.as_mut().0.as_trie_backend().expect("No trie backend?");
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
            self.backend.as_mut().0
                .full_storage_root(delta, child_deltas, sp_core::storage::StateVersion::V0);
        self.backend.as_mut().commit_transaction(root, transaction);
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
    use pkvdb::helper::new_in_memory_backend;

    // only useful in test
    pub fn new_in_memory() -> Storage<'static> {
        Storage::new(new_in_memory_backend::<Hashing>())
    }
}

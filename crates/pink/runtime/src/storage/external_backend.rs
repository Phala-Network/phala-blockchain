use super::{CommitTransaction, Storage};
use crate::{capi::OCallImpl, types::Hashing};
use hash_db::Prefix;
use phala_trie_storage::MemoryDB;
use pink_capi::v1::ocall::OCalls;
use sp_core::Hasher;
use sp_state_machine::{
    DBValue, DefaultError, TrieBackend, TrieBackendBuilder, TrieBackendStorage,
};

type Hash = <Hashing as Hasher>::Out;

pub struct ExternalDB;
pub type ExternalBackend = TrieBackend<ExternalDB, Hashing>;
pub type ExternalStorage = Storage<ExternalBackend>;

impl TrieBackendStorage<Hashing> for ExternalDB {
    type Overlay = MemoryDB<Hashing>;

    fn get(&self, key: &Hash, _prefix: Prefix) -> Result<Option<DBValue>, DefaultError> {
        Ok(OCallImpl.storage_get(key.as_ref().to_vec()))
    }
}

impl CommitTransaction for ExternalBackend {
    fn commit_transaction(&mut self, root: Hash, mut transaction: Self::Transaction) {
        let changes = transaction
            .drain()
            .into_iter()
            .map(|(k, v)| (k.as_bytes().to_vec(), v))
            .collect();
        OCallImpl.storage_commit(root, changes)
    }
}

impl ExternalStorage {
    pub fn instantiate() -> Self {
        let root = OCallImpl
            .storage_root()
            .unwrap_or_else(|| sp_trie::empty_trie_root::<sp_state_machine::LayoutV1<Hashing>>());
        let backend = TrieBackendBuilder::new(ExternalDB, root).build();
        crate::storage::Storage::new(backend)
    }
}

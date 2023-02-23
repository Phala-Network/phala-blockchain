use super::{CommitTransaction, Hash, Hashing, MemoryDB, Storage};

pub type InMemoryStorage = Storage<InMemoryBackend>;

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self::new(new_in_memory_backend())
    }
}

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

impl CommitTransaction for InMemoryBackend {
    fn commit_transaction(&mut self, root: Hash, transaction: Self::Transaction) {
        let mut storage = sp_std::mem::replace(self, new_in_memory_backend()).into_storage();
        storage.consolidate(transaction);
        *self = sp_state_machine::TrieBackendBuilder::new(storage, root).build();
    }
}

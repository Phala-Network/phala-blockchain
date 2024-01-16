//! An in-memory database backend intended for unit testing purposes.

use sp_state_machine::TrieBackend;
use sp_trie::PrefixedMemoryDB;

use super::{Hashing, Storage};

pub type InMemoryStorage = Storage<InMemoryBackend>;

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self::new(new_in_memory_backend())
    }
}

pub type InMemoryBackend = TrieBackend<PrefixedMemoryDB<Hashing>, Hashing>;

pub fn new_in_memory_backend() -> InMemoryBackend {
    let db = Default::default();
    // V1 is same as V0 for an empty trie.
    sp_state_machine::TrieBackendBuilder::new(
        db,
        sp_trie::empty_trie_root::<sp_state_machine::LayoutV1<Hashing>>(),
    )
    .build()
}

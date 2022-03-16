use phala_trie_storage::clone_trie_backend;

use crate::Storage;

use super::Snapshot;

impl Snapshot for Storage {
    fn snapshot(&self) -> Self {
        Storage {
            // TODO.kevin: This is a heavy deep copy. Replace with an optimized snapshot implementation.
            backend: clone_trie_backend(&self.backend),
        }
    }
}

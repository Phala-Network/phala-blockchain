use phala_trie_storage::snapshot;

use crate::Storage;

use super::Snapshot;

impl Snapshot for Storage {
    fn snapshot(&self) -> Self {
        Storage {
            // TODO.kevin: This is a heavy deep copy. Replace with an optimized snapshot implementation.
            backend: snapshot(&self.backend),
        }
    }
}

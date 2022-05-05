use crate::Inner;
use crate::{TRIE_MEMORY_MUST_HOLD_LOCK, TRIE_UNDERLYING_DB_MUST_HOLD_LOCK};
use hash_db::{AsHashDB, AsPlainDB, HashDB, HashDBRef, Hasher, PlainDB, PlainDBRef, Prefix};
use rusty_leveldb::DB;
use sp_std::borrow::Borrow;
use sp_std::borrow::BorrowMut;
use sp_std::convert::AsRef;
use sp_std::sync::Arc;
use sp_std::sync::Mutex;
use sp_trie::DBValue;
use sp_trie::MemoryDB;

// TrieEssenceStorage used to as the sp_state_machine::TrieBackend::S
// the storage should be derived from the outer PhalaStorage with different memory layer
// also support pure memory storage useful for test situation
pub enum TrieEssenceStorage<H: Hasher> {
    // FIXME: if the storage need prefix for different TrieStorage ?
    Disk {
        disk: Arc<Mutex<DB>>,
        memory: Arc<Mutex<MemoryDB<H>>>,
    },
    Memory(Arc<Mutex<MemoryDB<H>>>),
}

impl<H: Hasher> TrieEssenceStorage<H> {
    // commit transaction from the overlay in pink
    pub fn commit_transaction(&self, transaction: MemoryDB<H>) {
        let mut memory = match self {
            Self::Disk { disk: _, memory } => memory.lock().expect(TRIE_MEMORY_MUST_HOLD_LOCK),
            Self::Memory(memory) => memory.lock().expect(TRIE_MEMORY_MUST_HOLD_LOCK),
        };
        memory.borrow_mut().consolidate(transaction);
    }
}

impl<H: Hasher> PlainDB<H::Out, DBValue> for TrieEssenceStorage<H>
where
    H::Out: Borrow<[u8]>,
    H::Out: for<'a> From<&'a [u8]>,
    H::Out: AsRef<[u8]>,
{
    fn get(&self, key: &H::Out) -> Option<DBValue> {
        match self {
            Self::Disk { disk, memory } => {
                let memory = memory.lock().expect(TRIE_MEMORY_MUST_HOLD_LOCK);
                PlainDB::get(memory.as_plain_db(), key).or_else(|| {
                    let mut db = disk.lock().expect(TRIE_UNDERLYING_DB_MUST_HOLD_LOCK);
                    match DB::get(db.borrow_mut(), key.as_ref()).map(Inner::<DBValue>::from) {
                        Some(Inner(value, rc)) if rc > 0 => Some(value),
                        _ => None,
                    }
                })
            }
            Self::Memory(memory) => {
                let memory = memory.lock().expect(TRIE_MEMORY_MUST_HOLD_LOCK);
                PlainDB::get(memory.as_plain_db(), key)
            }
        }
    }

    fn contains(&self, key: &H::Out) -> bool {
        match self {
            Self::Disk { disk, memory } => {
                let memory = memory.lock().expect(TRIE_MEMORY_MUST_HOLD_LOCK);
                if PlainDB::contains(memory.as_plain_db(), key) {
                    true
                } else {
                    let mut db = disk.lock().expect(TRIE_UNDERLYING_DB_MUST_HOLD_LOCK);
                    match DB::get(db.borrow_mut(), key.as_ref()).map(Inner::<DBValue>::from) {
                        Some(Inner(_, rc)) if rc > 0 => true,
                        _ => false,
                    }
                }
            }
            Self::Memory(memory) => {
                let memory = memory.lock().expect(TRIE_MEMORY_MUST_HOLD_LOCK);
                PlainDB::contains(memory.as_plain_db(), key)
            }
        }
    }

    fn emplace(&mut self, key: H::Out, value: DBValue) {
        let memory = match self {
            Self::Disk { disk: _, memory } => memory,
            Self::Memory(memory) => memory,
        };
        let mut memory = memory.lock().expect(TRIE_MEMORY_MUST_HOLD_LOCK);
        PlainDB::emplace(memory.as_plain_db_mut(), key, value)
    }

    fn remove(&mut self, key: &H::Out) {
        let memory = match self {
            Self::Disk { disk: _, memory } => memory,
            Self::Memory(memory) => memory,
        };
        let mut memory = memory.lock().expect(TRIE_MEMORY_MUST_HOLD_LOCK);
        PlainDB::remove(memory.as_plain_db_mut(), key)
    }
}

impl<H> PlainDBRef<H::Out, DBValue> for TrieEssenceStorage<H>
where
    H: Hasher,
    H::Out: Borrow<[u8]>,
    H::Out: for<'a> From<&'a [u8]>,
{
    fn get(&self, key: &H::Out) -> Option<DBValue> {
        PlainDB::get(self, key)
    }

    fn contains(&self, key: &H::Out) -> bool {
        PlainDB::contains(self, key)
    }
}

impl<H> AsPlainDB<H::Out, DBValue> for TrieEssenceStorage<H>
where
    H: Hasher,
    H::Out: Borrow<[u8]>,
    H::Out: for<'a> From<&'a [u8]>,
{
    fn as_plain_db(&self) -> &dyn PlainDB<H::Out, DBValue> {
        self
    }

    fn as_plain_db_mut(&mut self) -> &mut dyn PlainDB<H::Out, DBValue> {
        self
    }
}

impl<H: Hasher> HashDB<H, DBValue> for TrieEssenceStorage<H>
where
    H::Out: AsRef<[u8]>,
{
    fn get(&self, key: &H::Out, prefix: Prefix) -> Option<DBValue> {
        match self {
            Self::Disk { disk, memory } => {
                let memory = memory.lock().expect(TRIE_MEMORY_MUST_HOLD_LOCK);
                HashDB::get(memory.as_hash_db(), key, prefix).or_else(|| {
                    let mut db = disk.lock().expect(TRIE_UNDERLYING_DB_MUST_HOLD_LOCK);
                    match DB::get(db.borrow_mut(), key.as_ref()).map(Inner::<DBValue>::from) {
                        Some(Inner(value, rc)) if rc > 0 => Some(value),
                        _ => None,
                    }
                })
            }
            Self::Memory(memory) => {
                let memory = memory.lock().expect(TRIE_MEMORY_MUST_HOLD_LOCK);
                HashDB::get(memory.as_hash_db(), key, prefix)
            }
        }
    }

    fn contains(&self, key: &H::Out, prefix: Prefix) -> bool {
        match self {
            Self::Disk { disk, memory } => {
                let memory = memory.lock().expect(TRIE_MEMORY_MUST_HOLD_LOCK);
                if HashDB::contains(memory.as_hash_db(), key, prefix) {
                    true
                } else {
                    let mut db = disk.lock().expect(TRIE_UNDERLYING_DB_MUST_HOLD_LOCK);
                    match DB::get(db.borrow_mut(), key.as_ref()).map(Inner::<DBValue>::from) {
                        Some(Inner(_, rc)) if rc > 0 => true,
                        _ => false,
                    }
                }
            }
            Self::Memory(memory) => {
                let memory = memory.lock().expect(TRIE_MEMORY_MUST_HOLD_LOCK);
                HashDB::contains(memory.as_hash_db(), key, prefix)
            }
        }
    }

    fn insert(&mut self, prefix: Prefix, value: &[u8]) -> H::Out {
        let memory = match self {
            Self::Disk { disk: _, memory } => memory,
            Self::Memory(memory) => memory,
        };
        let mut memory = memory.lock().expect(TRIE_MEMORY_MUST_HOLD_LOCK);
        HashDB::insert(memory.as_hash_db_mut(), prefix, value)
    }

    fn emplace(&mut self, key: H::Out, prefix: Prefix, value: DBValue) {
        let memory = match self {
            Self::Disk { disk: _, memory } => memory,
            Self::Memory(memory) => memory,
        };
        let mut memory = memory.lock().expect(TRIE_MEMORY_MUST_HOLD_LOCK);
        HashDB::emplace(memory.as_hash_db_mut(), key, prefix, value)
    }

    fn remove(&mut self, key: &H::Out, prefix: Prefix) {
        let memory = match self {
            Self::Disk { disk: _, memory } => memory,
            Self::Memory(memory) => memory,
        };
        let mut memory = memory.lock().expect(TRIE_MEMORY_MUST_HOLD_LOCK);
        HashDB::remove(memory.as_hash_db_mut(), key, prefix)
    }
}

impl<H> HashDBRef<H, DBValue> for TrieEssenceStorage<H>
where
    H: Hasher,
{
    fn get(&self, key: &H::Out, prefix: Prefix) -> Option<DBValue> {
        HashDB::get(self, key, prefix)
    }

    fn contains(&self, key: &H::Out, prefix: Prefix) -> bool {
        HashDB::contains(self, key, prefix)
    }
}

impl<H> AsHashDB<H, DBValue> for TrieEssenceStorage<H>
where
    H: Hasher,
{
    fn as_hash_db(&self) -> &dyn HashDB<H, DBValue> {
        self
    }

    fn as_hash_db_mut(&mut self) -> &mut dyn HashDB<H, DBValue> {
        self
    }
}

use crate::Inner;
use crate::{TRIE_MEMORY_MUST_HOLD_LOCK, TRIE_UNDERLYING_DB_MUST_HOLD_LOCK};
use hash_db::{AsHashDB, AsPlainDB, HashDB, HashDBRef, Hasher, PlainDB, PlainDBRef, Prefix};
use rusty_leveldb::OuterSnapshot;
use rusty_leveldb::Snapshot;
use rusty_leveldb::DB;
use sp_std::borrow::Borrow;
use sp_std::borrow::BorrowMut;
use sp_std::convert::AsRef;
use sp_std::sync::Arc;
use sp_std::sync::Mutex;
use sp_trie::DBValue;
use sp_trie::MemoryDB;

pub enum TrieSnapshot<H: Hasher> {
    Disk {
        db: Arc<Mutex<DB>>,
        snapshot: OuterSnapshot,
    },
    Memory(Arc<Mutex<MemoryDB<H>>>),
}

impl<H: Hasher> PlainDB<H::Out, DBValue> for TrieSnapshot<H>
where
    H::Out: Borrow<[u8]>,
    H::Out: for<'a> From<&'a [u8]>,
    H::Out: AsRef<[u8]>,
{
    fn get(&self, key: &H::Out) -> Option<DBValue> {
        match self {
            Self::Disk { db, snapshot } => {
                let mut db = db.lock().expect(TRIE_UNDERLYING_DB_MUST_HOLD_LOCK);
                if let Ok(item) = DB::get_at(
                    db.borrow_mut(),
                    &Snapshot::from(snapshot.clone()),
                    key.as_ref(),
                ) {
                    match item.map(Inner::<DBValue>::from) {
                        Some(Inner(value, rc)) if rc > 0 => Some(value),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            Self::Memory(memory) => {
                let memory = memory.lock().expect(TRIE_MEMORY_MUST_HOLD_LOCK);
                PlainDB::get(memory.as_plain_db(), key)
            }
        }
    }

    fn contains(&self, key: &H::Out) -> bool {
        match self {
            Self::Disk { db, snapshot } => {
                let mut db = db.lock().expect(TRIE_UNDERLYING_DB_MUST_HOLD_LOCK);
                if let Ok(item) = DB::get_at(
                    db.borrow_mut(),
                    &Snapshot::from(snapshot.clone()),
                    key.as_ref(),
                ) {
                    match item.map(Inner::<DBValue>::from) {
                        Some(Inner(_, rc)) if rc > 0 => true,
                        _ => false,
                    }
                } else {
                    false
                }
            }
            Self::Memory(memory) => {
                let memory = memory.lock().expect(TRIE_MEMORY_MUST_HOLD_LOCK);
                PlainDB::contains(memory.as_plain_db(), key)
            }
        }
    }

    fn emplace(&mut self, _key: H::Out, _value: DBValue) {
        unimplemented!("trie snapshot does not support the write methods")
    }

    fn remove(&mut self, _key: &H::Out) {
        unimplemented!("trie snapshot does not support the write methods")
    }
}

impl<H> PlainDBRef<H::Out, DBValue> for TrieSnapshot<H>
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

impl<H> AsPlainDB<H::Out, DBValue> for TrieSnapshot<H>
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

impl<H: Hasher> HashDB<H, DBValue> for TrieSnapshot<H>
where
    H::Out: AsRef<[u8]>,
{
    fn get(&self, key: &H::Out, prefix: Prefix) -> Option<DBValue> {
        match self {
            Self::Disk { db, snapshot } => {
                let mut db = db.lock().expect(TRIE_UNDERLYING_DB_MUST_HOLD_LOCK);
                if let Ok(item) = DB::get_at(
                    db.borrow_mut(),
                    &Snapshot::from(snapshot.clone()),
                    key.as_ref(),
                ) {
                    match item.map(Inner::<DBValue>::from) {
                        Some(Inner(value, rc)) if rc > 0 => Some(value),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            Self::Memory(memory) => {
                let memory = memory.lock().expect(TRIE_MEMORY_MUST_HOLD_LOCK);
                HashDB::get(memory.as_hash_db(), key, prefix)
            }
        }
    }

    fn contains(&self, key: &H::Out, prefix: Prefix) -> bool {
        match self {
            Self::Disk { db, snapshot } => {
                let mut db = db.lock().expect(TRIE_UNDERLYING_DB_MUST_HOLD_LOCK);
                if let Ok(item) = DB::get_at(
                    db.borrow_mut(),
                    &Snapshot::from(snapshot.clone()),
                    key.as_ref(),
                ) {
                    match item.map(Inner::<DBValue>::from) {
                        Some(Inner(_, rc)) if rc > 0 => true,
                        _ => false,
                    }
                } else {
                    false
                }
            }
            Self::Memory(memory) => {
                let memory = memory.lock().expect(TRIE_MEMORY_MUST_HOLD_LOCK);
                HashDB::contains(memory.as_hash_db(), key, prefix)
            }
        }
    }

    fn insert(&mut self, _prefix: Prefix, _value: &[u8]) -> H::Out {
        unimplemented!("trie snapshot does not support the write methods")
    }

    fn emplace(&mut self, _key: H::Out, _prefix: Prefix, _value: DBValue) {
        unimplemented!("trie snapshot does not support the write methods")
    }

    fn remove(&mut self, _key: &H::Out, _prefix: Prefix) {
        unimplemented!("trie snapshot does not support the write methods")
    }
}

impl<H> HashDBRef<H, DBValue> for TrieSnapshot<H>
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

impl<H> AsHashDB<H, DBValue> for TrieSnapshot<H>
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

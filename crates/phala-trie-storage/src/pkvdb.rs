use hash_db::{AsHashDB, AsPlainDB, HashDB, HashDBRef, Hasher, PlainDB, PlainDBRef, Prefix};
use sp_std::vec::Vec;

pub use sp_trie::MemoryDB as Transaction;
pub type StorageKey = Vec<u8>;
pub type StorageValue = Vec<u8>;
pub type StorageCollection = Vec<(StorageKey, Option<StorageValue>)>;
pub type ChildStorageCollection = Vec<(StorageKey, StorageCollection)>;

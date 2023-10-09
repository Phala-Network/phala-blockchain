use super::{CommitTransaction, Storage};
use crate::{capi::OCallImpl, types::Hashing};
use hash_db::Prefix;
pub use helper::code_exists;
use phala_trie_storage::BackendTransaction;
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
    fn get(&self, key: &Hash, _prefix: Prefix) -> Result<Option<DBValue>, DefaultError> {
        Ok(OCallImpl.storage_get(key.as_ref().to_vec()))
    }
}

impl CommitTransaction for ExternalBackend {
    fn commit_transaction(&mut self, root: Hash, mut transaction: BackendTransaction<Hashing>) {
        let changes = transaction
            .drain()
            .into_iter()
            .map(|(k, v)| (k[k.len() - 32..].to_vec(), v))
            .collect();
        OCallImpl.storage_commit(root, changes)
    }
}

impl ExternalStorage {
    pub fn instantiate() -> Self {
        let root = OCallImpl
            .storage_root()
            .unwrap_or_else(sp_trie::empty_trie_root::<sp_state_machine::LayoutV1<Hashing>>);
        let backend = TrieBackendBuilder::new(ExternalDB, root).build();
        crate::storage::Storage::new(backend)
    }
}

pub mod helper {
    use crate::types::Hash;
    use scale::Encode;
    use sp_core::hashing::twox_128;

    pub fn code_exists(code_hash: &Hash) -> bool {
        let key = code_owner_key(code_hash);
        super::ExternalStorage::instantiate().get(&key).is_some()
    }

    fn code_owner_key(code_hash: &Hash) -> Vec<u8> {
        let mut key = Vec::new();
        key.extend(twox_128("Contracts".as_bytes()));
        key.extend(twox_128("CodeInfoOf".as_bytes()));
        key.extend(&code_hash.encode());
        key
    }
}

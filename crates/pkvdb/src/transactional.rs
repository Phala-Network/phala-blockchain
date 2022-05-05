use crate::backend::PhalaTrieBackend;
use crate::backend::PhalaTrieStorage;
use hash_db::Hasher;
use parity_scale_codec::Codec;
use sp_state_machine::MemoryDB;
use sp_state_machine::TrieBackend;

pub trait TransactionalBackend<H: Hasher> {
    // NOTE: is not thread-safe so keep this method run with lock
    fn commit_transaction(&mut self, root: H::Out, transaction: MemoryDB<H>);
}

impl<H: Hasher> TransactionalBackend<H> for PhalaTrieBackend<H>
where
    H::Out: Codec + Ord,
{
    fn commit_transaction(&mut self, root: H::Out, transaction: MemoryDB<H>) {
        let storage = std::mem::take(self).0.into_storage();
        match &storage {
            PhalaTrieStorage::Essence(db) => {
                db.commit_transaction(transaction);
            }
            _ => unimplemented!("snapshot database is not supported commit transaction from pink"),
        }
        let backend = TrieBackend::new(storage, root);
        let _ = std::mem::replace(&mut self.0, backend);
    }
}

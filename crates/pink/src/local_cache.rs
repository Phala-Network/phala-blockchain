//! The LocalCache provides a local KV cache for contracts to do some offchain computation.
//! When we say local, it means that the data stored in the cache is different in different
//! machines of the same contract. And the data might loss when the pruntime restart or caused
//! by some kind of cache retirement machanism.

use chashmap::CHashMap;
use once_cell::sync::Lazy;
use alloc::borrow::Cow;

pub static GLOBAL_CACHE: Lazy<LocalCache> = Lazy::new(|| LocalCache::default());

// TODO.kevin: We may need a expiration time. Otherwise, the cache will explode finnally.
type Storage = CHashMap<Vec<u8>, Vec<u8>>;

#[derive(Default)]
pub struct LocalCache {
    storages: CHashMap<Vec<u8>, Storage>,
}

impl LocalCache {
    pub fn get(&self, id: &[u8], key: &[u8]) -> Option<Vec<u8>> {
        Some(self.storages.get(id)?.get(key)?.to_owned())
    }

    pub fn set(&self, id: Cow<[u8]>, key: Cow<[u8]>, value: Cow<[u8]>) {
        self.storages.alter(id.into(), move |store| {
            let store = match store {
                Some(store) => store,
                None => Default::default(),
            };
            store.insert(key.into(), value.into());
            Some(store)
        })
    }

    pub fn remove(&self, id: &[u8], key: &[u8]) -> Option<Vec<u8>> {
        self.storages.get_mut(id)?.remove(key)
    }

    #[allow(dead_code)]
    pub fn remove_storage(&self, id: &[u8]) {
        let _ = self.storages.remove(id);
    }
}

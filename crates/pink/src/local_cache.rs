//! The LocalCache provides a local KV cache for contracts to do some offchain computation.
//! When we say local, it means that the data stored in the cache is different in different
//! machines of the same contract. And the data might loss when the pruntime restart or caused
//! by some kind of cache expiring machanism.

use std::sync::atomic::{AtomicU64, Ordering};

use alloc::borrow::Cow;
use chashmap::CHashMap;
use once_cell::sync::Lazy;

pub static GLOBAL_CACHE: Lazy<LocalCache> = Lazy::new(|| LocalCache::default());

type Storage = CHashMap<Vec<u8>, StorageValue>;

struct StorageValue {
    /// Seconds since the UNIX epoch.
    expire_at: u64,
    value: Vec<u8>,
}

#[derive(Default)]
pub struct LocalCache {
    set_count: AtomicU64,
    storages: CHashMap<Vec<u8>, Storage>,
}

impl LocalCache {
    fn maybe_clear_expired(&self) {
        const GC_INTERVAL: u64 = 10000;
        let count = self.set_count.fetch_add(1, Ordering::Relaxed);
        if count == GC_INTERVAL {
            self.set_count.store(0, Ordering::Relaxed);
            let now = now();
            self.storages.retain(|_, store| {
                store.retain(|_, v| v.expire_at > now);
                true
            });
        }
    }

    pub fn get(&self, id: &[u8], key: &[u8]) -> Option<Vec<u8>> {
        Some(self.storages.get(id)?.get(key)?.value.to_owned())
    }

    pub fn set(&self, id: Cow<[u8]>, key: Cow<[u8]>, value: Cow<[u8]>) {
        self.maybe_clear_expired();
        self.storages.alter(id.into(), move |store| {
            let store = match store {
                Some(store) => store,
                None => Default::default(),
            };
            let value = StorageValue {
                expire_at: now().saturating_add(3600 * 24 * 7), // The default expire time is 7 days.
                value: value.into_owned(),
            };
            store.insert(key.into(), value);
            Some(store)
        })
    }

    pub fn set_expire(&self, id: Cow<[u8]>, key: Cow<[u8]>, expire: u64) {
        self.storages.alter(id.into(), move |store| {
            let store = match store {
                Some(store) => store,
                None => Default::default(),
            };
            store.alter(key.into(), |mut value| {
                if let Some(value) = &mut value {
                    value.expire_at = now().saturating_add(expire);
                }
                value
            });
            Some(store)
        })
    }

    pub fn remove(&self, id: &[u8], key: &[u8]) -> Option<Vec<u8>> {
        self.storages.get_mut(id)?.remove(key).map(|v| v.value)
    }

    #[allow(dead_code)]
    pub fn remove_storage(&self, id: &[u8]) {
        let _ = self.storages.remove(id);
    }
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("The system time is incorrect")
        .as_secs()
}

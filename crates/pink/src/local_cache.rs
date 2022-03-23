//! The LocalCache provides a local KV cache for contracts to do some offchain computation.
//! When we say local, it means that the data stored in the cache is different in different
//! machines of the same contract. And the data might loss when the pruntime restart or caused
//! by some kind of cache expiring machanism.

use alloc::borrow::Cow;
use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::RwLock, time::Instant};

pub use pink_extension::chain_extension::StorageQuotaExceeded;

pub static GLOBAL_CACHE: Lazy<RwLock<LocalCache>> = Lazy::new(Default::default);

#[derive(Default, Debug)]
struct Storage {
    // Sum of the size of all the keys and values.
    size: usize,
    kvs: HashMap<Vec<u8>, StorageValue>,
}

#[derive(Debug)]
struct StorageValue {
    // Expiration time in seconds since the first call to `now`.
    expire_at: u64,
    value: Vec<u8>,
}

#[derive(Debug)]
pub struct LocalCache {
    // Number of set ops between two GC ops.
    gc_interval: u64,
    // Accumulated number of set ops since last GC.
    sets_since_last_gc: u64,
    // Default expiration time in seconds.
    default_value_lifetime: u64,
    max_cache_size_per_contract: usize,
    storages: HashMap<Vec<u8>, Storage>,
}

impl Default for LocalCache {
    fn default() -> Self {
        Self {
            gc_interval: 1000,
            sets_since_last_gc: 0,
            default_value_lifetime: 3600 * 24 * 7, // 1 week
            max_cache_size_per_contract: 10 * 1024 * 1024, // 10MB
            storages: Default::default(),
        }
    }
}

impl LocalCache {
    fn maybe_clear_expired(&mut self) {
        self.sets_since_last_gc += 1;
        if self.sets_since_last_gc == self.gc_interval {
            self.sets_since_last_gc = 0;
            let now = now();
            self.storages.values_mut().for_each(|storage| {
                let storage_size = &mut storage.size;
                storage.kvs.retain(|k, v| {
                    if v.expire_at > now {
                        true
                    } else {
                        *storage_size -= v.value.len() + k.len();
                        false
                    }
                });
            });
        }
    }

    pub fn get(&self, id: &[u8], key: &[u8]) -> Option<Vec<u8>> {
        let entry = self.storages.get(id)?.kvs.get(key)?;
        if entry.expire_at <= now() {
            None
        } else {
            Some(entry.value.to_owned())
        }
    }

    #[cfg(test)]
    fn get_include_expired(&self, id: &[u8], key: &[u8]) -> Option<Vec<u8>> {
        Some(self.storages.get(id)?.kvs.get(key)?.value.to_owned())
    }

    pub fn set(
        &mut self,
        id: Cow<[u8]>,
        key: Cow<[u8]>,
        value: Cow<[u8]>,
    ) -> Result<(), StorageQuotaExceeded> {
        self.maybe_clear_expired();
        let store = self
            .storages
            .entry(id.into_owned())
            .or_insert_with(Storage::default);
        let key_len = key.len();
        let value_len = value.len();
        let prev_value = store.kvs.remove(key.as_ref());

        let new_size = match prev_value {
            Some(v) => store.size + value_len - v.value.len(),
            None => store.size + key_len + value_len,
        };

        if new_size > self.max_cache_size_per_contract {
            return Err(StorageQuotaExceeded);
        }

        store.size = new_size;
        store.kvs.insert(
            key.into_owned(),
            StorageValue {
                expire_at: now().saturating_add(self.default_value_lifetime),
                value: value.into_owned(),
            },
        );
        Ok(())
    }

    pub fn set_expire(&mut self, id: Cow<[u8]>, key: Cow<[u8]>, expire: u64) {
        self.maybe_clear_expired();
        if expire == 0 {
            let _ = self.remove(id.as_ref(), key.as_ref());
        } else {
            self.storages
                .get_mut(id.as_ref())
                .and_then(|storage| storage.kvs.get_mut(key.as_ref()))
                .map(|v| v.expire_at = now().saturating_add(expire));
        }
    }

    pub fn remove(&mut self, id: &[u8], key: &[u8]) -> Option<Vec<u8>> {
        self.maybe_clear_expired();
        let store = self.storages.get_mut(id)?;
        let v = store.kvs.remove(key).map(|v| v.value);
        if let Some(v) = &v {
            store.size -= v.len() + key.len();
        }
        v
    }

    #[allow(dead_code)]
    pub fn remove_storage(&mut self, id: &[u8]) {
        let _ = self.storages.remove(id);
    }
}

fn now() -> u64 {
    static REF_TIME: Lazy<Instant> = Lazy::new(Instant::now);
    REF_TIME.elapsed().as_secs()
}

#[cfg(test)]
mod test {
    use super::*;
    fn test_cache() -> LocalCache {
        LocalCache {
            gc_interval: 2,
            sets_since_last_gc: 0,
            default_value_lifetime: 2,
            max_cache_size_per_contract: 1024,
            storages: Default::default(),
        }
    }

    fn cow<'a>(s: &'a impl AsRef<[u8]>) -> Cow<'a, [u8]> {
        Cow::Borrowed(s.as_ref())
    }

    fn gc(cache: &mut LocalCache) {
        for _ in 0..cache.gc_interval + 1 {
            let _ = cache.set(cow(b"_"), cow(b"_"), cow(b"_"));
        }
    }

    fn sleep(secs: u64) {
        std::thread::sleep(std::time::Duration::from_secs(secs));
    }

    fn get_size(cache: &LocalCache, id: &[u8]) -> usize {
        cache.storages.get(id).unwrap().size
    }

    #[test]
    fn default_expire_should_work() {
        let mut cache = test_cache();
        let _ = cache.set(cow(b"id"), cow(b"foo"), cow(b"value"));
        assert_eq!(cache.get(b"id", b"foo"), Some(b"value".to_vec()));

        sleep(cache.default_value_lifetime);
        assert_eq!(cache.get(b"id", b"foo"), None);
        assert!(cache.get_include_expired(b"id", b"foo").is_some());
        gc(&mut cache);
        assert_eq!(cache.get_include_expired(b"id", b"foo"), None);
        assert_eq!(get_size(&cache, b"id"), 0);
    }

    #[test]
    fn set_expire_should_work() {
        let mut cache = test_cache();
        let _ = cache.set(cow(b"id"), cow(b"foo"), cow(b"value"));
        assert_eq!(cache.get(b"id", b"foo"), Some(b"value".to_vec()));
        cache.set_expire(cow(b"id"), cow(b"foo"), cache.default_value_lifetime + 2);

        sleep(cache.default_value_lifetime);
        gc(&mut cache);

        assert_eq!(cache.get(b"id", b"foo"), Some(b"value".to_vec()));

        sleep(2);
        gc(&mut cache);

        assert_eq!(cache.get_include_expired(b"id", b"foo"), None);
    }

    #[test]
    fn size_limit_should_work() {
        let mut cache = test_cache();
        cache.max_cache_size_per_contract = 10;
        assert!(cache.set(cow(b"id"), cow(b"foo"), cow(b"value")).is_ok());
        assert!(cache.set(cow(b"id"), cow(b"bar"), cow(b"value")).is_err());
    }

    #[test]
    fn size_calc() {
        let mut cache = test_cache();
        assert!(cache.set(cow(b"id"), cow(b"foo"), cow(b"bar")).is_ok());
        assert_eq!(get_size(&cache, b"id"), 6);
        assert!(cache.set(cow(b"id"), cow(b"foo"), cow(b"foobar")).is_ok());
        assert_eq!(get_size(&cache, b"id"), 9);
        assert!(cache.set(cow(b"id"), cow(b"foo"), cow(b"foo")).is_ok());
        assert_eq!(get_size(&cache, b"id"), 6);
        assert!(cache.remove(b"id", b"foo").is_some());
        assert_eq!(get_size(&cache, b"id"), 0);
    }
}

//! The LocalCache provides a local KV cache for contracts to do some offchain computation.
//! When we say local, it means that the data stored in the cache is different in different
//! machines of the same contract. And the data might loss when the pruntime restart or caused
//! by some kind of cache expiring machanism.

use once_cell::sync::Lazy;
use pink_extension::CacheOp;
use sp_core::crypto::AccountId32;
use std::{
    borrow::Cow,
    collections::BTreeMap,
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};

pub use pink_extension::chain_extension::StorageQuotaExceeded;

static TEST_MODE: AtomicBool = AtomicBool::new(false);

pub(crate) fn enable_test_mode() {
    TEST_MODE.store(true, Ordering::Relaxed);
}

fn with_global_cache<T>(f: impl FnOnce(&mut LocalCache) -> T) -> T {
    if TEST_MODE.load(Ordering::Relaxed) {
        // Unittests are running in multi-threaded env. Let's give per test case a cache instance.
        use std::cell::RefCell;
        thread_local! {
            pub static GLOBAL_CACHE: RefCell<LocalCache> = RefCell::new(LocalCache::new());
        }
        GLOBAL_CACHE.with(move |cache| f(&mut cache.borrow_mut()))
    } else {
        use std::sync::Mutex;
        pub static GLOBAL_CACHE: Mutex<LocalCache> = Mutex::new(LocalCache::new());
        f(&mut GLOBAL_CACHE.lock().unwrap())
    }
}

struct Storage {
    // Sum of the size of all the keys and values.
    size: usize,
    max_size: usize,
    kvs: BTreeMap<Vec<u8>, StorageValue>,
}

impl Storage {
    fn new(max_size: usize) -> Self {
        Self {
            size: 0,
            max_size,
            kvs: Default::default(),
        }
    }

    fn fit_size(&mut self) {
        if self.size <= self.max_size {
            return;
        }
        let map = std::mem::take(&mut self.kvs);

        let mut kvs: Vec<_> = map
            .into_iter()
            .map(|(k, v)| (v.expire_at, (k, v)))
            .collect();
        kvs.sort_by_key(|(expire, _)| *expire);
        self.kvs = kvs
            .into_iter()
            .filter_map(|(_, (k, v))| {
                if self.size < self.max_size {
                    return Some((k, v));
                }
                self.size -= k.len() + v.value.len();
                None
            })
            .collect();
    }

    fn clear_expired(&mut self, now: u64) {
        self.kvs.retain(|k, v| {
            if v.expire_at > now {
                true
            } else {
                self.size -= v.value.len() + k.len();
                false
            }
        });
    }

    fn remove(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        let v = self.kvs.remove(key).map(|v| v.value);
        if let Some(v) = &v {
            self.size -= v.len() + key.len();
        }
        v
    }

    fn set(
        &mut self,
        key: Cow<[u8]>,
        value: Cow<[u8]>,
        lifetime: u64,
    ) -> Result<(), StorageQuotaExceeded> {
        _ = self.remove(key.as_ref());
        let data_len = key.len() + value.len();
        let mut store_size = self.size + data_len;
        if store_size > self.max_size {
            self.clear_expired(now());
            store_size = self.size + data_len;
            if store_size > self.max_size {
                return Err(StorageQuotaExceeded);
            }
        }
        self.size = store_size;
        self.kvs.insert(
            key.into_owned(),
            StorageValue {
                expire_at: now().saturating_add(lifetime),
                value: value.into_owned(),
            },
        );
        Ok(())
    }

    fn get(&self, key: &[u8]) -> Option<&StorageValue> {
        self.kvs.get(key)
    }
}

struct StorageValue {
    // Expiration time in seconds since the first call to `now`.
    expire_at: u64,
    value: Vec<u8>,
}

pub struct LocalCache {
    // Number of set ops between two GC ops.
    gc_interval: u64,
    // Accumulated number of set ops since last GC.
    sets_since_last_gc: u64,
    // Default expiration time in seconds.
    default_value_lifetime: u64,
    storages: BTreeMap<Vec<u8>, Storage>,
}

impl LocalCache {
    const fn new() -> Self {
        Self {
            gc_interval: 1000,
            sets_since_last_gc: 0,
            default_value_lifetime: 3600 * 24 * 7, // 1 week
            storages: BTreeMap::new(),
        }
    }
}

impl LocalCache {
    fn maybe_clear_expired(&mut self) {
        self.sets_since_last_gc += 1;
        if self.sets_since_last_gc == self.gc_interval {
            self.clear_expired();
        }
    }

    fn clear_expired(&mut self) {
        self.sets_since_last_gc = 0;
        let now = now();
        self.storages.values_mut().for_each(|storage| {
            storage.clear_expired(now);
        });
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
        self.storages
            .get_mut(id.as_ref())
            .ok_or(StorageQuotaExceeded)?
            .set(key, value, self.default_value_lifetime)
    }

    pub fn set_expire(&mut self, id: Cow<[u8]>, key: Cow<[u8]>, expire: u64) {
        self.maybe_clear_expired();
        if expire == 0 {
            let _ = self.remove(id.as_ref(), key.as_ref());
        } else if let Some(v) = self
            .storages
            .get_mut(id.as_ref())
            .and_then(|storage| storage.kvs.get_mut(key.as_ref()))
        {
            v.expire_at = now().saturating_add(expire)
        }
    }

    pub fn remove(&mut self, id: &[u8], key: &[u8]) -> Option<Vec<u8>> {
        self.maybe_clear_expired();
        let store = self.storages.get_mut(id)?;
        store.remove(key)
    }

    #[allow(dead_code)]
    pub fn remove_storage(&mut self, id: &[u8]) {
        let _ = self.storages.remove(id);
    }

    pub fn apply_quotas<'a>(&mut self, quotas: impl IntoIterator<Item = (&'a [u8], usize)>) {
        for (contract, max_size) in quotas.into_iter() {
            log::trace!(
                "Applying cache quotas for {} max_size={max_size}",
                hex_fmt::HexFmt(contract)
            );
            if max_size == 0 {
                self.storages.remove(contract);
                continue;
            }
            match self.storages.get_mut(contract) {
                Some(store) => {
                    store.max_size = max_size;
                    store.fit_size();
                }
                None => {
                    self.storages
                        .insert(contract.to_vec(), Storage::new(max_size));
                }
            }
        }
    }
}

fn now() -> u64 {
    static REF_TIME: Lazy<Instant> = Lazy::new(Instant::now);
    REF_TIME.elapsed().as_secs()
}

pub fn apply_cache_op(contract: &AccountId32, op: CacheOp) {
    with_global_cache(|cache| {
        let contract: &[u8] = contract.as_ref();
        match op {
            CacheOp::Set { key, value } => {
                let _ = cache.set(contract.into(), key.into(), value.into());
            }
            CacheOp::SetExpiration { key, expiration } => {
                cache.set_expire(contract.into(), key.into(), expiration)
            }
            CacheOp::Remove { key } => {
                let _ = cache.remove(contract, &key);
            }
        }
    })
}

pub fn set(contract: &[u8], key: &[u8], value: &[u8]) -> Result<(), StorageQuotaExceeded> {
    with_global_cache(|cache| cache.set(contract.into(), key.into(), value.into()))
}

pub fn get(contract: &[u8], key: &[u8]) -> Option<Vec<u8>> {
    with_global_cache(|cache| cache.get(contract, key))
}

pub fn set_expiration(contract: &[u8], key: &[u8], expiration: u64) {
    with_global_cache(|cache| cache.set_expire(contract.into(), key.into(), expiration))
}

pub fn remove(contract: &[u8], key: &[u8]) -> Option<Vec<u8>> {
    with_global_cache(|cache| cache.remove(contract, key))
}

pub fn apply_quotas<'a>(quotas: impl IntoIterator<Item = (&'a [u8], usize)>) {
    with_global_cache(|cache| cache.apply_quotas(quotas))
}

#[cfg(test)]
mod test {
    use super::*;
    fn test_cache() -> LocalCache {
        LocalCache {
            gc_interval: 2,
            sets_since_last_gc: 0,
            default_value_lifetime: 2,
            storages: Default::default(),
        }
    }

    fn cow(s: &impl AsRef<[u8]>) -> Cow<[u8]> {
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
        cache.apply_quotas([(&b"id"[..], 1000)]);
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
        cache.apply_quotas([(&b"id"[..], 1000)]);

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
        cache.apply_quotas([(&b"id"[..], 10)]);

        assert!(cache.set(cow(b"id"), cow(b"foo"), cow(b"value")).is_ok());
        assert!(cache.set(cow(b"id"), cow(b"bar"), cow(b"value")).is_err());
    }

    #[test]
    fn size_calc() {
        let mut cache = test_cache();
        cache.apply_quotas([(&b"id"[..], 100)]);

        assert!(cache.set(cow(b"id"), cow(b"foo"), cow(b"bar")).is_ok());
        assert_eq!(get_size(&cache, b"id"), 6);
        assert!(cache.set(cow(b"id"), cow(b"foo"), cow(b"foobar")).is_ok());
        assert_eq!(get_size(&cache, b"id"), 9);
        assert!(cache.set(cow(b"id"), cow(b"foo"), cow(b"foo")).is_ok());
        assert_eq!(get_size(&cache, b"id"), 6);
        assert!(cache.remove(b"id", b"foo").is_some());
        assert_eq!(get_size(&cache, b"id"), 0);
    }

    #[test]
    fn fit_size_works() {
        let mut store = Storage::new(20);
        assert!(store.set(cow(b"k0"), cow(b"v0"), 1000).is_ok());
        assert_eq!(store.size, 4);
        assert!(store.set(cow(b"k1"), cow(b"v0"), 50).is_ok());
        assert_eq!(store.size, 8);
        assert!(store.set(cow(b"k2"), cow(b"v0"), 200).is_ok());
        assert_eq!(store.size, 12);
        assert!(store.set(cow(b"k3"), cow(b"v0"), 100).is_ok());
        assert_eq!(store.size, 16);
        assert!(store.set(cow(b"k4"), cow(b"v"), 100).is_ok());
        assert_eq!(store.size, 19);
        assert!(store.set(cow(b"k4"), cow(b"vvvvv"), 100).is_err());
        assert_eq!(store.size, 16);

        assert!(store.get(b"k0").is_some());
        assert!(store.get(b"k1").is_some());
        assert!(store.get(b"k2").is_some());
        assert!(store.get(b"k3").is_some());

        store.max_size = 10;
        store.fit_size();

        assert!(store.get(b"k0").is_some());
        assert!(store.get(b"k2").is_some());

        assert!(store.get(b"k1").is_none());
        assert!(store.get(b"k3").is_none());
        assert_eq!(store.size, 8);
    }
}

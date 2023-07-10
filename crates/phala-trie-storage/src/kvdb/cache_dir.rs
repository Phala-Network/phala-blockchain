//! The `cache_dir` is employed to store the trie db.
//! In gramine execution, the PHACTORY_TRIE_CACHE_PATH is hardcoded into the manifest.
//! For unittest execution, `test_cached_path` is set to a temporary directory.
//! In non-SGX test pruntime execution, the default cache directory is used.
//!
//! The `test_cached_path` represents a global cache directory specifically for unit tests.
//!
//! The `with` function, primarily used within unit tests, takes a cache directory
//! and a function as parameters. It temporarily switches the cache directory to
//! the specified one, executes the supplied function, and then restores the
//! original cache directory.
//!
//! The `get` function retrieves the path of the cache directory. It first attempts to
//! fetch the path from the `test_cached_path` variable. If that fails,
//! it turns to the `PHACTORY_TRIE_CACHE_PATH` environment variable. In the event of
//! both attempts failing, it defaults to "data/protected_files/caches".

// Global cache directory for unit tests.
environmental::environmental!(test_cached_path: String);

#[cfg(test)]
pub(crate) fn with<T>(cache_dir: &str, f: impl FnOnce() -> T) -> T {
    let mut cache_dir = cache_dir.to_string();
    test_cached_path::using(&mut cache_dir, f)
}

pub(crate) fn get() -> String {
    let test_path = test_cached_path::with(|cache_dir| cache_dir.clone());
    test_path
        .or_else(|| immutable_env::get("PHACTORY_TRIE_CACHE_PATH"))
        .unwrap_or_else(|| "data/protected_files/caches".to_string())
}

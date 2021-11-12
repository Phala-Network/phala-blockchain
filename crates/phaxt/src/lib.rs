#[subxt::subxt(runtime_metadata_path = "metadata_files/khala_metadata.scale")]
pub mod khala {
    #[subxt(substitute_type = "phala_mq::types::SignedMessage")]
    pub use phala_types::messaging::SignedMessage;
}
#[subxt::subxt(runtime_metadata_path = "metadata_files/kusama_metadata.scale")]
pub mod kusama {}

mod workaround;

pub mod extra;
pub mod rpc;
pub mod khala_config;
pub use khala_config::*;

pub type ParachainApi = khala::RuntimeApi<khala_config::KhalaConfig>;
pub type RelaychainApi = kusama::RuntimeApi<kusama::DefaultConfig>;

pub use subxt;
pub use subxt::sp_core::storage::{StorageData, StorageKey};

pub fn storage_key<T: subxt::StorageEntry>(entry: T) -> StorageKey {
    let prefix = subxt::storage::StorageKeyPrefix::new::<T>();
    entry.key().final_key(prefix)
}

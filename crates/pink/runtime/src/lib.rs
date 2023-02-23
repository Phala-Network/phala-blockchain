extern crate alloc;

mod contract;
mod export_fixtures;

pub mod runtime;
pub mod storage;

pub mod types;

// todo!: remove this
pub use export_fixtures::load_test_wasm;
pub use pink_extension_runtime::local_cache;
pub use storage::InMemoryStorage as Storage;

pub use frame_support::weights;
pub mod capi;


mod async_context;
mod env;
mod resource;
mod run;
pub mod service;

pub type VmId = [u8; 32];
pub use run::WasmRun;

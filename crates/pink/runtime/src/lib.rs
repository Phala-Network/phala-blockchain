extern crate alloc;

pub mod capi;
mod contract;
mod runtime;
mod storage;
pub mod types;

pub use crate::contract::{ContractResult, ContractInstantiateResult, ContractExecResult};

/// Returns a tuple containing the major and minor version numbers of the current crate.
///
/// # Examples
///
/// ```
/// let (major, minor) = pink::version();
/// println!("Current version: {}.{}", major, minor);
/// ```
pub fn version() -> (u32, u32) {
    let major = env!("CARGO_PKG_VERSION_MAJOR")
        .parse()
        .expect("Invalid major version");

    let minor = env!("CARGO_PKG_VERSION_MINOR")
        .parse()
        .expect("Invalid minor version");

    (major, minor)
}

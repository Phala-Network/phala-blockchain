//! # Glossary
//!
//! - **pruntime**: A program within the TEE that monitors blocks from the Phala blockchain and processes onchain
//!   instructions. This is where the Pink runtime runs.
//!
//! - **Cluster**: Refers to a group of workers executing identical contracts while maintaining consistent state.
//!
//! - **Contract call nonce**: A unique identifier for each contract call, enabling external tools to link events
//!   back to their original call.
//!
//! - **ecall (Enterwards Call)**: The terms `ecall` and `ocall` are borrowed from Intel's SGX SDK, where `e` and
//!   `o` indicate the direction of the call across module boundaries. An `ecall` here is a call from outside the
//!   `libpink.so` into it, used to invoke operations in the Pink runtime, such as processing transactions or queries.
//!
//! - **ocall (Outwards Call)**: A call from within the `libpink.so` to the outside environment. `ocalls` are used
//!   by the Pink runtime to interact with external resources or perform operations outside of the Pink runtime
//!   environment, such as making HTTP requests or running JavaScript.
//!
//! # Pink Runtime Library
//!
//! This crate contains the core functionality of the Pink runtime, which powers the ink contract at Phala.
//! It is compiled into dynamic libraries and included with the `pruntime` binary. In each `pruntime` release,
//! there are several versions of `libpink.so`. The `pruntime` would choose the right `libpink.so` version
//! when it starts, based on instructions from the Phala blockchain. When transactions or queries are sent to
//! the `pruntime`, it would invoke the `libpink.so` by calling ecalls to serve the requests.
//!
//! # Design Overview of the Pink Runtime
//!
//! The Pink runtime is built on top the `pallet-contracts` framework, extended with some chain extensions.
//! These extensions introduce Phala-exclusive features, such as enabling HTTP requests in query operations.
//!
//! The Pink runtime itself is stateless. When the host invokes the Pink runtime via `ecalls`, it sets the
//! state in thread-local storage. Consequently, the Pink runtime accesses this state through `ocalls`.
//!
//! The entire `pruntime` process operates within a Trusted Execution Environment (TEE). It ensures that
//! contract states, which are private to `pruntime`, remain confidential and secure. The pink runtime must
//! not leak any sensitive information to the outside world.
//!
//! Lets take a contract query as an example. The overall flow is as follows:
//! ![Query Flow](../assets/query-flow.png)

extern crate alloc;

pub mod capi;
mod contract;
mod runtime;
mod storage;
pub mod types;

pub use crate::contract::{ContractExecResult, ContractInstantiateResult, ContractResult};

/// Returns a tuple containing the major and minor version numbers of the current crate.
///
/// # Examples
///
/// ```
/// let (major, minor) = pink_runtime::version();
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

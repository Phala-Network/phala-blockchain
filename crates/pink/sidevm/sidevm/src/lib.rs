//! This crate provides some instrumentation to write sidevm programs. It is built on top of the
//! SideVM ocalls.

#![warn(missing_docs)]

pub use env::ocall_funcs_guest as ocall;
pub use pink_sidevm_env as env;
pub use pink_sidevm_logger as logger;
pub use pink_sidevm_macro::main;
pub use res_id::ResourceId;

pub mod channel;
pub mod time;

mod res_id;

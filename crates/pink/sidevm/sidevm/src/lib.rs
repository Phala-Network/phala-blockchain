//! This crate provides some instrumentation to write sidevm programs. It is built on top of the
//! Sidevm ocalls.

#![warn(missing_docs)]

pub use env::ocall_funcs_guest as ocall;
pub use pink_sidevm_env as env;
pub use pink_sidevm_logger as logger;
pub use pink_sidevm_macro::main;
pub use res_id::ResourceId;

pub use env::spawn;
pub use env::tasks as task;

pub mod channel;
pub mod net;
pub mod time;
pub mod exec;

mod res_id;

use super::*;
use crate::args_stack::{I32Convertible, RetDecode, RetEncode, StackedArgs};

#[pink_sidevm_macro::ocall]
pub trait OcallFuncs {
    #[ocall(id = 100)]
    fn echo(input: Vec<u8>) -> Result<Vec<u8>>;

    #[ocall(id = 101, fast_input, fast_return)]
    fn close(resource_id: i32) -> Result<()>;

    #[ocall(id = 102, fast_input, fast_return)]
    fn poll(resource_id: i32) -> Result<()>;

    #[ocall(id = 103, fast_input, fast_return)]
    fn next_ready_task() -> Result<i32>;

    #[ocall(id = 201, fast_input, fast_return)]
    fn create_timer(timeout: i32) -> Result<i32>;

    #[ocall(id = 202, fast_return)]
    fn set_log_level(log_level: LogLevel) -> Result<()>;
}

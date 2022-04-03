use super::*;

#[pink_sidevm_macro::ocall]
pub trait OcallFuncs {
    #[ocall(id = 100)]
    fn echo(input: Vec<u8>) -> Vec<u8>;

    #[ocall(id = 101, fast_input, fast_return)]
    fn close(resource_id: i32) -> i32;

    #[ocall(id = 102, fast_input, fast_return)]
    fn poll(resource_id: i32) -> i32;

    #[ocall(id = 103, fast_input, fast_return)]
    fn next_ready_task() -> i32;

    #[ocall(id = 201, fast_input, fast_return)]
    fn create_timer(timeout: i32) -> i32;

    #[ocall(id = 202, fast_return)]
    fn set_log_level(log_level: LogLevel) -> i32;
}

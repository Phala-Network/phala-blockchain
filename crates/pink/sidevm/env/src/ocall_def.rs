use super::*;
use crate::args_stack::{I32Convertible, RetDecode, StackedArgs};

#[derive(Debug, Clone, Copy)]
pub enum PollState {
    Pending,
    Ready,
}

impl I32Convertible for PollState {
    fn to_i32(self) -> i32 {
        match self {
            PollState::Ready => 0,
            PollState::Pending => 1,
        }
    }

    fn from_i32(i: i32) -> Self {
        match i {
            0 => PollState::Ready,
            1 => PollState::Pending,
            _ => panic!("Invalid PollResult value: {}", i),
        }
    }
}

#[pink_sidevm_macro::ocall]
pub trait OcallFuncs {
    #[ocall(id = 100)]
    fn echo(input: Vec<u8>) -> Result<Vec<u8>>;

    #[ocall(id = 101, fast_input, fast_return)]
    fn close(resource_id: i32) -> Result<()>;

    #[ocall(id = 102, fast_input, fast_return)]
    fn poll_read(resource_id: i32, data: &mut [u8]) -> Result<PollState>;

    #[ocall(id = 103, fast_input, fast_return)]
    fn poll_write(resource_id: i32, data: &[u8]) -> Result<PollState>;

    #[ocall(id = 104, fast_input, fast_return)]
    fn next_ready_task() -> Result<i32>;

    #[ocall(id = 105, fast_return)]
    fn enable_ocall_trace(enable: bool) -> Result<()>;

    #[ocall(id = 106, fast_return)]
    fn set_log_level(log_level: LogLevel) -> Result<()>;

    #[ocall(id = 201, fast_input, fast_return)]
    fn create_timer(timeout: i32) -> Result<i32>;
}

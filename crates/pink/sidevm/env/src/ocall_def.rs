use super::*;
use crate::args_stack::{I32Convertible, RetDecode, StackedArgs};

/// Poll state for a dynamic returned buffer (for the ocall `fn poll`).
#[derive(Encode, Decode)]
pub enum Poll<T> {
    /// Represents that a value is not ready yet.
    Pending,
    /// Represents that a value is immediately ready.
    Ready(T),
}

// Poll state for poll_read/poll_write.
impl I32Convertible for Poll<u32> {
    fn to_i32(self) -> i32 {
        match self {
            // In theory, the n could be u32::MAX which is the same binary representation as -1.
            // But in practice, no buffer in wasm32 can have a size of u32::MAX or greater.
            Self::Ready(n) => n as i32,
            Self::Pending => -1,
        }
    }

    fn from_i32(i: i32) -> Self {
        match i {
            -1 => Self::Pending,
            n => Self::Ready(n as u32),
        }
    }
}

/// All ocall definitions for pink SideVM.
#[pink_sidevm_macro::ocall]
pub trait OcallFuncs {
    /// Close given resource by id.
    #[ocall(id = 101, fast_input, fast_return)]
    fn close(resource_id: i32) -> Result<()>;

    /// Poll given resource by id and return a dynamic sized data.
    #[ocall(id = 102, fast_input)]
    fn poll(resource_id: i32) -> Result<Poll<Vec<u8>>>;

    /// Poll given resource to read data. Low level support for AsyncRead.
    #[ocall(id = 103, fast_input, fast_return)]
    fn poll_read(resource_id: i32, data: &mut [u8]) -> Result<Poll<u32>>;

    /// Poll given resource to write data. Low level support for AsyncWrite.
    #[ocall(id = 104, fast_input, fast_return)]
    fn poll_write(resource_id: i32, data: &[u8]) -> Result<Poll<u32>>;

    /// Get the next waken up task id.
    #[ocall(id = 110, fast_input, fast_return)]
    fn next_ready_task() -> Result<i32>;

    /// Enable logging for ocalls
    #[ocall(id = 111, fast_return)]
    fn enable_ocall_trace(enable: bool) -> Result<()>;

    /// Set log level
    #[ocall(id = 112, fast_return)]
    fn set_log_level(log_level: LogLevel) -> Result<()>;

    /// Create a timer given a duration of time in milliseconds.
    #[ocall(id = 201, fast_input, fast_return)]
    fn create_timer(timeout: i32) -> Result<i32>;
}

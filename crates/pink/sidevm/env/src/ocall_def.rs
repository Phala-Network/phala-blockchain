use super::*;
use crate::args_stack::{I32Convertible, RetDecode, StackedArgs};

/// All ocall definitions for pink Sidevm.
#[pink_sidevm_macro::ocall]
pub trait OcallFuncs {
    /// Close given resource by id.
    #[ocall(id = 101)]
    fn close(resource_id: i32) -> Result<()>;

    /// Poll given resource by id and return a dynamic sized data.
    #[ocall(id = 102, encode_output)]
    fn poll(resource_id: i32) -> Result<Vec<u8>>;

    /// Poll given resource to read data. Low level support for AsyncRead.
    #[ocall(id = 103)]
    fn poll_read(resource_id: i32, data: &mut [u8]) -> Result<u32>;

    /// Poll given resource to write data. Low level support for AsyncWrite.
    #[ocall(id = 104)]
    fn poll_write(resource_id: i32, data: &[u8]) -> Result<u32>;

    /// Shutdown a socket
    #[ocall(id = 105)]
    fn poll_shutdown(resource_id: i32) -> Result<()>;

    /// Poll given resource to generate a new resource id.
    #[ocall(id = 106)]
    fn poll_res(resource_id: i32) -> Result<i32>;

    /// Mark a task as ready for next polling
    #[ocall(id = 109)]
    fn mark_task_ready(task_id: i32) -> Result<()>;

    /// Get the next waken up task id.
    #[ocall(id = 110)]
    fn next_ready_task() -> Result<i32>;

    /// Enable logging for ocalls
    #[ocall(id = 111)]
    fn enable_ocall_trace(enable: bool) -> Result<()>;

    /// Create a timer given a duration of time in milliseconds.
    #[ocall(id = 201)]
    fn create_timer(timeout: i32) -> Result<i32>;

    /// Create a TCP socket, bind to given address and listen to incoming connections.
    ///
    /// The backlog argument defines the maximum length to which the queue of pending connections
    /// for sockfd may grow.
    ///
    /// Invoke tcp_accept on the returned resource_id to accept incoming connections.
    #[ocall(id = 210)]
    fn tcp_listen(addr: &str, backlog: i32) -> Result<i32>;

    /// Accept incoming TCP connections.
    #[ocall(id = 211)]
    fn tcp_accept(resource_id: i32) -> Result<i32>;

    /// Initiate a TCP connection to a remote endpoint.
    #[ocall(id = 212)]
    fn tcp_connect(&mut self, addr: &str) -> Result<i32>;

    /// Print log message.
    #[ocall(id = 220)]
    fn log(level: log::Level, message: &str) -> Result<()>;

    /// Get value from the local cache.
    #[ocall(id = 230, encode_output)]
    fn local_cache_get(key: &[u8]) -> Result<Option<Vec<u8>>>;

    /// Set value to the local cache.
    #[ocall(id = 231)]
    fn local_cache_set(key: &[u8], value: &[u8]) -> Result<()>;

    /// Set expiration time for a key in the local cache.
    #[ocall(id = 232)]
    fn local_cache_set_expiration(key: &[u8], expire_after_secs: u64) -> Result<()>;

    /// Remove a value from the local cache.
    ///
    /// Returns the previous value if it existed.
    #[ocall(id = 233, encode_output)]
    fn local_cache_remove(key: &[u8]) -> Result<Option<Vec<u8>>>;
}

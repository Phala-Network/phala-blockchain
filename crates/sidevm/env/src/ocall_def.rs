use super::*;
use crate::args_stack::{I32Convertible, RetDecode, StackedArgs};
use crate::tls::{TlsClientConfig, TlsServerConfig};
use std::borrow::Cow;

/// All ocall definitions for pink Sidevm.
#[sidevm_macro::ocall]
pub trait OcallFuncs {
    /// Close given resource by id.
    #[ocall(id = 101)]
    fn close(resource_id: i32) -> Result<()>;

    /// Poll given resource by id and return a dynamic sized data.
    #[ocall(id = 102, encode_output)]
    fn poll(waker_id: i32, resource_id: i32) -> Result<Vec<u8>>;

    /// Poll given resource to read data. Low level support for AsyncRead.
    #[ocall(id = 103)]
    fn poll_read(waker_id: i32, resource_id: i32, data: &mut [u8]) -> Result<u32>;

    /// Poll given resource to write data. Low level support for AsyncWrite.
    #[ocall(id = 104)]
    fn poll_write(waker_id: i32, resource_id: i32, data: &[u8]) -> Result<u32>;

    /// Shutdown a socket
    #[ocall(id = 105)]
    fn poll_shutdown(waker_id: i32, resource_id: i32) -> Result<()>;

    /// Poll given resource to generate a new resource id.
    #[ocall(id = 106)]
    fn poll_res(waker_id: i32, resource_id: i32) -> Result<i32>;

    /// Mark a task as ready for next polling
    #[ocall(id = 109)]
    fn mark_task_ready(task_id: i32) -> Result<()>;

    /// Get the next waken up task id.
    #[ocall(id = 110)]
    fn next_ready_task() -> Result<i32>;

    /// Enable logging for ocalls
    #[ocall(id = 111)]
    fn enable_ocall_trace(enable: bool) -> Result<()>;

    /// Get awake wakers
    #[ocall(id = 112, encode_output)]
    fn awake_wakers() -> Result<Vec<i32>>;

    /// Get random number
    #[ocall(id = 113)]
    fn getrandom(buf: &mut [u8]) -> Result<()>;

    /// Create a timer given a duration of time in milliseconds.
    #[ocall(id = 201)]
    fn create_timer(timeout: i32) -> Result<i32>;

    /// Send data to a oneshot channel.
    #[ocall(id = 202)]
    fn oneshot_send(resource_id: i32, data: &[u8]) -> Result<()>;

    /// Percentage of the gas remaining to the next breath
    #[ocall(id = 203)]
    fn gas_remaining() -> Result<u8>;

    /// Create a TCP socket, bind to given address and listen to incoming connections.
    ///
    /// If `tls_config` is not `None`, then the socket will be TLS encrypted.
    /// Invoke tcp_accept on the returned resource_id to accept incoming connections.
    #[ocall(id = 210, encode_input)]
    fn tcp_listen(addr: Cow<str>, tls_config: Option<TlsServerConfig>) -> Result<i32>;

    /// Accept incoming TCP connections.
    #[ocall(id = 211, encode_output)]
    fn tcp_accept(waker_id: i32, resource_id: i32) -> Result<(i32, String)>;

    /// Accept incoming TCP connections without returning the remote address.
    #[ocall(id = 212)]
    fn tcp_accept_no_addr(waker_id: i32, resource_id: i32) -> Result<i32>;

    /// Initiate a TCP connection to a remote endpoint.
    #[ocall(id = 213)]
    fn tcp_connect(host: &str, port: u16) -> Result<i32>;

    /// Initiate a TLS/TCP connection to a remote endpoint.
    #[ocall(id = 214, encode_input)]
    fn tcp_connect_tls(host: String, port: u16, config: TlsClientConfig) -> Result<i32>;

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

    /// Create input channel
    #[ocall(id = 240, encode_output)]
    fn create_input_channel(ch: InputChannel) -> Result<i32>;

    /// Query a contract
    ///
    /// Returns a channel id for the query result.
    ///
    /// # Limitation
    /// Only one query can be processed at a time.
    #[ocall(id = 241, encode_input)]
    fn query_local_contract(contract_id: [u8; 32], payload: Vec<u8>) -> Result<i32>;

    /// Returns the vmid of the current instance.
    #[ocall(id = 242, encode_output)]
    fn vmid() -> Result<[u8; 32]>;
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputChannel {
    /// Input channel for system messages such as receiving log events from other contracts.
    SystemMessage = 1,
    /// Input channel for general messages pushed from pink contract part of this phat contract.
    GeneralMessage = 2,
    /// Input channel for queries from external RPC requests.
    Query = 3,
    /// Input channel for incoming HTTP requests.
    HttpRequest = 4,
}

impl I32Convertible for InputChannel {
    fn to_i32(&self) -> i32 {
        *self as i32
    }
    fn from_i32(i: i32) -> Result<Self> {
        match i {
            1 => Ok(InputChannel::SystemMessage),
            2 => Ok(InputChannel::GeneralMessage),
            3 => Ok(InputChannel::Query),
            4 => Ok(InputChannel::HttpRequest),
            _ => Err(OcallError::InvalidParameter),
        }
    }
}

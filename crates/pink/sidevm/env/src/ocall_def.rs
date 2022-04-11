use super::*;
use crate::args_stack::{I32Convertible, RetDecode, StackedArgs};
use std::borrow::Cow;

/// Poll state for a dynamic returned buffer (for the ocall `fn poll`).
#[derive(Encode, Decode)]
pub enum Poll<T> {
    /// Represents that a value is not ready yet.
    Pending,
    /// Represents that a value is immediately ready.
    Ready(T),
}

impl<T> Poll<T> {
    /// Maps a `Poll<T>` to `Poll<U>` by applying a function to a contained value.
    pub fn map<U, F>(self, f: F) -> Poll<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Poll::Ready(t) => Poll::Ready(f(t)),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<T> From<std::task::Poll<T>> for Poll<T> {
    fn from(poll: std::task::Poll<T>) -> Self {
        match poll {
            std::task::Poll::Ready(t) => Poll::Ready(t),
            std::task::Poll::Pending => Poll::Pending,
        }
    }
}

impl<T> From<Poll<T>> for std::task::Poll<T> {
    fn from(poll: Poll<T>) -> Self {
        match poll {
            Poll::Pending => std::task::Poll::Pending,
            Poll::Ready(t) => std::task::Poll::Ready(t),
        }
    }
}

// Poll state for poll_read/poll_write.
impl I32Convertible for Poll<u32> {
    fn to_i32(&self) -> i32 {
        match self {
            // In theory, the n could be u32::MAX which is the same binary representation as -1.
            // But in practice, no buffer in wasm32 can have a size of u32::MAX or greater.
            Self::Ready(n) => *n as i32,
            Self::Pending => -1,
        }
    }

    fn from_i32(i: i32) -> Result<Self> {
        Ok(match i {
            -1 => Self::Pending,
            n => Self::Ready(n as u32),
        })
    }
}

impl I32Convertible for Poll<()> {
    fn to_i32(&self) -> i32 {
        match self {
            Poll::Pending => -1,
            Poll::Ready(_) => 0,
        }
    }

    fn from_i32(i: i32) -> Result<Self>
    where
        Self: Sized,
    {
        match i {
            -1 => Ok(Self::Pending),
            0 => Ok(Self::Ready(())),
            _ => Err(OcallError::InvalidEncoding),
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
    fn poll(resource_id: i32) -> Result<Poll<Option<Vec<u8>>>>;

    /// Poll given resource to read data. Low level support for AsyncRead.
    #[ocall(id = 103, fast_input, fast_return)]
    fn poll_read(resource_id: i32, data: &mut [u8]) -> Result<Poll<u32>>;

    /// Poll given resource to write data. Low level support for AsyncWrite.
    #[ocall(id = 104, fast_input, fast_return)]
    fn poll_write(resource_id: i32, data: &[u8]) -> Result<Poll<u32>>;

    /// Shutdown a socket
    #[ocall(id = 105, fast_input, fast_return)]
    fn poll_shutdown(resource_id: i32) -> Result<Poll<()>>;

    /// Mark a task as ready for next polling
    #[ocall(id = 109, fast_input, fast_return)]
    fn mark_task_ready(task_id: i32) -> Result<()>;

    /// Get the next waken up task id.
    #[ocall(id = 110, fast_input, fast_return)]
    fn next_ready_task() -> Result<i32>;

    /// Enable logging for ocalls
    #[ocall(id = 111, fast_return)]
    fn enable_ocall_trace(enable: bool) -> Result<()>;

    /// Create a timer given a duration of time in milliseconds.
    #[ocall(id = 201, fast_input, fast_return)]
    fn create_timer(timeout: i32) -> Result<i32>;

    /// Create a TCP socket, bind to given address and listen to incoming connections.
    ///
    /// The backlog argument defines the maximum length to which the queue of pending connections
    /// for sockfd may grow.
    ///
    /// Invoke tcp_accept on the returned resource_id to accept incoming connections.
    #[ocall(id = 210, fast_return)]
    fn tcp_listen(addr: Cow<str>, backlog: i32) -> Result<i32>;

    /// Accept incoming TCP connections.
    #[ocall(id = 211, fast_input)]
    fn tcp_accept(resource_id: i32) -> Result<Poll<i32>>;

    /// Print log message.
    #[ocall(id = 220, fast_return)]
    fn log(message: Cow<str>) -> Result<()>;
}

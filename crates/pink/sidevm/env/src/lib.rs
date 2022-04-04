use core::ops::{Deref, DerefMut};

use scale::{Decode, Encode};
use tinyvec::TinyVec;

pub use ocall_def::*;

mod ocall_def;
mod args_stack;

cfg_if::cfg_if! {
    if #[cfg(any(target_pointer_width = "32", feature = "host"))] {
        pub type IntPtr = i32;
    } else {
        // For unit test
        pub type IntPtr = i64;
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Encode, Decode)]
pub enum LogLevel {
    None = 0,
    Error,
    Warn,
    Debug,
    Trace,
}

#[derive(Clone, Copy, Debug)]
#[repr(i32)]
pub enum OcallError {
    Ok = 0,
    UnknownCallNumber = -1,
    InvalidAddress = -2,
    InvalidParameter = -3,
    InvalidEncoding = -4,
    NoMemory = -5,
    NoReturnValue = -6,
    ResourceNotFound = -7,
    Pending = -8,
}

impl OcallError {
    pub fn to_errno(self) -> i32 {
        self as i32
    }
}

pub type Result<T, E = OcallError> = core::result::Result<T, E>;
pub trait OcallEnv {
    fn put_return(&mut self, rv: Vec<u8>) -> usize;
    fn take_return(&mut self) -> Option<Vec<u8>>;
    fn copy_to_vm(&self, data: &[u8], ptr: IntPtr) -> Result<()>;
    fn slice_from_vm(&self, ptr: IntPtr, len: IntPtr) -> Result<&[u8]>;
    fn slice_from_vm_mut(&self, ptr: IntPtr, len: IntPtr) -> Result<&mut [u8]>;
}

extern "C" {
    pub fn sidevm_ocall(
        task_id: i32,
        func_id: i32,
        p0: IntPtr,
        p1: IntPtr,
        p2: IntPtr,
        p3: IntPtr,
    ) -> IntPtr;
    pub fn sidevm_ocall_fast_return(
        task_id: i32,
        func_id: i32,
        p0: IntPtr,
        p1: IntPtr,
        p2: IntPtr,
        p3: IntPtr,
    ) -> IntPtr;
}

#[derive(Default)]
struct Buffer(TinyVec<[u8; 128]>);

impl Deref for Buffer {
    type Target = TinyVec<[u8; 128]>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Buffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl scale::Output for Buffer {
    fn write(&mut self, bytes: &[u8]) {
        self.0.extend_from_slice(bytes)
    }
}

impl AsRef<[u8]> for Buffer {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

fn empty_buffer() -> Buffer {
    Default::default()
}

fn alloc_buffer(size: usize) -> Buffer {
    let mut buf = Buffer::default();
    buf.0.resize(size, 0_u8);
    buf
}

thread_local! {
    static CURRENT_TASK: std::cell::Cell<i32>  = Default::default();
}

pub fn current_task() -> i32 {
    CURRENT_TASK.with(|id| id.get())
}

pub fn set_current_task(task_id: i32) {
    CURRENT_TASK.with(|id| id.set(task_id))
}
#[cfg(test)]
mod tests;

use core::ops::{Deref, DerefMut};

use scale::{Decode, Encode};
use tinyvec::TinyVec;

cfg_if::cfg_if! {
    if #[cfg(any(target_pointer_width = "32", feature = "runtime"))] {
        pub type IntPtr = i32;
    } else {
        // For unit test
        pub type IntPtr = i64;
    }
}

#[derive(Clone, Copy)]
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

#[pink_sidevm_macro::ocall]
pub trait OcallFuncs {
    #[ocall(id = 100)]
    fn echo(&self, input: Vec<u8>) -> Vec<u8>;

    #[ocall(id = 101, fast_input, fast_return)]
    fn close(&self, resource_id: i32) -> i32;

    #[ocall(id = 102, fast_input, fast_return)]
    fn poll(&self, resource_id: i32) -> i32;

    #[ocall(id = 103, fast_input, fast_return)]
    fn next_ready_task(&self) -> i32;

    #[ocall(id = 201, fast_input, fast_return)]
    fn create_timer(&self, timeout: i32) -> i32;
}

#[cfg(test)]
mod test {
    use super::*;
    use std::cell::Cell;
    #[pink_sidevm_macro::ocall]
    pub trait TestOCall {
        #[ocall(id = 100)]
        fn echo(&self, input: Vec<u8>) -> Vec<u8>;

        #[ocall(id = 101)]
        fn add(&self, a: u32, b: u32) -> u32;

        #[ocall(id = 102, fast_input, fast_return)]
        fn add_fi_fo(&self, a: u32, b: u32) -> u32;

        #[ocall(id = 103, fast_input)]
        fn add_fi(&self, a: u32, b: u32) -> u32;

        #[ocall(id = 104, fast_return)]
        fn add_fo(&self, a: u32, b: u32) -> u32;

        #[ocall(id = 105, fast_input, fast_return)]
        fn add_fi_fo_64(&self, a: u64, b: u64) -> u64;
    }

    struct Backend;
    impl TestOCall for Backend {
        fn echo(&self, input: Vec<u8>) -> Vec<u8> {
            return input.to_vec();
        }
        fn add(&self, a: u32, b: u32) -> u32 {
            a + b
        }
        fn add_fi_fo(&self, a: u32, b: u32) -> u32 {
            a + b
        }
        fn add_fi(&self, a: u32, b: u32) -> u32 {
            a + b
        }
        fn add_fo(&self, a: u32, b: u32) -> u32 {
            a + b
        }

        fn add_fi_fo_64(&self, a: u64, b: u64) -> u64 {
            a + b
        }
    }

    impl OcallEnv for Backend {
        fn put_return(&self, v: Vec<u8>) -> usize {
            let len = v.len();
            RETURN_VALUE.with(move |value| {
                value.set(Some(v));
            });
            len
        }

        fn take_return(&self) -> Option<Vec<u8>> {
            RETURN_VALUE.with(move |value| value.take())
        }

        fn copy_to_vm(&self, data: &[u8], ptr: IntPtr) -> Result<()> {
            let dst_buf = unsafe { core::slice::from_raw_parts_mut(ptr as _, data.len()) };
            dst_buf.clone_from_slice(&data);
            Ok(())
        }

        fn with_slice_from_vm<T>(
            &self,
            ptr: IntPtr,
            len: IntPtr,
            f: impl FnOnce(&[u8]) -> T,
        ) -> Result<T> {
            let buf = unsafe { core::slice::from_raw_parts(ptr as _, len as _) };
            Ok(f(buf))
        }
    }

    thread_local! {
        static RETURN_VALUE: Cell<Option<Vec<u8>>> = Default::default();
    }

    #[no_mangle]
    extern "C" fn sidevm_ocall(
        task_id: i32,
        func_id: i32,
        p0: IntPtr,
        p1: IntPtr,
        p2: IntPtr,
        p3: IntPtr,
    ) -> IntPtr {
        let rv: IntPtr = match dispatch_call(&Backend, func_id, p0, p1, p2, p3) {
            Ok(rv) => rv,
            Err(err) => err.to_errno().into(),
        };
        println!("sidevm_ocall {} rv={}", func_id, rv);
        rv
    }

    #[no_mangle]
    extern "C" fn sidevm_ocall_fast_return(
        task_id: i32,
        func_id: i32,
        p0: IntPtr,
        p1: IntPtr,
        p2: IntPtr,
        p3: IntPtr,
    ) -> IntPtr {
        let rv: IntPtr = match dispatch_call_fast_return(&Backend, func_id, p0, p1, p2, p3) {
            Ok(rv) => rv,
            Err(err) => err.to_errno().into(),
        };
        println!("sidevm_ocall_fast_return {} rv={}", func_id, rv);
        rv
    }

    #[test]
    fn test_echo() {
        let pong = TestOCallImplement.echo(b"Hello".to_vec());
        assert_eq!(&pong, "Hello".as_bytes());
    }

    #[test]
    fn test_fi_fo() {
        let a = u32::MAX / 2;
        let b = 2;
        let c = a + b;
        assert_eq!(TestOCallImplement.add(a, b), c);
        assert_eq!(TestOCallImplement.add_fi(a, b), c);
        assert_eq!(TestOCallImplement.add_fo(a, b), c);
        assert_eq!(TestOCallImplement.add_fi_fo(a, b), c);
    }

    #[test]
    fn test_fi_fo_64() {
        let a = u64::MAX / 2;
        let b = 2;
        let c = a + b;
        assert_eq!(TestOCallImplement.add_fi_fo_64(a, b), c);
    }
}

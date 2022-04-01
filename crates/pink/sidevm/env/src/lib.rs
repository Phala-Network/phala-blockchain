#![cfg_attr(not(test), no_std)]
extern crate alloc;

use core::ops::{Deref, DerefMut};

use alloc::vec::Vec;
use scale::{Decode, Encode};
use tinyvec::TinyVec;

#[cfg(target_pointer_width = "32")]
pub type IntPtr = i32;

#[cfg(target_pointer_width = "64")]
pub type IntPtr = i64;

extern "C" {
    pub fn sidevm_ocall(func_id: i32, p0: IntPtr, p1: IntPtr, p2: IntPtr, p3: IntPtr) -> IntPtr;
    pub fn sidevm_ocall_fast_return(
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

#[pink_sidevm_macro::ocall]
pub trait OcallFuncs {
    #[ocall(id = 100)]
    fn echo(&self, input: Vec<u8>) -> Vec<u8>;
}

#[cfg(test)]
mod test {
    use core::cell::Cell;

    use super::*;
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

        fn copy_to_vm(&self, data: &[u8], ptr: IntPtr) {
            let dst_buf = unsafe { core::slice::from_raw_parts_mut(ptr as _, data.len()) };
            dst_buf.clone_from_slice(&data);
        }

        fn slice_from_vm(&self, ptr: IntPtr, len: IntPtr) -> &[u8] {
            unsafe { core::slice::from_raw_parts(ptr as _, len as _) }
        }
    }

    thread_local! {
        static RETURN_VALUE: Cell<Option<Vec<u8>>> = Default::default();
    }

    #[no_mangle]
    extern "C" fn sidevm_ocall(
        func_id: i32,
        p0: IntPtr,
        p1: IntPtr,
        p2: IntPtr,
        p3: IntPtr,
    ) -> IntPtr {
        let rv: IntPtr = dispatch_call_slow(&mut Backend, func_id, p0, p1, p2, p3);
        println!("sidevm_ocall {} rv={}", func_id, rv);
        rv
    }

    #[no_mangle]
    extern "C" fn sidevm_ocall_fast_return(
        func_id: i32,
        p0: IntPtr,
        p1: IntPtr,
        p2: IntPtr,
        p3: IntPtr,
    ) -> IntPtr {
        let rv: IntPtr = dispatch_call_fast_return(&mut Backend, func_id, p0, p1, p2, p3);
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

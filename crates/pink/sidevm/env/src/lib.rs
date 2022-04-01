#![cfg_attr(not(test), no_std)]
extern crate alloc;

use std::ops::{Deref, DerefMut};

use alloc::vec::Vec;
use scale::{Decode, Encode};
use tinyvec::TinyVec;

#[cfg(target_pointer_width = "32")]
pub type PtrInt = i32;

#[cfg(target_pointer_width = "64")]
pub type PtrInt = i64;

extern "C" {
    pub fn sidevm_ocall(func_id: i32, p0: PtrInt, p1: PtrInt, p2: PtrInt, p3: PtrInt) -> PtrInt;
    pub fn sidevm_ocall_fast(func_id: i32, p0: PtrInt, p1: PtrInt, p2: PtrInt, p3: PtrInt) -> PtrInt;
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
pub trait OCall {
    #[ocall(id = 100)]
    fn echo(&self, input: &[u8]) -> Vec<u8>;
}

#[cfg(test)]
mod test {
    use std::sync::Mutex;
    use once_cell::sync::Lazy;

    use super::*;
    #[pink_sidevm_macro::ocall]
    pub trait TestOCall {
        #[ocall(id = 100)]
        fn echo(&self, input: Vec<u8>) -> Vec<u8>;
    }

    struct Backend;
    impl TestOCall for Backend {
        fn echo(&self, input: Vec<u8>) -> Vec<u8> {
            return input.to_vec();
        }
    }

    static RETURN_VALUE: Lazy<Mutex<Option<Vec<u8>>>> = Lazy::new(Default::default);

    macro_rules! decode_input {
        ($p0: expr, $p1: expr) => {{
            let ptr = $p0 as *mut u8;
            let len = $p1 as usize;
            let mut buf = unsafe { core::slice::from_raw_parts(ptr, len) };
            match Decode::decode(&mut buf) {
                Ok(p) => {
                    p
                }
                Err(_) => return -1,
            }
        }};
    }

    macro_rules! encode_result {
        ($rv: expr) => {{
            let ret = Encode::encode(&$rv);
            let len = ret.len();
            *RETURN_VALUE.lock().unwrap() = Some(ret);
            len as PtrInt
        }};
    }

    macro_rules! dispatch_call_fast {
        ($id: ident, $p0: ident, $p1: ident, $p2: ident, $p3: ident) => {{
            match $id {
                0 => {
                    let buffer = RETURN_VALUE.lock().unwrap().take().unwrap_or_default();
                    let ptr = $p0 as *mut u8;
                    let len = $p1 as usize;
                    if buffer.len() != len {
                        return -1;
                    }
                    let dst_buf = unsafe { core::slice::from_raw_parts_mut(ptr, len) };
                    dst_buf.clone_from_slice(&buffer);
                    len as PtrInt
                }
                _ => -1,
            }
        }};
    }

    macro_rules! dispatch_call_slow {
        ($id: ident, $p0: ident, $p1: ident, $p2: ident, $p3: ident) => {{
            match $id {
                0 => {{
                    let ret = dispatch_call_fast!($id, $p0, $p1, $p2, $p3);
                    encode_result!(ret as PtrInt)
                }}
                100 => {{
                    let input = decode_input!($p0, $p1);
                    let rv = Backend.echo(input);
                    encode_result!(rv)
                }}
                _ => {
                    return -1;
                }
            }
        }};
    }

    #[no_mangle]
    extern "C" fn sidevm_ocall(func_id: i32, p0: PtrInt, p1: PtrInt, _: PtrInt, _: PtrInt) -> PtrInt {
        let rv: PtrInt = dispatch_call_slow!(func_id, p0, p1, p2, p3);
        println!("sidevm_ocall {} rv={}", func_id, rv);
        rv
    }

    #[no_mangle]
    extern "C" fn sidevm_ocall_fast(func_id: i32, p0: PtrInt, p1: PtrInt, _: PtrInt, _: PtrInt) -> PtrInt {
        let rv: PtrInt = dispatch_call_fast!(func_id, p0, p1, p2, p3);
        println!("sidevm_ocall_fast {} rv={}", func_id, rv);
        rv
    }

    #[test]
    fn test_echo() {
        let pong = TestOCallImplement.echo(b"Hello".to_vec());
        assert_eq!(&pong, "Hello".as_bytes());
    }
}

use super::*;
use std::cell::Cell;
#[pink_sidevm_macro::ocall]
pub trait TestOcall {
    #[ocall(id = 100)]
    fn echo(input: Vec<u8>) -> Vec<u8>;

    #[ocall(id = 101)]
    fn add(a: u32, b: u32) -> u32;

    #[ocall(id = 102, fast_input, fast_return)]
    fn add_fi_fo(a: u32, b: u32) -> u32;

    #[ocall(id = 103, fast_input)]
    fn add_fi(a: u32, b: u32) -> u32;

    #[ocall(id = 104, fast_return)]
    fn add_fo(a: u32, b: u32) -> u32;

    #[ocall(id = 105, fast_input, fast_return)]
    fn add_fi_fo_64(a: u64, b: u64) -> u64;
}

struct Backend;
impl TestOcall for Backend {
    fn echo(&mut self, input: Vec<u8>) -> Vec<u8> {
        return input.to_vec();
    }
    fn add(&mut self, a: u32, b: u32) -> u32 {
        a + b
    }
    fn add_fi_fo(&mut self, a: u32, b: u32) -> u32 {
        a + b
    }
    fn add_fi(&mut self, a: u32, b: u32) -> u32 {
        a + b
    }
    fn add_fo(&mut self, a: u32, b: u32) -> u32 {
        a + b
    }

    fn add_fi_fo_64(&mut self, a: u64, b: u64) -> u64 {
        a + b
    }
}

impl OcallEnv for Backend {
    fn put_return(&mut self, v: Vec<u8>) -> usize {
        let len = v.len();
        RETURN_VALUE.with(move |value| {
            value.set(Some(v));
        });
        len
    }

    fn take_return(&mut self) -> Option<Vec<u8>> {
        RETURN_VALUE.with(move |value| value.take())
    }

    fn copy_to_vm(&mut self, data: &[u8], ptr: IntPtr) -> Result<()> {
        let dst_buf = unsafe { core::slice::from_raw_parts_mut(ptr as _, data.len()) };
        dst_buf.clone_from_slice(&data);
        Ok(())
    }

    fn slice_from_vm(&mut self, ptr: IntPtr, len: IntPtr) -> Result<&[u8]> {
        let buf = unsafe { core::slice::from_raw_parts(ptr as _, len as _) };
        Ok(buf)
    }

    fn slice_from_vm_mut(&mut self, ptr: IntPtr, len: IntPtr) -> Result<&mut [u8]> {
        let buf = unsafe { core::slice::from_raw_parts_mut(ptr as _, len as _) };
        Ok(buf)
    }
}

thread_local! {
    static RETURN_VALUE: Cell<Option<Vec<u8>>> = Default::default();
}

#[no_mangle]
extern "C" fn sidevm_ocall(
    _task_id: i32,
    func_id: i32,
    p0: IntPtr,
    p1: IntPtr,
    p2: IntPtr,
    p3: IntPtr,
) -> IntPtr {
    let rv: IntPtr = match dispatch_call(&mut Backend, func_id, p0, p1, p2, p3) {
        Ok(rv) => rv,
        Err(err) => err.to_errno().into(),
    };
    println!("sidevm_ocall {} rv={}", func_id, rv);
    rv
}

#[no_mangle]
extern "C" fn sidevm_ocall_fast_return(
    _task_id: i32,
    func_id: i32,
    p0: IntPtr,
    p1: IntPtr,
    p2: IntPtr,
    p3: IntPtr,
) -> IntPtr {
    let rv: IntPtr = match dispatch_call_fast_return(&mut Backend, func_id, p0, p1, p2, p3) {
        Ok(rv) => rv,
        Err(err) => err.to_errno().into(),
    };
    println!("sidevm_ocall_fast_return {} rv={}", func_id, rv);
    rv
}

use test_ocall_guest as ocall;

#[test]
fn test_echo() {
    let pong = ocall::echo(b"Hello".to_vec());
    assert_eq!(&pong, "Hello".as_bytes());
}

#[test]
fn test_fi_fo() {
    let a = u32::MAX / 2;
    let b = 2;
    let c = a + b;
    assert_eq!(ocall::add(a, b), c);
    assert_eq!(ocall::add_fi(a, b), c);
    assert_eq!(ocall::add_fo(a, b), c);
    assert_eq!(ocall::add_fi_fo(a, b), c);
}

#[test]
fn test_fi_fo_64() {
    let a = u64::MAX / 2;
    let b = 2;
    let c = a + b;
    assert_eq!(ocall::add_fi_fo_64(a, b), c);
}

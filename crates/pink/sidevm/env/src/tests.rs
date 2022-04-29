use crate::args_stack::{I32Convertible, RetDecode, RetEncode, StackedArgs};

use super::*;
use std::cell::Cell;
#[pink_sidevm_macro::ocall]
pub trait TestOcall {
    #[ocall(id = 100)]
    fn echo(input: Vec<u8>) -> Result<Vec<u8>>;

    #[ocall(id = 101)]
    fn add(a: u32, b: u32) -> Result<u32>;

    #[ocall(id = 102, fast_input, fast_return)]
    fn add_fi_fo(a: u32, b: u32) -> Result<u32>;

    #[ocall(id = 103, fast_input)]
    fn add_fi(a: u32, b: u32) -> Result<u32>;

    #[ocall(id = 104, fast_return)]
    fn add_fo(a: u32, b: u32) -> Result<u32>;

    #[ocall(id = 105, fast_input)]
    fn sub_fi(a: i32, b: i32) -> Result<i32>;

    #[ocall(id = 106, fast_input, fast_return)]
    fn copy(dst: &mut [u8], src: &[u8]) -> Result<()>;
}

struct TestHost;
impl TestOcall for TestHost {
    fn echo(&mut self, input: Vec<u8>) -> Result<Vec<u8>> {
        Ok(input.to_vec())
    }
    fn add(&mut self, a: u32, b: u32) -> Result<u32> {
        Ok(a.wrapping_add(b))
    }
    fn add_fi_fo(&mut self, a: u32, b: u32) -> Result<u32> {
        Ok(a.wrapping_add(b))
    }
    fn add_fi(&mut self, a: u32, b: u32) -> Result<u32> {
        Ok(a.wrapping_add(b))
    }
    fn add_fo(&mut self, a: u32, b: u32) -> Result<u32> {
        Ok(a.wrapping_add(b))
    }
    fn sub_fi(&mut self, a: i32, b: i32) -> Result<i32> {
        Ok(a.wrapping_sub(b))
    }
    fn copy(&mut self, dst: &mut [u8], src: &[u8]) -> Result<()> {
        dst.copy_from_slice(src);
        Ok(())
    }
}

impl OcallEnv for TestHost {
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
}

impl VmMemory for TestHost {
    fn copy_to_vm(&self, data: &[u8], ptr: IntPtr) -> Result<()> {
        let dst_buf = unsafe { core::slice::from_raw_parts_mut(ptr as _, data.len()) };
        dst_buf.clone_from_slice(&data);
        Ok(())
    }

    fn slice_from_vm(&self, ptr: IntPtr, len: IntPtr) -> Result<&[u8]> {
        let buf = unsafe { core::slice::from_raw_parts(ptr as _, len as _) };
        Ok(buf)
    }

    fn slice_from_vm_mut(&self, ptr: IntPtr, len: IntPtr) -> Result<&mut [u8]> {
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
) -> IntRet {
    let result = dispatch_call(&mut TestHost, &TestHost, func_id, p0, p1, p2, p3);
    println!("sidevm_ocall {} result={:?}", func_id, result);
    result.encode_ret()
}

#[no_mangle]
extern "C" fn sidevm_ocall_fast_return(
    _task_id: i32,
    func_id: i32,
    p0: IntPtr,
    p1: IntPtr,
    p2: IntPtr,
    p3: IntPtr,
) -> IntRet {
    let result = dispatch_call_fast_return(&mut TestHost, &TestHost, func_id, p0, p1, p2, p3);
    println!("sidevm_ocall_fast_return {} result={:?}", func_id, result);
    result.encode_ret()
}

use test_ocall_guest as ocall;

#[test]
fn test_echo() {
    let pong = ocall::echo(b"Hello".to_vec()).unwrap();
    assert_eq!(&pong, "Hello".as_bytes());
}

#[test]
fn test_fi_fo() {
    let a = u32::MAX / 2;
    let b = 2;
    let c = a.wrapping_add(b);
    assert_eq!(ocall::add(a, b).unwrap(), c);
    assert_eq!(ocall::add_fi(a, b).unwrap(), c);
    assert_eq!(ocall::add_fo(a, b).unwrap(), c);
    assert_eq!(ocall::add_fi_fo(a, b).unwrap(), c);
    assert_eq!(ocall::sub_fi(4, 1).unwrap(), 3);
    assert_eq!(ocall::sub_fi(1, 4).unwrap(), -3);
}

#[test]
fn test_fi_fo_buf() {
    let mut a = [0u8; 4];
    let b = [1u8, 2, 3, 4];
    ocall::copy(&mut a[..], &b[..]).unwrap();
    assert_eq!(a, b);
}

#[test]
fn test_fi_fo_overflow() {
    let a = u32::MAX;
    let b = 1;
    let c = a.wrapping_add(b);
    assert_eq!(ocall::add(a, b).unwrap(), c);
    assert_eq!(ocall::add_fi(a, b).unwrap(), c);
    assert_eq!(ocall::add_fo(a, b).unwrap(), c);
    assert_eq!(ocall::add_fi_fo(a, b).unwrap(), c);
}

#[test]
fn test_nargs_encode() {
    let stack = StackedArgs::empty();
    let stack = stack.push_arg(1u8);
    let stack = stack.push_arg(2u64);
    let stack = stack.push_arg(3u32);
    assert_eq!(stack.dump(), [1, 0, 2, 3]);
}

#[test]
fn test_nargs_decode() {
    let args: &[IntPtr] = &[1, 2, 3, 4, 5];
    let stack = StackedArgs::load(args).unwrap();

    let (_, stack): (i32, _) = stack.pop_arg(&TestHost).unwrap();
    let (_, stack): (i64, _) = stack.pop_arg(&TestHost).unwrap();
    let (c, stack): (i32, _) = stack.pop_arg(&TestHost).unwrap();
    let _: StackedArgs<()> = stack;
    assert_eq!(c, 1);
}

#![no_std]
extern crate alloc;

use alloc::vec::Vec;
use scale::{Decode, Encode};

pub type ptr = i32;
pub type size = i32;

extern "C" {
    fn sidevm_ocall(
        func_id: u32,
        input: ptr,
        input_len: size,
        output: ptr,
        output_len_ptr: ptr,
    ) -> i32;
}

fn ocall<Input: Encode, Output: Decode>(func_id: u32, input: &Input) -> Output {
    // TODO.kevin: use a pre-allocated buffer
    let output_buffer = alloc::vec![0u8; 1024*256];
    let input_buffer = input.encode();
    let input = &input_buffer[0] as *const u8 as ptr;
    let input_len = input_buffer.len() as size;
    let output = &output_buffer[0] as *const u8 as ptr;
    let mut output_len = output_buffer.len() as size;
    let output_len_ptr = &mut output_len as *mut size as ptr;
    let rv = unsafe { sidevm_ocall(func_id, input, input_len, output, output_len_ptr) };
    if rv != 0 {
        panic!("sidevm_ocall failed, error: {}", rv);
    }
    let mut output = &output_buffer[0..output_len as usize];
    Decode::decode(&mut output).expect("Failed to decode ocall output")
}

#[pink_extension_macro::sidevm_extension]
trait SideVMHostFunctions {
    fn echo(&self, input: &[u8]) -> Vec<u8>;
}

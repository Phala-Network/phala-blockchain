use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::Poll,
    time::Duration,
};

use log::error;
use wasmer::{
    imports, Function, ImportObject, Memory, MemoryType, Pages, Store, WasmPtr, WasmerEnv,
};

use pink_sidevm_env::{
    dispatch_call, dispatch_call_fast_return, IntPtr, OcallEnv, OcallError, OcallFuncs, Result,
};

use crate::async_context::poll_in_task_cx;
use crate::resource::{Resource, ResourceKeeper};

fn _sizeof_i32_must_eq_to_intptr() {
    let _ = core::mem::transmute::<i32, IntPtr>;
}

pub fn create_env(store: &Store) -> (Env, ImportObject) {
    let env = Env::new(store);
    (
        env.clone(),
        imports! {
            "env" => {
                "sidevm_ocall" => Function::new_native_with_env(
                    store,
                    env.clone(),
                    sidevm_ocall,
                ),
                "sidevm_ocall_fast_return" => Function::new_native_with_env(
                    store,
                    env,
                    sidevm_ocall_fast_return,
                ),
            }
        },
    )
}

struct EnvInner {
    resources: ResourceKeeper,
    memory: Option<Memory>,
    temp_return_value: Option<Vec<u8>>,
}

#[derive(WasmerEnv, Clone)]
pub struct Env {
    inner: Arc<Mutex<EnvInner>>,
}

impl Env {
    fn new(store: &Store) -> Self {
        Self {
            inner: Arc::new(Mutex::new(EnvInner {
                resources: ResourceKeeper::default(),
                memory: None,
                temp_return_value: None,
            })),
        }
    }

    pub fn set_memory(&self, memory: Memory) {
        self.inner.lock().unwrap().memory = Some(memory);
    }

    pub fn cleanup(&self) {
        // Cut up the reference cycle to avoid leaks.
        self.inner.lock().unwrap().memory = None;
    }
}

fn check_addr(memory: &Memory, offset: usize, len: usize) -> Result<(usize, usize)> {
    let end = offset.checked_add(len).ok_or(OcallError::InvalidAddress)?;
    if end > memory.size().bytes().0 {
        return Err(OcallError::InvalidAddress);
    }
    Ok((offset, end))
}

impl OcallEnv for Env {
    fn put_return(&self, rv: Vec<u8>) -> usize {
        let len = rv.len();
        self.inner.lock().unwrap().temp_return_value = Some(rv);
        len
    }

    fn take_return(&self) -> Option<Vec<u8>> {
        self.inner.lock().unwrap().temp_return_value.take()
    }

    fn copy_to_vm(&self, data: &[u8], ptr: IntPtr) -> Result<()> {
        if data.len() > u32::MAX as usize {
            return Err(OcallError::NoMemory);
        }
        let inner = self.inner.lock().unwrap();
        let memory = inner.memory.as_ref().ok_or(OcallError::NoMemory)?;
        let (offset, end) = check_addr(memory, ptr as _, data.len())?;
        let mem = unsafe { &mut memory.data_unchecked_mut()[offset..end] };
        mem.clone_from_slice(data);
        Ok(())
    }

    fn with_slice_from_vm<T>(
        &self,
        ptr: IntPtr,
        len: IntPtr,
        f: impl FnOnce(&[u8]) -> T,
    ) -> Result<T> {
        let inner = self.inner.lock().unwrap();
        let memory = inner.memory.as_ref().ok_or(OcallError::NoMemory)?;
        let (offset, end) = check_addr(memory, ptr as _, len as _)?;
        let slice = unsafe { &memory.data_unchecked()[offset..end] };
        Ok(f(slice))
    }
}

impl OcallFuncs for Env {
    fn echo(&self, input: Vec<u8>) -> Vec<u8> {
        input
    }
}

fn sidevm_ocall_fast_return(
    env: &Env,
    func_id: i32,
    p0: IntPtr,
    p1: IntPtr,
    p2: IntPtr,
    p3: IntPtr,
) -> IntPtr {
    match dispatch_call_fast_return(env, func_id, p0, p1, p2, p3) {
        Err(err) => err.to_errno().into(),
        Ok(rv) => rv,
    }
}

// Support all ocalls. Put the result into a temporary vec and wait for next fetch_result ocall to fetch the result.
fn sidevm_ocall(env: &Env, func_id: i32, p0: IntPtr, p1: IntPtr, p2: IntPtr, p3: IntPtr) -> IntPtr {
    match dispatch_call(env, func_id, p0, p1, p2, p3) {
        Err(err) => err.to_errno().into(),
        Ok(rv) => rv,
    }
}

use std::{
    cell::Cell,
    sync::{Arc, Mutex},
    time::Duration,
};

use wasmer::{imports, Function, ImportObject, Memory, Store, WasmerEnv};

use env::{IntPtr, IntRet, LogLevel, OcallError, Result, RetEncode};
use pink_sidevm_env as env;
use thread_local::ThreadLocal;

use crate::resource::{Resource, ResourceKeeper};

fn _sizeof_i32_must_eq_to_intptr() {
    let _ = core::mem::transmute::<i32, IntPtr>;
}

pub fn create_env(store: &Store) -> (Env, ImportObject) {
    let env = Env::new();
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

struct State {
    resources: ResourceKeeper,
    temp_return_value: ThreadLocal<Cell<Option<Vec<u8>>>>,
    log_level: LogLevel,
    ocall_trace_enabled: bool,
}

struct VmMemory(Option<Memory>);

struct EnvInner {
    memory: VmMemory,
    state: State,
}

#[derive(WasmerEnv, Clone)]
pub struct Env {
    inner: Arc<Mutex<EnvInner>>,
}

impl Env {
    fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(EnvInner {
                memory: VmMemory(None),
                state: State {
                    resources: ResourceKeeper::default(),
                    temp_return_value: Default::default(),
                    log_level: LogLevel::None,
                    ocall_trace_enabled: false,
                },
            })),
        }
    }

    pub fn set_memory(&self, memory: Memory) {
        self.inner.lock().unwrap().memory.0 = Some(memory);
    }

    pub fn cleanup(&self) {
        // Cut up the reference cycle to avoid leaks.
        self.inner.lock().unwrap().memory.0 = None;
    }
}

fn check_addr(memory: &Memory, offset: usize, len: usize) -> Result<(usize, usize)> {
    let end = offset.checked_add(len).ok_or(OcallError::InvalidAddress)?;
    if end > memory.size().bytes().0 {
        return Err(OcallError::InvalidAddress);
    }
    Ok((offset, end))
}

impl env::OcallEnv for State {
    fn put_return(&mut self, rv: Vec<u8>) -> usize {
        let len = rv.len();
        self.temp_return_value.get_or_default().set(Some(rv));
        len
    }

    fn take_return(&mut self) -> Option<Vec<u8>> {
        self.temp_return_value.get_or_default().take()
    }
}

impl env::VmMemory for VmMemory {
    fn copy_to_vm(&self, data: &[u8], ptr: IntPtr) -> Result<()> {
        if data.len() > u32::MAX as usize {
            return Err(OcallError::NoMemory);
        }
        let memory = self.0.as_ref().ok_or(OcallError::NoMemory)?;
        let (offset, end) = check_addr(memory, ptr as _, data.len())?;
        let mem = unsafe { &mut memory.data_unchecked_mut()[offset..end] };
        mem.clone_from_slice(data);
        Ok(())
    }

    fn slice_from_vm(&self, ptr: IntPtr, len: IntPtr) -> Result<&[u8]> {
        let memory = self.0.as_ref().ok_or(OcallError::NoMemory)?;
        let (offset, end) = check_addr(memory, ptr as _, len as _)?;
        let slice = unsafe { &memory.data_unchecked()[offset..end] };
        Ok(slice)
    }

    fn slice_from_vm_mut(&self, ptr: IntPtr, len: IntPtr) -> Result<&mut [u8]> {
        let memory = self.0.as_ref().ok_or(OcallError::NoMemory)?;
        let (offset, end) = check_addr(memory, ptr as _, len as _)?;
        let slice = unsafe { &mut memory.data_unchecked_mut()[offset..end] };
        Ok(slice)
    }
}

impl env::OcallFuncs for State {
    fn echo(&mut self, input: Vec<u8>) -> Result<Vec<u8>> {
        Ok(input)
    }

    fn close(&mut self, resource_id: i32) -> Result<()> {
        match self.resources.take(resource_id) {
            None => Err(OcallError::ResourceNotFound),
            Some(_res) => Ok(()),
        }
    }

    fn poll(&mut self, resource_id: i32) -> Result<()> {
        let res = self
            .resources
            .get_mut(resource_id)
            .ok_or(OcallError::ResourceNotFound)?;
        if res.poll(env::current_task()) {
            Ok(())
        } else {
            Err(OcallError::Pending)
        }
    }

    fn next_ready_task(&mut self) -> Result<i32> {
        Err(OcallError::ResourceNotFound)
    }

    fn create_timer(&mut self, timeout: i32) -> Result<i32> {
        let sleep = tokio::time::sleep(Duration::from_millis(timeout as u64));
        self.resources
            .push(Resource::Sleep(Box::pin(sleep)))
            .ok_or(OcallError::NoMemory)
    }

    fn set_log_level(&mut self, log_level: LogLevel) -> Result<()> {
        self.log_level = log_level;
        Ok(())
    }

    fn enable_ocall_trace(&mut self, enable: bool) -> Result<()> {
        self.ocall_trace_enabled = enable;
        Ok(())
    }
}

fn sidevm_ocall_fast_return(
    env: &Env,
    task_id: i32,
    func_id: i32,
    p0: IntPtr,
    p1: IntPtr,
    p2: IntPtr,
    p3: IntPtr,
) -> IntRet {
    env::set_current_task(task_id);
    let mut env = env.inner.lock().unwrap();
    let env = &mut *env;
    let result =
        env::dispatch_call_fast_return(&mut env.state, &env.memory, func_id, p0, p1, p2, p3);
    if env.state.ocall_trace_enabled {
        let func_name = env::ocall_id2name(func_id);
        eprintln!("[{task_id:>3}](F) {func_name}({p0}, {p1}, {p2}, {p3}) = {result:?}");
    }
    result.encode_ret()
}

// Support all ocalls. Put the result into a temporary vec and wait for next fetch_result ocall to fetch the result.
fn sidevm_ocall(
    env: &Env,
    task_id: i32,
    func_id: i32,
    p0: IntPtr,
    p1: IntPtr,
    p2: IntPtr,
    p3: IntPtr,
) -> IntRet {
    env::set_current_task(task_id);
    let mut env = env.inner.lock().unwrap();
    let env = &mut *env;
    let result = env::dispatch_call(&mut env.state, &env.memory, func_id, p0, p1, p2, p3);
    if env.state.ocall_trace_enabled {
        let func_name = env::ocall_id2name(func_id);
        eprintln!("[{task_id:>3}](S) {func_name}({p0}, {p1}, {p2}, {p3}) = {result:?}");
    }
    result.encode_ret()
}

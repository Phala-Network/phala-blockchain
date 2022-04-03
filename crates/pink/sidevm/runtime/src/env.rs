use std::{
    cell::Cell,
    sync::{Arc, Mutex},
    time::Duration,
};

use log::error;
use wasmer::{imports, Function, ImportObject, Memory, Store, WasmerEnv};

use env::{IntPtr, LogLevel, OcallError, Result};
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

struct EnvInner {
    resources: ResourceKeeper,
    memory: Option<Memory>,
    temp_return_value: ThreadLocal<Cell<Option<Vec<u8>>>>,
    log_level: LogLevel,
}

#[derive(WasmerEnv, Clone)]
pub struct Env {
    inner: Arc<Mutex<EnvInner>>,
}

impl Env {
    fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(EnvInner {
                resources: ResourceKeeper::default(),
                memory: None,
                temp_return_value: Default::default(),
                log_level: LogLevel::None,
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

impl env::OcallEnv for Env {
    fn put_return(&self, rv: Vec<u8>) -> usize {
        let len = rv.len();
        self.inner
            .lock()
            .unwrap()
            .temp_return_value
            .get_or_default()
            .set(Some(rv));
        len
    }

    fn take_return(&self) -> Option<Vec<u8>> {
        self.inner
            .lock()
            .unwrap()
            .temp_return_value
            .get_or_default()
            .take()
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

impl Env {
    fn take_resource(&self, resource_id: i32) -> Option<Resource> {
        self.inner.lock().unwrap().resources.take(resource_id)
    }

    fn with_resource<T>(&self, resource_id: i32, f: impl FnOnce(&mut Resource) -> T) -> Option<T> {
        self.inner
            .lock()
            .unwrap()
            .resources
            .get_mut(resource_id)
            .map(f)
    }

    fn push_resource(&self, resource: Resource) -> Option<i32> {
        self.inner.lock().unwrap().resources.push(resource)
    }

    fn log_level(&self) -> LogLevel {
        self.inner.lock().unwrap().log_level
    }
}

impl env::OcallFuncs for Env {
    fn echo(&self, input: Vec<u8>) -> Vec<u8> {
        input
    }

    fn close(&self, resource_id: i32) -> i32 {
        match self.take_resource(resource_id) {
            None => OcallError::ResourceNotFound as _,
            Some(_res) => OcallError::Ok as _,
        }
    }

    fn poll(&self, resource_id: i32) -> i32 {
        self.with_resource(resource_id, |res| {
            if res.poll(env::current_task()) {
                OcallError::Ok
            } else {
                OcallError::Pending
            }
        })
        .unwrap_or(OcallError::ResourceNotFound) as _
    }

    fn next_ready_task(&self) -> i32 {
        OcallError::ResourceNotFound as _
    }

    fn create_timer(&self, timeout: i32) -> i32 {
        let sleep = tokio::time::sleep(Duration::from_millis(timeout as u64));
        match self.push_resource(Resource::Sleep(Box::pin(sleep))) {
            Some(id) => id,
            None => {
                error!("failed to push sleep resource");
                OcallError::NoMemory as _
            }
        }
    }

    fn set_log_level(&self, log_level: LogLevel) -> i32 {
        self.inner.lock().unwrap().log_level = log_level;
        OcallError::Ok as _
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
) -> IntPtr {
    env::set_current_task(task_id);
    let rv = match env::dispatch_call_fast_return(env, func_id, p0, p1, p2, p3) {
        Ok(rv) => rv,
        Err(err) => err.to_errno().into(),
    };
    if env.log_level() >= LogLevel::Trace {
        let func_name = env::ocall_id2name(func_id);
        eprintln!("[{task_id:>3}](F) {func_name}({p0}, {p1}, {p2}, {p3}) = {rv}");
    }
    rv
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
) -> IntPtr {
    env::set_current_task(task_id);
    let rv = match env::dispatch_call(env, func_id, p0, p1, p2, p3) {
        Ok(rv) => rv,
        Err(err) => err.to_errno().into(),
    };

    if env.log_level() >= LogLevel::Trace {
        let func_name = env::ocall_id2name(func_id);
        eprintln!("[{task_id:>3}](S) {func_name}({p0}, {p1}, {p2}, {p3}) = {rv}");
    }
    rv
}

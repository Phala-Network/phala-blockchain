use std::{
    borrow::Cow,
    cell::Cell,
    sync::{Arc, Mutex},
    task::Poll::{Pending, Ready},
    time::Duration,
};

use tokio::{
    net::TcpListener,
    sync::mpsc::{error::SendError, Sender},
};
use wasmer::{imports, Function, ImportObject, Memory, Store, WasmerEnv};

use env::{current_task, IntPtr, IntRet, LogLevel, OcallError, Poll, Result, RetEncode};
use pink_sidevm_env as env;
use thread_local::ThreadLocal;

use crate::{
    async_context::poll_in_task_cx,
    resource::{Resource, ResourceKeeper},
};

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
    message_tx: Sender<Vec<u8>>,
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
        let (message_tx, message_rx) = tokio::sync::mpsc::channel(100);
        let mut resources = ResourceKeeper::default();
        let _ = resources.push(Resource::ChannelRx(message_rx));
        Self {
            inner: Arc::new(Mutex::new(EnvInner {
                memory: VmMemory(None),
                state: State {
                    resources,
                    temp_return_value: Default::default(),
                    log_level: LogLevel::None,
                    ocall_trace_enabled: false,
                    message_tx,
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

    /// Push a pink message into the SideVM instance.
    pub async fn push_message(&self, message: Vec<u8>) -> Result<(), SendError<Vec<u8>>> {
        let tx = self.inner.lock().unwrap().state.message_tx.clone();
        tx.send(message).await
    }

    /// The blocking version of `push_message`.
    pub fn blocking_push_message(&self, message: Vec<u8>) -> Result<(), SendError<Vec<u8>>> {
        self.inner
            .lock()
            .unwrap()
            .state
            .message_tx
            .blocking_send(message)
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
    fn close(&mut self, resource_id: i32) -> Result<()> {
        match self.resources.take(resource_id) {
            None => Err(OcallError::ResourceNotFound),
            Some(_res) => Ok(()),
        }
    }

    fn poll(&mut self, resource_id: i32) -> Result<Poll<Option<Vec<u8>>>> {
        let res = self.resources.get_mut(resource_id)?;
        res.poll(env::current_task())
    }

    fn poll_read(&mut self, resource_id: i32, data: &mut [u8]) -> Result<Poll<u32>> {
        let res = self.resources.get_mut(resource_id)?;
        res.poll_read(env::current_task(), data)
    }

    fn poll_write(&mut self, resource_id: i32, data: &[u8]) -> Result<Poll<u32>> {
        let res = self.resources.get_mut(resource_id)?;
        res.poll_write(env::current_task(), data)
    }

    fn next_ready_task(&mut self) -> Result<i32> {
        Err(OcallError::ResourceNotFound)
    }

    fn create_timer(&mut self, timeout: i32) -> Result<i32> {
        let sleep = tokio::time::sleep(Duration::from_millis(timeout as u64));
        self.resources.push(Resource::Sleep(Box::pin(sleep)))
    }

    fn set_log_level(&mut self, log_level: LogLevel) -> Result<()> {
        self.log_level = log_level;
        Ok(())
    }

    fn enable_ocall_trace(&mut self, enable: bool) -> Result<()> {
        self.ocall_trace_enabled = enable;
        Ok(())
    }

    fn tcp_listen(&mut self, addr: Cow<str>, _backlog: i32) -> Result<i32> {
        let std_listener = std::net::TcpListener::bind(&*addr).or(Err(OcallError::IoError))?;
        std_listener
            .set_nonblocking(true)
            .or(Err(OcallError::IoError))?;
        let listener = TcpListener::from_std(std_listener).or(Err(OcallError::IoError))?;
        self.resources.push(Resource::TcpListener(listener))
    }

    fn tcp_accept(&mut self, tcp_res_id: i32) -> Result<Poll<i32>> {
        let (stream, remote_addr) = {
            let res = self.resources.get_mut(tcp_res_id)?;
            let listener = match res {
                Resource::TcpListener(listener) => listener,
                _ => return Err(OcallError::UnsupportedOperation),
            };
            let accept = listener.accept();
            futures::pin_mut!(accept);
            match poll_in_task_cx(accept, current_task()) {
                Pending => return Ok(Poll::Pending),
                Ready(result) => result.or(Err(OcallError::IoError))?,
            }
        };
        let result = self.resources.push(Resource::TcpStream {
            stream,
            remote_addr,
        });
        result.map(|res_id| Poll::Ready(res_id))
    }

    fn log(&mut self, message: Cow<str>) -> Result<()> {
        // TODO.kevin.must: use log crate
        let vm_id = "VMID";
        let task = current_task();
        println!("[vm:{vm_id:<8}][{task:<3}] {message}");
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
        // TODO.kevin.must: use log crate
        let func_name = env::ocall_id2name(func_id);
        let vm_id = "VMID";
        eprintln!(
            "[vm:{vm_id:<8}][{task_id:<3}](F) {func_name}({p0}, {p1}, {p2}, {p3}) = {result:?}"
        );
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
        // TODO.kevin.must: use log crate
        let func_name = env::ocall_id2name(func_id);
        let vm_id = "VMID";
        eprintln!(
            "[vm:{vm_id:<8}][{task_id:<3}](S) {func_name}({p0}, {p1}, {p2}, {p3}) = {result:?}"
        );
    }
    result.encode_ret()
}

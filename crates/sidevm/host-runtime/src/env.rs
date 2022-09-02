use std::{
    borrow::Cow,
    cell::Cell,
    collections::VecDeque,
    fmt,
    future::Future,
    sync::{Arc, Mutex},
    task::Poll::{Pending, Ready},
    time::Duration,
};

use serde::{Deserialize, Serialize};
use tokio::{
    net::TcpListener,
    sync::mpsc::{error::SendError, Sender},
    sync::oneshot::Sender as OneshotSender,
};
use wasmer::{imports, Function, ImportObject, Instance, Memory, Store, WasmerEnv};

use env::{
    messages::{AccountId, QueryRequest, SystemMessage},
    tls::{TlsClientConfig, TlsServerConfig},
    IntPtr, IntRet, OcallError, OcallFuncs, Result, RetEncode,
};
use sidevm_env as env;
use scale::Encode;
use thread_local::ThreadLocal;
use wasmer_middlewares::metering;

use crate::{
    async_context::{get_task_cx, set_task_env, GuestWaker},
    resource::{Resource, ResourceKeeper},
    tls::{load_tls_config, TlsStream},
    VmId,
};

mod wasi_env;

pub struct ShortId<'a>(pub &'a [u8]);

impl fmt::Display for ShortId<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let len = self.0.len();
        hex_fmt::HexFmt(&self.0[..len.min(6)]).fmt(f)
    }
}

// Let the compiler check IntPtr is 32bit sized.
fn _sizeof_i32_must_eq_to_intptr() {
    let _ = core::mem::transmute::<i32, IntPtr>;
}

pub fn create_env(id: VmId, store: &Store, cache_ops: DynCacheOps) -> (Env, ImportObject) {
    let env = Env::new(id, cache_ops);
    let wasi_imports = wasi_env::wasi_imports(store, &env);
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
                    env.clone(),
                    sidevm_ocall_fast_return,
                ),
            },
            "sidevm" => {
                "gas" => Function::new_native_with_env(
                    store,
                    env,
                    gas,
                ),
            },
            "wasi_snapshot_preview1" => wasi_imports,
        },
    )
}

pub(crate) struct TaskSet {
    awake_tasks: dashmap::DashSet<i32>,
    /// Guest waker ids that are ready to be woken up, or to be dropped if negative.
    pub(crate) awake_wakers: Mutex<VecDeque<i32>>,
}

impl TaskSet {
    fn with_task0() -> Self {
        let awake_tasks = dashmap::DashSet::new();
        awake_tasks.insert(0);
        Self {
            awake_tasks,
            awake_wakers: Default::default(),
        }
    }

    pub(crate) fn push_task(&self, task_id: i32) {
        self.awake_tasks.insert(task_id);
    }

    pub(crate) fn pop_task(&self) -> Option<i32> {
        let item = self.awake_tasks.iter().next().map(|task_id| *task_id);
        match item {
            Some(task_id) => {
                self.awake_tasks.remove(&task_id);
                Some(task_id)
            }
            None => None,
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.awake_tasks.is_empty() && self.awake_wakers.lock().unwrap().is_empty()
    }
}

pub trait CacheOps {
    fn get(&self, contract: &[u8], key: &[u8]) -> Result<Option<Vec<u8>>>;
    fn set(&self, contract: &[u8], key: &[u8], value: &[u8]) -> Result<()>;
    fn set_expiration(&self, contract: &[u8], key: &[u8], expire_after_secs: u64) -> Result<()>;
    fn remove(&self, contract: &[u8], key: &[u8]) -> Result<Option<Vec<u8>>>;
}

pub type DynCacheOps = &'static (dyn CacheOps + Send + Sync);

struct State {
    id: VmId,
    gas_per_breath: u64,
    resources: ResourceKeeper,
    temp_return_value: ThreadLocal<Cell<Option<Vec<u8>>>>,
    ocall_trace_enabled: bool,
    message_tx: Option<Sender<Vec<u8>>>,
    query_tx: Option<Sender<Vec<u8>>>,
    sys_message_tx: Option<Sender<Vec<u8>>>,
    awake_tasks: Arc<TaskSet>,
    current_task: i32,
    cache_ops: DynCacheOps,
    weight: u32,
    instance: Option<Instance>,
}

struct VmMemory(Option<Memory>);

pub(crate) struct EnvInner {
    memory: VmMemory,
    state: State,
}

impl VmMemory {
    pub(crate) fn unwrap_ref(&self) -> &Memory {
        self.0.as_ref().expect("memory is not initialized")
    }
}

#[derive(WasmerEnv, Clone)]
pub struct Env {
    pub(crate) inner: Arc<Mutex<EnvInner>>,
}

impl Env {
    fn new(id: VmId, cache_ops: DynCacheOps) -> Self {
        Self {
            inner: Arc::new(Mutex::new(EnvInner {
                memory: VmMemory(None),
                state: State {
                    id,
                    gas_per_breath: 0,
                    resources: Default::default(),
                    temp_return_value: Default::default(),
                    ocall_trace_enabled: false,
                    message_tx: None,
                    sys_message_tx: None,
                    query_tx: None,
                    awake_tasks: Arc::new(TaskSet::with_task0()),
                    current_task: 0,
                    cache_ops,
                    weight: 1,
                    instance: None,
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

    /// Push a pink message into the Sidevm instance.
    pub fn push_message(
        &self,
        message: Vec<u8>,
    ) -> Option<impl Future<Output = Result<(), SendError<Vec<u8>>>>> {
        let tx = self.inner.lock().unwrap().state.message_tx.clone()?;
        Some(async move { tx.send(message).await })
    }

    /// Push a pink system message into the Sidevm instance.
    pub fn push_system_message(
        &self,
        message: SystemMessage,
    ) -> Option<impl Future<Output = Result<(), SendError<Vec<u8>>>>> {
        let tx = self.inner.lock().unwrap().state.sys_message_tx.clone()?;
        Some(async move { tx.send(message.encode()).await })
    }

    /// Push a contract query to the Sidevm instance.
    pub fn push_query(
        &self,
        origin: Option<AccountId>,
        payload: Vec<u8>,
        reply_tx: OneshotSender<Vec<u8>>,
    ) -> Option<impl Future<Output = anyhow::Result<()>>> {
        let mut env_guard = self.inner.lock().unwrap();
        let tx = env_guard.state.query_tx.clone()?;
        let reply_tx = env_guard
            .state
            .resources
            .push(Resource::OneshotTx(Some(reply_tx)));
        let inner = self.inner.clone();
        Some(async move {
            let reply_tx = reply_tx?;
            let query = QueryRequest {
                origin,
                payload,
                reply_tx,
            };
            let result = tx.send(query.encode()).await;
            if result.is_err() {
                // FIXME: The channel doesn't guarantee the rx side would receive the message even
                // if the send returns an Ok. So the reply_tx could still leak in that case.
                let mut env_guard = inner.lock().unwrap();
                let _ = env_guard.state.close(reply_tx);
            }
            let _ = result?;
            Ok(())
        })
    }

    pub fn set_gas_per_breath(&self, gas: u64) {
        self.inner.lock().unwrap().state.gas_per_breath = gas;
    }

    pub fn reset_gas_to_breath(&self) {
        let guard = self.inner.lock().unwrap();
        let instance = guard
            .state
            .instance
            .as_ref()
            .expect("BUG: missing instance in env");
        metering::set_remaining_points(&instance, guard.state.gas_per_breath);
    }

    pub fn has_more_ready(&self) -> bool {
        !self.inner.lock().unwrap().state.awake_tasks.is_empty()
    }

    pub fn weight(&self) -> u32 {
        self.inner.lock().unwrap().state.weight
    }

    pub fn set_weight(&self, weight: u32) {
        self.inner.lock().unwrap().state.weight = weight;
    }

    pub fn set_instance(&self, instance: Instance) {
        self.inner.lock().unwrap().state.instance = Some(instance);
    }

    pub fn is_stifled(&self) -> bool {
        self.inner.lock().unwrap().state.is_stifled()
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
            None => Err(OcallError::NotFound),
            Some(_res) => Ok(()),
        }
    }

    fn poll(&mut self, waker_id: i32, resource_id: i32) -> Result<Vec<u8>> {
        self.resources.get_mut(resource_id)?.poll(waker_id)
    }

    fn poll_read(&mut self, waker_id: i32, resource_id: i32, data: &mut [u8]) -> Result<u32> {
        self.resources
            .get_mut(resource_id)?
            .poll_read(waker_id, data)
    }

    fn poll_write(&mut self, waker_id: i32, resource_id: i32, data: &[u8]) -> Result<u32> {
        self.resources
            .get_mut(resource_id)?
            .poll_write(waker_id, data)
    }

    fn poll_shutdown(&mut self, waker_id: i32, resource_id: i32) -> Result<()> {
        self.resources.get_mut(resource_id)?.poll_shutdown(waker_id)
    }

    fn poll_res(&mut self, waker_id: i32, resource_id: i32) -> Result<i32> {
        let res = self.resources.get_mut(resource_id)?.poll_res(waker_id)?;
        self.resources.push(res)
    }

    fn mark_task_ready(&mut self, task_id: i32) -> Result<()> {
        self.awake_tasks.push_task(task_id);
        Ok(())
    }

    fn next_ready_task(&mut self) -> Result<i32> {
        self.awake_tasks.pop_task().ok_or(OcallError::NotFound)
    }

    fn create_timer(&mut self, timeout: i32) -> Result<i32> {
        let sleep = tokio::time::sleep(Duration::from_millis(timeout as u64));
        self.resources.push(Resource::Sleep(Box::pin(sleep)))
    }

    fn enable_ocall_trace(&mut self, enable: bool) -> Result<()> {
        self.ocall_trace_enabled = enable;
        Ok(())
    }

    fn tcp_listen(&mut self, addr: Cow<str>, tls_config: Option<TlsServerConfig>) -> Result<i32> {
        let std_listener = std::net::TcpListener::bind(&*addr).or(Err(OcallError::IoError))?;
        std_listener
            .set_nonblocking(true)
            .or(Err(OcallError::IoError))?;
        let listener = TcpListener::from_std(std_listener).or(Err(OcallError::IoError))?;
        let tls_config = tls_config.map(load_tls_config).transpose()?.map(Arc::new);
        self.resources.push(Resource::TcpListener {
            listener,
            tls_config,
        })
    }

    fn tcp_accept(&mut self, waker_id: i32, tcp_res_id: i32) -> Result<(i32, String)> {
        let waker = GuestWaker::from_id(waker_id);
        let (res, remote_addr) = {
            let res = self.resources.get_mut(tcp_res_id)?;
            let (listener, tls_config) = match res {
                Resource::TcpListener {
                    listener,
                    tls_config,
                } => (listener, tls_config),
                _ => return Err(OcallError::UnsupportedOperation),
            };
            let (stream, addr) = match get_task_cx(waker, |ct| listener.poll_accept(ct)) {
                Pending => return Err(OcallError::Pending),
                Ready(result) => result.or(Err(OcallError::IoError))?,
            };
            let res = match tls_config {
                Some(tls_config) => {
                    Resource::TlsStream(TlsStream::accept(stream, tls_config.clone()))
                }
                None => Resource::TcpStream(stream),
            };
            (res, addr)
        };
        self.resources
            .push(res)
            .map(|res_id| (res_id, remote_addr.to_string()))
    }

    fn tcp_accept_no_addr(&mut self, waker_id: i32, resource_id: i32) -> Result<i32> {
        self.tcp_accept(waker_id, resource_id)
            .map(|(res_id, _)| res_id)
    }

    fn tcp_connect(&mut self, host: &str, port: u16) -> Result<i32> {
        if host.len() > 253 {
            return Err(OcallError::InvalidParameter);
        }
        let host = host.to_owned();
        let fut = async move { tcp_connect(&host, port).await };
        self.resources.push(Resource::TcpConnect(Box::pin(fut)))
    }

    fn tcp_connect_tls(&mut self, host: String, port: u16, config: TlsClientConfig) -> Result<i32> {
        if host.len() > 253 {
            return Err(OcallError::InvalidParameter);
        }
        let TlsClientConfig::V0 = config;
        let domain = host
            .as_str()
            .try_into()
            .or(Err(OcallError::InvalidParameter))?;
        let fut = async move {
            tcp_connect(&host, port)
                .await
                .map(move |stream| TlsStream::connect(domain, stream))
        };
        self.resources.push(Resource::TlsConnect(Box::pin(fut)))
    }

    fn log(&mut self, level: log::Level, message: &str) -> Result<()> {
        let task = self.current_task;
        let vm_id = ShortId(&self.id);
        log::log!(target: "sidevm", level, "[{vm_id}][tid={task:<3}] {message}");
        Ok(())
    }

    fn local_cache_get(&mut self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        self.cache_ops.get(&self.id[..], key)
    }

    fn local_cache_set(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        self.cache_ops.set(&self.id[..], key, value)
    }

    fn local_cache_set_expiration(&mut self, key: &[u8], expire_after_secs: u64) -> Result<()> {
        self.cache_ops
            .set_expiration(&self.id[..], key, expire_after_secs)
    }

    fn local_cache_remove(&mut self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        self.cache_ops.remove(&self.id[..], key)
    }

    fn awake_wakers(&mut self) -> Result<Vec<i32>> {
        Ok(self
            .awake_tasks
            .awake_wakers
            .lock()
            .unwrap()
            .drain(..)
            .collect())
    }

    fn getrandom(&mut self, buf: &mut [u8]) -> Result<()> {
        use rand::RngCore;
        const RANDOM_BYTE_WEIGHT: usize = 1_000_000;

        self.pay((RANDOM_BYTE_WEIGHT * buf.len()) as _)?;
        rand::thread_rng().fill_bytes(buf);
        Ok(())
    }

    fn oneshot_send(&mut self, resource_id: i32, data: &[u8]) -> Result<()> {
        let res = self.resources.get_mut(resource_id)?;
        match res {
            Resource::OneshotTx(sender) => match sender.take() {
                Some(sender) => sender.send(data.to_vec()).or(Err(OcallError::IoError))?,
                None => return Err(OcallError::IoError),
            },
            _ => return Err(OcallError::UnsupportedOperation),
        }
        Ok(())
    }

    fn create_input_channel(&mut self, ch: env::InputChannel) -> Result<i32> {
        use env::InputChannel::*;
        macro_rules! create_channel {
            ($field: expr) => {{
                if $field.is_some() {
                    return Err(OcallError::AlreadyExists);
                }
                let (tx, rx) = tokio::sync::mpsc::channel(20);
                let res = self.resources.push(Resource::ChannelRx(rx))?;
                $field = Some(tx);
                Ok(res)
            }};
        }
        match ch {
            GeneralMessage => create_channel!(self.message_tx),
            SystemMessage => create_channel!(self.sys_message_tx),
            Query => create_channel!(self.query_tx),
        }
    }

    fn gas_remaining(&mut self) -> Result<u8> {
        self.pay(1_000_000)?;
        Ok(if self.gas_per_breath == 0 {
            100
        } else {
            (self.gas_to_breath() * 100 / self.gas_per_breath) as u8
        })
    }

}

impl State {
    fn is_stifled(&mut self) -> bool {
        let instance = self.instance.as_ref().expect("BUG: instance is not set");
        match metering::get_remaining_points(&instance) {
            metering::MeteringPoints::Remaining(_) => false,
            metering::MeteringPoints::Exhausted => true,
        }
    }

    fn gas_to_breath(&self) -> u64 {
        let instance = self.instance.as_ref().expect("BUG: instance is not set");
        match metering::get_remaining_points(&instance) {
            metering::MeteringPoints::Remaining(v) => v,
            metering::MeteringPoints::Exhausted => 0,
        }
    }

    fn set_gas_to_breath(&self, gas: u64) {
        let instance = self.instance.as_ref().expect("BUG: instance is not set");
        metering::set_remaining_points(&instance, gas);
    }

    fn pay(&mut self, cost: u64) -> Result<(), OcallAborted> {
        let gas = self.gas_to_breath();
        if cost > gas {
            return Err(OcallAborted::Stifled);
        }
        self.set_gas_to_breath(gas - cost);
        Ok(())
    }
}

async fn tcp_connect(host: &str, port: u16) -> std::io::Result<tokio::net::TcpStream> {
    fn get_proxy(key: &str) -> Option<String> {
        std::env::var(key).ok().and_then(|uri| {
            if uri.trim().is_empty() {
                None
            } else {
                Some(uri)
            }
        })
    }

    let proxy_url = if host.ends_with(".i2p") {
        get_proxy("i2p_proxy")
    } else {
        None
    };

    if let Some(proxy_url) = proxy_url.or(get_proxy("all_proxy")) {
        tokio_proxy::connect((host, port), proxy_url).await
    } else {
        tokio::net::TcpStream::connect((host, port)).await
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
) -> Result<IntRet, OcallAborted> {
    let mut env = env.inner.lock().unwrap();
    let env = &mut *env;

    env.state.current_task = task_id;
    let result = set_task_env(env.state.awake_tasks.clone(), task_id, || {
        env::dispatch_call_fast_return(&mut env.state, &env.memory, func_id, p0, p1, p2, p3)
    });
    if env.state.ocall_trace_enabled {
        let func_name = env::ocall_id2name(func_id);
        let vm_id = ShortId(&env.state.id);
        log::trace!(
            target: "sidevm",
            "[{vm_id}][tid={task_id:<3}](F) {func_name}({p0}, {p1}, {p2}, {p3}) = {result:?}"
        );
    }
    convert(result)
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
) -> Result<IntRet, OcallAborted> {
    let mut env = env.inner.lock().unwrap();
    let env = &mut *env;

    env.state.current_task = task_id;
    let result = set_task_env(env.state.awake_tasks.clone(), task_id, || {
        env::dispatch_call(&mut env.state, &env.memory, func_id, p0, p1, p2, p3)
    });
    if env.state.ocall_trace_enabled {
        let func_name = env::ocall_id2name(func_id);
        let vm_id = ShortId(&env.state.id);
        log::trace!(
            target: "sidevm",
            "[{vm_id}][tid={task_id:<3}](S) {func_name}({p0}, {p1}, {p2}, {p3}) = {result:?}"
        );
    }
    convert(result)
}

fn convert(result: Result<i32, OcallError>) -> Result<IntRet, OcallAborted> {
    match result {
        Err(OcallError::GasExhausted) => Err(OcallAborted::GasExhausted),
        Err(OcallError::Stifled) => Err(OcallAborted::Stifled),
        _ => Ok(result.encode_ret()),
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OcallAborted {
    GasExhausted,
    Stifled,
}

impl From<OcallAborted> for OcallError {
    fn from(aborted: OcallAborted) -> Self {
        match aborted {
            OcallAborted::GasExhausted => OcallError::GasExhausted,
            OcallAborted::Stifled => OcallError::Stifled,
        }
    }
}

impl fmt::Display for OcallAborted {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OcallAborted::GasExhausted => write!(f, "Gas exhausted"),
            OcallAborted::Stifled => write!(f, "Stifled"),
        }
    }
}

impl std::error::Error for OcallAborted {}

fn gas(env: &Env, cost: u32) -> Result<(), OcallAborted> {
    let mut env = env.inner.lock().unwrap();
    env.state.pay(cost as _)
}

use std::{
    borrow::Cow,
    cell::Cell,
    collections::VecDeque,
    fmt,
    future::Future,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
    task::Poll::{Pending, Ready},
    time::Duration,
};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use tokio::{
    net::TcpListener,
    sync::{
        mpsc::{error::TrySendError, Receiver, Sender},
        oneshot,
    },
    sync::{oneshot::Sender as OneshotSender, Semaphore},
};
use tracing::{error, info, warn, Instrument, Span};
use wasmer::{
    self, imports, AsStoreMut, Function, FunctionEnv, FunctionEnvMut, Imports, Instance, Memory,
    Store, StoreMut,
};

use env::{
    messages::{AccountId, HttpRequest, HttpResponseHead, QueryRequest, SystemMessage},
    tls::{TlsClientConfig, TlsServerConfig},
    IntPtr, IntRet, OcallError, Result, RetEncode,
};
use scale::{Decode, Encode};
use sidevm_env as env;
use thread_local::ThreadLocal;
use wasmer_middlewares::metering;

use crate::{
    async_context::{get_task_cx, set_task_env, GuestWaker},
    resource::{Resource, ResourceKeeper, TcpListenerResource},
    tls::{load_tls_config, TlsStream},
    IncomingHttpRequest, VmId,
};

mod wasi_env;

pub struct FnEnvMut<'a, T> {
    store: StoreMut<'a>,
    inner: T,
}

impl<'a, T> Deref for FnEnvMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, T> DerefMut for FnEnvMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<'a, T> FnEnvMut<'a, T> {
    pub fn new(store: &'a mut impl AsStoreMut, value: T) -> Self {
        Self {
            store: store.as_store_mut(),
            inner: value,
        }
    }
}

pub struct ShortId<T>(pub T);

impl<T: AsRef<[u8]>> fmt::Display for ShortId<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let len = self.0.as_ref().len();
        hex_fmt::HexFmt(&self.0.as_ref()[..len.min(6)]).fmt(f)
    }
}

// Let the compiler check IntPtr is 32bit sized.
fn _sizeof_i32_must_eq_to_intptr() {
    let _ = core::mem::transmute::<i32, IntPtr>;
}

pub fn create_env(
    id: VmId,
    store: &mut Store,
    cache_ops: DynCacheOps,
    out_tx: OutgoingRequestChannel,
) -> (Env, Imports) {
    let raw_env = Env::new(id, cache_ops, out_tx);
    let env = FunctionEnv::new(store, raw_env.clone());
    let wasi_imports = wasi_env::wasi_imports(store, &env);
    (
        raw_env,
        imports! {
            "env" => {
                "sidevm_ocall" => Function::new_typed_with_env(
                    store,
                    &env,
                    sidevm_ocall,
                ),
                "sidevm_ocall_fast_return" => Function::new_typed_with_env(
                    store,
                    &env,
                    sidevm_ocall_fast_return,
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

pub type OutgoingRequestChannel = Sender<(VmId, OutgoingRequest)>;

pub enum OutgoingRequest {
    Query {
        contract_id: [u8; 32],
        payload: Vec<u8>,
        reply_tx: OneshotSender<Vec<u8>>,
    },
}

struct VmMemory(Option<Memory>);

pub(crate) struct EnvInner {
    memory: VmMemory,
    id: VmId,
    gas_per_breath: u64,
    resources: ResourceKeeper,
    temp_return_value: ThreadLocal<Cell<Option<Vec<u8>>>>,
    ocall_trace_enabled: bool,
    message_tx: Option<Sender<Vec<u8>>>,
    query_tx: Option<Sender<Vec<u8>>>,
    sys_message_tx: Option<Sender<Vec<u8>>>,
    http_connect_tx: Option<Sender<Vec<u8>>>,
    awake_tasks: Arc<TaskSet>,
    current_task: i32,
    cache_ops: DynCacheOps,
    weight: u32,
    instance: Option<Instance>,
    outgoing_query_guard: Arc<Semaphore>,
    outgoing_request_tx: OutgoingRequestChannel,
    _counter: vm_counter::Counter,
}

impl VmMemory {
    pub(crate) fn unwrap_ref(&self) -> &Memory {
        self.0.as_ref().expect("memory is not initialized")
    }
}

struct MemoryView<'a>(wasmer::MemoryView<'a>);

impl<'a> Deref for MemoryView<'a> {
    type Target = wasmer::MemoryView<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> MemoryView<'a> {
    fn check_addr(&self, offset: usize, len: usize) -> Result<(usize, usize)> {
        let end = offset.checked_add(len).ok_or(OcallError::InvalidAddress)?;
        if end > self.size().bytes().0 {
            return Err(OcallError::InvalidAddress);
        }
        Ok((offset, end))
    }
}

impl<'a> env::VmMemory for MemoryView<'a> {
    fn copy_to_vm(&self, data: &[u8], ptr: IntPtr) -> Result<()> {
        if data.len() > u32::MAX as usize {
            return Err(OcallError::NoMemory);
        }
        self.write(ptr as _, data)
            .or(Err(OcallError::InvalidAddress))?;
        Ok(())
    }

    fn slice_from_vm(&self, ptr: IntPtr, len: IntPtr) -> Result<&[u8]> {
        let (offset, end) = self.check_addr(ptr as _, len as _)?;
        let slice = unsafe { &self.data_unchecked()[offset..end] };
        Ok(slice)
    }

    fn slice_from_vm_mut(&self, ptr: IntPtr, len: IntPtr) -> Result<&mut [u8]> {
        let (offset, end) = self.check_addr(ptr as _, len as _)?;
        let slice = unsafe { &mut self.data_unchecked_mut()[offset..end] };
        Ok(slice)
    }
}

#[derive(Clone)]
pub struct Env {
    pub(crate) inner: Arc<Mutex<EnvInner>>,
}

impl Env {
    fn new(id: VmId, cache_ops: DynCacheOps, outgoing_request_tx: OutgoingRequestChannel) -> Self {
        Self {
            inner: Arc::new(Mutex::new(EnvInner {
                memory: VmMemory(None),
                id,
                gas_per_breath: 0,
                resources: Default::default(),
                temp_return_value: Default::default(),
                ocall_trace_enabled: false,
                message_tx: None,
                sys_message_tx: None,
                query_tx: None,
                http_connect_tx: None,
                awake_tasks: Arc::new(TaskSet::with_task0()),
                current_task: 0,
                cache_ops,
                weight: 1,
                instance: None,
                outgoing_query_guard: Arc::new(Semaphore::new(1)),
                outgoing_request_tx,
                _counter: Default::default(),
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
    pub fn push_message(&self, message: Vec<u8>) -> Option<Result<(), TrySendError<Vec<u8>>>> {
        let tx = self.inner.lock().unwrap().message_tx.clone()?;
        Some(tx.try_send(message))
    }

    /// Push a pink system message into the Sidevm instance.
    pub fn push_system_message(
        &self,
        message: SystemMessage,
    ) -> Option<Result<(), TrySendError<Vec<u8>>>> {
        let tx = self.inner.lock().unwrap().sys_message_tx.clone()?;
        Some(tx.try_send(message.encode()))
    }

    /// Push a contract query to the Sidevm instance.
    pub fn push_query(
        &self,
        origin: Option<AccountId>,
        payload: Vec<u8>,
        reply_tx: OneshotSender<Vec<u8>>,
    ) -> Option<impl Future<Output = anyhow::Result<()>>> {
        let mut env_guard = self.inner.lock().unwrap();
        let tx = env_guard.query_tx.clone()?;
        let reply_tx = env_guard
            .resources
            .push(Resource::OneshotTx(Some(reply_tx)));
        let inner = Arc::downgrade(&self.inner);
        Some(async move {
            let reply_tx = reply_tx?;
            let query = QueryRequest {
                origin,
                payload,
                reply_tx,
            };
            let result = tx.send(query.encode()).await;
            if result.is_err() {
                if let Some(inner) = inner.upgrade() {
                    let mut env_guard = inner.lock().unwrap();
                    let _ = env_guard.close(reply_tx);
                }
            }
            result?;
            Ok(())
        })
    }

    pub fn set_gas_per_breath(&self, gas: u64) {
        self.inner.lock().unwrap().gas_per_breath = gas;
    }

    pub fn reset_gas_to_breath(&self, store: &mut impl AsStoreMut) {
        let guard = self.inner.lock().unwrap();
        let instance = guard
            .instance
            .as_ref()
            .expect("BUG: missing instance in env");
        metering::set_remaining_points(store, instance, guard.gas_per_breath);
    }

    pub fn has_more_ready(&self) -> bool {
        !self.inner.lock().unwrap().awake_tasks.is_empty()
    }

    pub fn weight(&self) -> u32 {
        self.inner.lock().unwrap().weight
    }

    pub fn set_weight(&self, weight: u32) {
        let mut inner = self.inner.lock().unwrap();
        inner.weight = weight;
        tracing::debug!(target: "sidevm", weight, "Weight updated");
    }

    pub fn set_instance(&self, instance: Instance) {
        self.inner.lock().unwrap().instance = Some(instance);
    }

    pub fn is_stifled(&self, store: &mut impl AsStoreMut) -> bool {
        self.inner.lock().unwrap().is_stifled(store)
    }

    pub fn memory(&self) -> Memory {
        self.inner
            .lock()
            .unwrap()
            .memory
            .0
            .as_ref()
            .expect("BUG: missing memory in env")
            .clone()
    }

    /// Establish a incoming HTTP connection.
    pub fn push_http_request(
        &self,
        request: IncomingHttpRequest,
    ) -> Option<impl Future<Output = anyhow::Result<()>>> {
        let IncomingHttpRequest {
            head,
            body_stream,
            response_tx,
        } = request;
        let mut env_guard = self.inner.lock().unwrap();
        let connect_tx = env_guard.http_connect_tx.clone()?;
        let (reply_tx, reply_rx) = oneshot::channel();
        let reply_tx = env_guard
            .resources
            .push(Resource::OneshotTx(Some(reply_tx)));
        tokio::spawn(
            async move {
                let reply = reply_rx.await;
                let reply = reply
                    .context("Failed to receive http response")
                    .and_then(|bytes| {
                        let response = HttpResponseHead::decode(&mut &bytes[..])?;
                        Ok(response)
                    });
                if response_tx.send(reply).is_err() {
                    info!(target: "sidevm", "Failed to send http response");
                }
            }
            .instrument(Span::current()),
        );
        let body_stream = env_guard
            .resources
            .push(Resource::DuplexStream(body_stream));
        let inner = Arc::downgrade(&self.inner);
        Some(async move {
            let response_tx = reply_tx?;
            let body_stream = body_stream?;
            let query = HttpRequest {
                head,
                response_tx,
                io_stream: body_stream,
            };
            let result = connect_tx.send(query.encode()).await;
            if result.is_err() {
                if let Some(inner) = inner.upgrade() {
                    let mut env_guard = inner.lock().unwrap();
                    let _ = env_guard.close(response_tx);
                    let _ = env_guard.close(body_stream);
                }
            }
            result?;
            Ok(())
        })
    }
}

impl<'a, 'b> env::OcallEnv for FnEnvMut<'a, &'b mut EnvInner> {
    fn put_return(&mut self, rv: Vec<u8>) -> usize {
        let len = rv.len();
        self.temp_return_value.get_or_default().set(Some(rv));
        len
    }

    fn take_return(&mut self) -> Option<Vec<u8>> {
        self.temp_return_value.get_or_default().take()
    }
}

impl<'a, 'b> env::OcallFuncs for FnEnvMut<'a, &'b mut EnvInner> {
    fn close(&mut self, resource_id: i32) -> Result<()> {
        self.inner.close(resource_id)
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
        self.resources
            .push(Resource::TcpListener(Box::new(TcpListenerResource {
                listener,
                tls_config,
            })))
    }

    fn tcp_accept(&mut self, waker_id: i32, tcp_res_id: i32) -> Result<(i32, String)> {
        let waker = GuestWaker::from_id(waker_id);
        let (res, remote_addr) = {
            let res = self.resources.get_mut(tcp_res_id)?;
            let res = match res {
                Resource::TcpListener(res) => res,
                _ => return Err(OcallError::UnsupportedOperation),
            };
            let (stream, addr) = match get_task_cx(waker, |ct| res.listener.poll_accept(ct)) {
                Pending => return Err(OcallError::Pending),
                Ready(result) => result.or(Err(OcallError::IoError))?,
            };
            let res = match &res.tls_config {
                Some(tls_config) => {
                    Resource::TlsStream(Box::new(TlsStream::accept(stream, tls_config.clone())))
                }
                None => Resource::TcpStream(Box::new(stream)),
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
        log::log!(target: "sidevm", level, "{message}");
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

        self.inner
            .pay(&mut self.store, (RANDOM_BYTE_WEIGHT * buf.len()) as _)?;
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
            HttpRequest => create_channel!(self.http_connect_tx),
        }
    }

    fn gas_remaining(&mut self) -> Result<u8> {
        self.inner.pay(&mut self.store, 1_000_000)?;
        Ok(if self.gas_per_breath == 0 {
            100
        } else {
            (self.inner.gas_to_breath(&mut self.store) * 100 / self.gas_per_breath) as u8
        })
    }

    fn query_local_contract(&mut self, contract_id: [u8; 32], payload: Vec<u8>) -> Result<i32> {
        let sem = self
            .inner
            .outgoing_query_guard
            .clone()
            .try_acquire_owned()
            .or(Err(OcallError::ResourceLimited))?;
        let (res_tx, res_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(1);
        let res_id = self.resources.push(Resource::ChannelRx(res_rx))?;
        let (reply_tx, reply_rx) = oneshot::channel();
        let request = OutgoingRequest::Query {
            contract_id,
            payload,
            reply_tx,
        };
        let from = self.inner.id;
        self.inner
            .outgoing_request_tx
            .try_send((from, request))
            .or(Err(OcallError::IoError))?;
        tokio::spawn(async move {
            let _sem = sem;
            let result = match reply_rx.await {
                Ok(reply) => res_tx.send(reply).await,
                Err(_) => {
                    warn!(target: "sidevm", "Failed to receive query result");
                    res_tx.send(Vec::new()).await
                }
            };
            if result.is_err() {
                error!(target: "sidevm", "Failed to send query result");
            }
        });
        Ok(res_id)
    }
}

impl EnvInner {
    pub(crate) fn make_mut<'a, 'b>(
        &'a mut self,
        store: &'b mut impl AsStoreMut,
    ) -> FnEnvMut<'b, &'a mut Self> {
        FnEnvMut::new(store, self)
    }

    pub(crate) fn close(&mut self, resource_id: i32) -> Result<()> {
        match self.resources.take(resource_id) {
            None => Err(OcallError::NotFound),
            Some(_res) => Ok(()),
        }
    }

    fn is_stifled(&mut self, store: &mut impl AsStoreMut) -> bool {
        let instance = self.instance.as_ref().expect("BUG: instance is not set");
        match metering::get_remaining_points(store, instance) {
            metering::MeteringPoints::Remaining(_) => false,
            metering::MeteringPoints::Exhausted => true,
        }
    }

    fn gas_to_breath(&self, store: &mut impl AsStoreMut) -> u64 {
        let instance = self.instance.as_ref().expect("BUG: instance is not set");
        match metering::get_remaining_points(store, instance) {
            metering::MeteringPoints::Remaining(v) => v,
            metering::MeteringPoints::Exhausted => 0,
        }
    }

    fn set_gas_to_breath(&self, store: &mut impl AsStoreMut, gas: u64) {
        let instance = self.instance.as_ref().expect("BUG: instance is not set");
        metering::set_remaining_points(store, instance, gas);
    }

    fn pay(&mut self, store: &mut impl AsStoreMut, cost: u64) -> Result<(), OcallAborted> {
        let gas = self.gas_to_breath(store);
        if cost > gas {
            return Err(OcallAborted::Stifled);
        }
        self.set_gas_to_breath(store, gas - cost);
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

    if let Some(proxy_url) = proxy_url.or_else(|| get_proxy("all_proxy")) {
        tokio_proxy::connect((host, port), proxy_url).await
    } else {
        tokio::net::TcpStream::connect((host, port)).await
    }
}

fn sidevm_ocall_fast_return(
    func_env: FunctionEnvMut<Env>,
    task_id: i32,
    func_id: i32,
    p0: IntPtr,
    p1: IntPtr,
    p2: IntPtr,
    p3: IntPtr,
) -> Result<IntRet, OcallAborted> {
    do_ocall(func_env, task_id, func_id, p0, p1, p2, p3, true)
}

// Support all ocalls. Put the result into a temporary vec and wait for next fetch_result ocall to fetch the result.
#[allow(clippy::too_many_arguments)]
fn sidevm_ocall(
    func_env: FunctionEnvMut<Env>,
    task_id: i32,
    func_id: i32,
    p0: IntPtr,
    p1: IntPtr,
    p2: IntPtr,
    p3: IntPtr,
) -> Result<IntRet, OcallAborted> {
    do_ocall(func_env, task_id, func_id, p0, p1, p2, p3, false)
}

#[allow(clippy::too_many_arguments)]
#[tracing::instrument(name="ocall", fields(tid=task_id), skip_all)]
fn do_ocall(
    mut func_env: FunctionEnvMut<Env>,
    task_id: i32,
    func_id: i32,
    p0: IntPtr,
    p1: IntPtr,
    p2: IntPtr,
    p3: IntPtr,
    fast_return: bool,
) -> Result<IntRet, OcallAborted> {
    let inner = func_env.data().inner.clone();
    let mut guard = inner.lock().unwrap();
    let env = &mut *guard;

    env.current_task = task_id;
    let result = set_task_env(env.awake_tasks.clone(), task_id, || {
        let memory = env.memory.unwrap_ref().clone();
        let vm = MemoryView(memory.view(&func_env));

        // Safety:
        //
        // Started from wasmer 3.3, the lifetime of MemoryView is bound to the lifetime of the Store
        // rather than bound to the Memory as before. The behavior before was unsound because the
        // Memory contents could be changed. Especially when a grow operation happens, the Memory
        // could be reallocated and the old MemoryView would be invalid.
        //
        // Why we need to transmute the lifetime here?
        // To make the larger chunks of data passings between the host and the guest efficient, we
        // decided to directly pass the data chunks to ocall functions by reference instead of
        // copying them. For example, given a ocall defined as `fn tcp_read(fd: i32, data: &mut [u8])`,
        // the parameter `data` is a reference to the guest memory rather than copied to a owned vec.
        // Obviously, the lifetime of the reference is bound to the guest memory which stored in the
        // instance of Store.
        //
        // It would be safe only when the following conditions are met:
        //   1. The referred slice of guest memory is not changed by other codes during the ocall.
        //   2. The entire guest memory is not reallocated during the ocall function call.
        // Currently, we can guarantee both conditions are met by carefully implementing the ocall
        // functions and make sure no guest reentrancy happens during the ocall function call.
        unsafe fn translife<'b, T>(m: MemoryView<'_>, _l: &'b T) -> MemoryView<'b> {
            std::mem::transmute(m)
        }
        let vm = unsafe { translife(vm, &memory) };
        let mut state = env.make_mut(&mut func_env);
        env::dispatch_ocall(fast_return, &mut state, &vm, func_id, p0, p1, p2, p3)
    });

    if env.ocall_trace_enabled {
        let func_name = env::ocall_id2name(func_id);
        tracing::trace!(target: "sidevm", "{func_name}({p0}, {p1}, {p2}, {p3}) = {result:?}");
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

pub use vm_counter::vm_count;
mod vm_counter {
    use std::sync::atomic::{AtomicUsize, Ordering};

    pub fn vm_count() -> usize {
        Counter::current()
    }

    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    pub struct Counter(());
    impl Counter {
        pub fn current() -> usize {
            COUNTER.load(Ordering::Relaxed)
        }
    }
    impl Default for Counter {
        fn default() -> Self {
            COUNTER.fetch_add(1, Ordering::Relaxed);
            Self(())
        }
    }
    impl Drop for Counter {
        fn drop(&mut self) {
            COUNTER.fetch_sub(1, Ordering::Relaxed);
        }
    }
}

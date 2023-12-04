use anyhow::{Context as _, Result};
use phala_scheduler::TaskScheduler;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use wasmer::{BaseTunables, Engine, Instance, Module, Pages, RuntimeError, Store, TypedFunction};
#[cfg(feature = "wasmer-compiler-cranelift")]
use wasmer_compiler_cranelift::Cranelift;
#[cfg(feature = "wasmer-compiler-llvm")]
use wasmer_compiler_llvm::LLVM;
use wasmer_compiler_singlepass::Singlepass;
use wasmer_tunables::LimitingTunables;

use crate::env::{DynCacheOps, LogHandler};
use crate::{async_context, env, metering::metering, VmId};

#[derive(Clone)]
pub struct WasmModule {
    engine: WasmEngine,
    module: Module,
}

#[derive(Clone)]
pub struct WasmEngine {
    inner: Engine,
}

impl Default for WasmEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl WasmEngine {
    pub fn new() -> Self {
        let compiler_env = std::env::var("WASMER_COMPILER");
        let compiler_env = compiler_env
            .as_ref()
            .map(AsRef::as_ref)
            .unwrap_or("singlepass");
        let engine = match compiler_env {
            "singlepass" => metering(Singlepass::default()).into(),
            #[cfg(feature = "wasmer-compiler-cranelift")]
            "cranelift" => metering(Cranelift::default()).into(),
            #[cfg(feature = "wasmer-compiler-llvm")]
            "llvm" => LLVM::default().into(),
            _ => panic!("Unsupported compiler engine: {compiler_env}"),
        };
        Self { inner: engine }
    }

    pub fn compile(&self, wasm_code: &[u8]) -> Result<WasmModule> {
        Ok(WasmModule {
            engine: self.clone(),
            module: Module::new(&self.inner, wasm_code)?,
        })
    }
}

impl WasmModule {
    pub fn run(
        &self,
        args: Vec<String>,
        config: WasmInstanceConfig,
    ) -> Result<(WasmRun, env::Env)> {
        let WasmInstanceConfig {
            max_memory_pages,
            id,
            gas_per_breath,
            cache_ops,
            scheduler,
            weight,
            event_tx,
            log_handler,
        } = config;
        let base = BaseTunables {
            // Always use dynamic heap memory to save memory
            static_memory_bound: Pages(0),
            static_memory_offset_guard_size: 0,
            dynamic_memory_offset_guard_size: page_size::get() as _,
        };
        let tunables = LimitingTunables::new(base, Pages(max_memory_pages));
        let mut engine = self.engine.inner.clone();
        engine.set_tunables(tunables);
        let mut store = Store::new(engine);
        let (env, import_object) =
            env::create_env(id, &mut store, cache_ops, event_tx, log_handler, args);
        let instance = Instance::new(&mut store, &self.module, &import_object)?;
        let memory = instance
            .exports
            .get_memory("memory")
            .context("No memory exported")?;
        let wasm_poll_entry = instance.exports.get_typed_function(&store, "sidevm_poll")?;
        env.set_memory(memory.clone());
        env.set_instance(instance);
        env.set_gas_per_breath(gas_per_breath);
        env.set_weight(weight);
        if let Some(scheduler) = &scheduler {
            scheduler.reset(&id);
        }
        Ok((
            WasmRun {
                env: env.clone(),
                wasm_poll_entry,
                store,
                scheduler,
                id,
            },
            env,
        ))
    }
}

pub struct WasmInstanceConfig {
    pub max_memory_pages: u32,
    pub id: crate::VmId,
    pub gas_per_breath: u64,
    pub cache_ops: DynCacheOps,
    pub scheduler: Option<TaskScheduler<VmId>>,
    pub weight: u32,
    pub event_tx: crate::OutgoingRequestChannel,
    pub log_handler: Option<LogHandler>,
}

pub struct WasmRun {
    id: VmId,
    env: env::Env,
    store: Store,
    wasm_poll_entry: TypedFunction<(), i32>,
    scheduler: Option<TaskScheduler<VmId>>,
}

impl Drop for WasmRun {
    fn drop(&mut self) {
        self.env.cleanup();
        if let Some(scheduler) = &self.scheduler {
            scheduler.exit(&self.id);
        }
    }
}

impl Future for WasmRun {
    type Output = Result<i32, RuntimeError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let _guard = match &self.scheduler {
            Some(scheduler) => Some(futures::ready!(scheduler.poll_resume(
                cx,
                &self.id,
                self.env.weight()
            ))),
            None => None,
        };
        let run = self.get_mut();
        run.env.reset_gas_to_breath(&mut run.store);
        match async_context::set_task_cx(cx, || run.wasm_poll_entry.call(&mut run.store)) {
            Ok(rv) => {
                if rv == 0 {
                    if run.env.has_more_ready() {
                        cx.waker().wake_by_ref();
                    }
                    Poll::Pending
                } else {
                    Poll::Ready(Ok(rv))
                }
            }
            Err(err) => {
                if run.env.is_stifled(&mut run.store) {
                    Poll::Ready(Err(RuntimeError::user(
                        crate::env::OcallAborted::Stifled.into(),
                    )))
                } else {
                    Poll::Ready(Err(err))
                }
            }
        }
    }
}

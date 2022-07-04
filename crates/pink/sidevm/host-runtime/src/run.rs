use anyhow::{Context as _, Result};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use wasmer::{BaseTunables, Instance, Module, NativeFunc, Pages, RuntimeError, Store, Universal};
#[cfg(feature = "wasmer-compiler-cranelift")]
use wasmer_compiler_cranelift::Cranelift;
#[cfg(feature = "wasmer-compiler-llvm")]
use wasmer_compiler_llvm::LLVM;
use wasmer_compiler_singlepass::Singlepass;
use wasmer_tunables::LimitingTunables;

use crate::env::DynCacheOps;
use crate::{async_context, env};

pub struct WasmRun {
    env: env::Env,
    wasm_poll_entry: NativeFunc<(), i32>,
    gas_per_breath: u128,
}

impl Drop for WasmRun {
    fn drop(&mut self) {
        self.env.cleanup()
    }
}

impl WasmRun {
    pub fn run(
        code: &[u8],
        max_pages: u32,
        id: crate::VmId,
        gas_per_breath: u128,
        cache_ops: DynCacheOps,
    ) -> Result<(WasmRun, env::Env)> {
        let compiler_env = std::env::var("WASMER_COMPILER");
        let compiler_env = compiler_env
            .as_ref()
            .map(AsRef::as_ref)
            .unwrap_or("singlepass");

        let engine = match compiler_env {
            "singlepass" => Universal::new(Singlepass::default()),
            #[cfg(feature = "wasmer-compiler-cranelift")]
            "cranelift" => Universal::new(Cranelift::default()),
            #[cfg(feature = "wasmer-compiler-llvm")]
            "llvm" => Universal::new(LLVM::default()),
            _ => panic!("Unsupported compiler engine: {}", compiler_env),
        }
        .engine();
        let base = BaseTunables {
            // Always use dynamic heap memory to save memory
            static_memory_bound: Pages(0),
            static_memory_offset_guard_size: 0,
            dynamic_memory_offset_guard_size: page_size::get() as _,
        };
        let tunables = LimitingTunables::new(base, Pages(max_pages));
        let store = Store::new_with_tunables(&engine, tunables);
        let module = Module::new(&store, code)?;
        let (env, import_object) = env::create_env(id, &store, cache_ops);
        let instance = Instance::new(&module, &import_object)?;
        let memory = instance
            .exports
            .get_memory("memory")
            .context("No memory exported")?;
        env.set_memory(memory.clone());
        Ok((
            WasmRun {
                env: env.clone(),
                wasm_poll_entry: instance.exports.get_native_function("sidevm_poll")?,
                gas_per_breath,
            },
            env,
        ))
    }
}

impl Future for WasmRun {
    type Output = Result<i32, RuntimeError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.env.set_gas_to_breath(self.gas_per_breath);
        match async_context::set_task_cx(cx, || self.wasm_poll_entry.call()) {
            Ok(rv) => {
                if rv == 0 {
                    Poll::Pending
                } else {
                    Poll::Ready(Ok(rv))
                }
            }
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

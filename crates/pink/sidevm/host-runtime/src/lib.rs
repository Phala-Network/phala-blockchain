use anyhow::{Context as _, Result};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use wasmer::{BaseTunables, Instance, Module, NativeFunc, Pages, RuntimeError, Store, Universal};
use wasmer_compiler_singlepass::Singlepass;
use wasmer_tunables::LimitingTunables;

mod async_context;
mod env;
mod resource;

pub struct WasmRun {
    env: env::Env,
    wasm_poll_entry: NativeFunc<(), i32>,
}

impl Drop for WasmRun {
    fn drop(&mut self) {
        self.env.cleanup()
    }
}

impl WasmRun {
    pub fn run(code: &[u8], max_pages: u32) -> Result<(WasmRun, env::Env)> {
        let compiler = Singlepass::default();
        let engine = Universal::new(compiler).engine();
        let base = BaseTunables {
            static_memory_bound: Pages(0x10),
            static_memory_offset_guard_size: 0x1000,
            dynamic_memory_offset_guard_size: 0x1000,
        };
        let tunables = LimitingTunables::new(base, Pages(max_pages));
        let store = Store::new_with_tunables(&engine, tunables);
        let module = Module::new(&store, code)?;
        let (env, import_object) = env::create_env(&store);
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
            },
            env,
        ))
    }
}

impl Future for WasmRun {
    type Output = Result<i32, RuntimeError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
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

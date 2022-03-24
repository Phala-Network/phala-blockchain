use anyhow::Result;
use resource::ResourceKeeper;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use wasmer::{
    BaseTunables, Function, Instance, Module, NativeFunc, Pages, RuntimeError, Store, Universal,
};
use wasmer_compiler_singlepass::Singlepass;
use wasmer_tunables::LimitingTunables;

mod async_context;
mod env;
mod resource;

pub struct WasmRun {
    wasm_poll_entry: NativeFunc<(), i32>,
}

impl WasmRun {
    pub fn run(code: &[u8], max_pages: u32) -> Result<WasmRun> {
        let compiler = Singlepass::default();
        let engine = Universal::new(compiler).engine();
        let base = BaseTunables::for_target(&Default::default());
        let tunables = LimitingTunables::new(base, Pages(max_pages));
        let store = Store::new_with_tunables(&engine, tunables);
        let module = Module::new(&store, code)?;
        let instance = Instance::new(&module, &env::imports(&store))?;
        Ok(WasmRun {
            wasm_poll_entry: instance.exports.get_native_function("sidevm_poll")?,
        })
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

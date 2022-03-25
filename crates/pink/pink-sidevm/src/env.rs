use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::Poll,
    time::Duration,
};

use log::error;
use wasmer::{imports, Function, ImportObject, Memory, MemoryType, Pages, Store, WasmerEnv};

use crate::async_context::poll_in_task_cx;
use crate::resource::{Resource, ResourceKeeper};

fn no_memory() -> Memory {
    let store = Store::default();
    Memory::new(&store, MemoryType::new(Pages(0), Some(Pages(0)), false))
        .expect("Create memory failed")
}

pub fn create_env(store: &Store) -> (Env, ImportObject) {
    let env = Env::new(store);
    (
        env.clone(),
        imports! {
            "env" => {
                "sidevm_ocall" => Function::new_native_with_env(
                    store,
                    env,
                    sidevm_ocall
                ),
            }
        },
    )
}

struct EnvInner {
    resources: ResourceKeeper,
    memory: Memory,
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
                memory: no_memory(),
            })),
        }
    }

    pub fn set_memory(&self, memory: Memory) {
        self.inner.lock().unwrap().memory = memory;
    }

    pub fn cleanup(&self) {
        // Cut up the reference cycle to avoid leaks.
        self.set_memory(no_memory());
    }
}

fn sidevm_ocall(env: &Env, func_id: i32, p0: i32, p1: i32, p2: i32, p3: i32) -> i32 {
    match func_id {
        // close
        0 => {
            // TODO.
            0
        }
        // create a sleep timer
        1 => {
            let sleep = tokio::time::sleep(Duration::from_millis(p0 as u64));
            let result = env
                .inner
                .lock()
                .unwrap()
                .resources
                .push(Resource::Sleep(Box::pin(sleep)));
            match result {
                Some(id) => id as i32,
                None => {
                    error!("failed to push sleep resource");
                    -1
                }
            }
        }
        // poll given sleep timer
        2 => {
            let id = p0 as usize;
            let mut res = env.inner.lock().unwrap();
            match res.resources.get_mut(id) {
                Some(res) => {
                    if let Resource::Sleep(sleep) = res {
                        match poll_in_task_cx(sleep.as_mut()) {
                            Poll::Ready(()) => 1,
                            Poll::Pending => 0,
                        }
                    } else {
                        -1
                    }
                }
                None => {
                    error!("poll_sleep: invalid resource id: {}", id);
                    -1
                }
            }
        }
        _ => {
            error!("unknown ocall function id: {}", func_id);
            -1
        }
    }
}

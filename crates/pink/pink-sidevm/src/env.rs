use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::Poll,
    time::Duration,
};

use log::error;
use wasmer::{imports, Function, ImportObject, Store, WasmerEnv};

use crate::async_context::poll_in_task_cx;
use crate::resource::{Resource, ResourceKeeper};

pub fn imports(store: &Store) -> ImportObject {
    imports! {
        "env" => {
            "sidevm_ocall" => Function::new_native_with_env(
                store,
                Env::new(),
                sidevm_ocall
            ),
        }
    }
}

#[derive(WasmerEnv, Clone)]
struct Env {
    resources: Arc<Mutex<ResourceKeeper>>,
}

impl Env {
    fn new() -> Self {
        Self {
            resources: Arc::new(Mutex::new(ResourceKeeper::default())),
        }
    }
}

fn sidevm_ocall(env: &Env, func_id: i32, p0: i32, p1: i32, p2: i32, p3: i32) -> i32 {
    match func_id {
        // sleep
        1 => {
            let sleep = tokio::time::sleep(Duration::from_millis(p0 as u64));
            let result = env
                .resources
                .lock()
                .unwrap()
                .push(Resource::Sleep(Box::pin(sleep)));
            match result {
                Some(id) => id as i32,
                None => {
                    error!("failed to push sleep resource");
                    -1
                }
            }
        }
        // poll_sleep
        2 => {
            let id = p0 as usize;
            let mut res = env.resources.lock().unwrap();
            match res.get_mut(id) {
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

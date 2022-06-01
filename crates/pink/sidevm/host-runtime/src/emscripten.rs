use anyhow::{anyhow, Result};
use log::info;
use sidevm_emscripten::{
    generate_emscripten_env, imports, EmEnv, EmscriptenGlobals, Exited, ImportObject,
};
use wasmer::{Function, Memory, Module, Store, WasmPtr};

type ptr = i32;
type __wasi_fd_t = u32;
type __wasi_errno_t = u16;
type __wasi_exitcode_t = u32;
pub const __WASI_EBADF: u16 = 8;

fn fd_close(fd: __wasi_fd_t) -> __wasi_errno_t {
    return __WASI_EBADF;
}

fn fd_read(fd: __wasi_fd_t, iovs: ptr, iovs_len: u32, nread: ptr) -> __wasi_errno_t {
    return __WASI_EBADF;
}

fn fd_write(fd: __wasi_fd_t, iovs: ptr, iovs_len: u32, nwritten: ptr) -> __wasi_errno_t {
    return __WASI_EBADF;
}

fn environ_sizes_get(
    environ_count: WasmPtr<u32>,
    environ_buf_size: WasmPtr<u32>,
) -> __wasi_errno_t {
    return __WASI_EBADF;
}

fn environ_get(environ: WasmPtr<u8>, environ_buf: WasmPtr<u8>) -> __wasi_errno_t {
    return __WASI_EBADF;
}

fn proc_exit(code: __wasi_exitcode_t) -> Result<(), Exited> {
    info!("wasi::proc_exit, {}", code);
    Err(Exited(code as _))
}

fn wasi_imports(store: &Store) -> ImportObject {
    imports! {
        "wasi_snapshot_preview1" => {
            "fd_close" => Function::new_native(store, fd_close),
            "fd_write" => Function::new_native(store, fd_write),
            "fd_read" => Function::new_native(store, fd_read),
            "environ_sizes_get" => Function::new_native(store, environ_sizes_get),
            "environ_get" => Function::new_native(store, environ_get),
            "proc_exit" => Function::new_native(store, proc_exit),
        },
    }
}

pub fn merge_imports(from: ImportObject, into: &mut ImportObject) {
    for (key, value) in from.into_iter() {
        if let Some(module) = into.get_mut(&key) {
            module.extend(value);
        } else {
            into.insert(key, value);
        }
    }
}

pub fn create_env(module: &Module) -> Result<(Memory, ImportObject)> {
    let mut emscripten_globals =
        EmscriptenGlobals::new(module.store(), &module).map_err(|e| anyhow!("{}", e))?;
    let mut em_env = EmEnv::new(&emscripten_globals.data, Default::default());
    let memory = emscripten_globals.memory.clone();
    let mut imports = generate_emscripten_env(module.store(), &mut emscripten_globals, &mut em_env);
    merge_imports(wasi_imports(module.store()), &mut imports);
    Ok((memory, imports))
}

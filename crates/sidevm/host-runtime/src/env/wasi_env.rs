use super::{Env as WasiEnv, Result};
use libc::{
    clock_getres, clock_gettime, timespec, CLOCK_MONOTONIC, CLOCK_PROCESS_CPUTIME_ID,
    CLOCK_REALTIME, CLOCK_THREAD_CPUTIME_ID,
};
use sidevm_env::{OcallError, OcallFuncs};
use thiserror::Error;
use wasmer::{
    namespace, AsStoreMut, Exports, Function, FunctionEnv, FunctionEnvMut, Memory32, WasmPtr,
};
use wasmer_wasi_types::*;

/// This is returned in `RuntimeError`.
/// Use `downcast` or `downcast_ref` to retrieve the `ExitCode`.
#[derive(Error, Debug)]
pub enum WasiError {
    #[error("WASI exited with code: {0}")]
    Exited(__wasi_exitcode_t),
}

macro_rules! wasi_try {
    ($expr:expr) => {{
        let res: Result<_, __wasi_errno_t> = $expr;
        match res {
            Ok(val) => val,
            Err(err) => {
                return err;
            }
        }
    }};
    ($expr:expr, $e:expr) => {{
        let opt: Option<_> = $expr;
        wasi_try!(opt.ok_or($e))
    }};
}

pub(crate) fn wasi_imports(store: &mut impl AsStoreMut, env: &FunctionEnv<WasiEnv>) -> Exports {
    namespace! {
        "args_get" => Function::new_typed_with_env(store, env, args_get),
        "args_sizes_get" => Function::new_typed_with_env(store, env, args_sizes_get),
        "clock_res_get" => Function::new_typed_with_env(store, env, clock_res_get),
        "clock_time_get" => Function::new_typed_with_env(store, env, clock_time_get),
        "environ_get" => Function::new_typed_with_env(store, env, environ_get),
        "environ_sizes_get" => Function::new_typed_with_env(store, env, environ_sizes_get),
        "fd_advise" => Function::new_typed_with_env(store, env, fd_advise),
        "fd_allocate" => Function::new_typed_with_env(store, env, fd_allocate),
        "fd_close" => Function::new_typed_with_env(store, env, fd_close),
        "fd_datasync" => Function::new_typed_with_env(store, env, fd_datasync),
        "fd_fdstat_get" => Function::new_typed_with_env(store, env, fd_fdstat_get),
        "fd_fdstat_set_flags" => Function::new_typed_with_env(store, env, fd_fdstat_set_flags),
        "fd_fdstat_set_rights" => Function::new_typed_with_env(store, env, fd_fdstat_set_rights),
        "fd_filestat_get" => Function::new_typed_with_env(store, env, fd_filestat_get),
        "fd_filestat_set_size" => Function::new_typed_with_env(store, env, fd_filestat_set_size),
        "fd_filestat_set_times" => Function::new_typed_with_env(store, env, fd_filestat_set_times),
        "fd_pread" => Function::new_typed_with_env(store, env, fd_pread),
        "fd_prestat_get" => Function::new_typed_with_env(store, env, fd_prestat_get),
        "fd_prestat_dir_name" => Function::new_typed_with_env(store, env, fd_prestat_dir_name),
        "fd_pwrite" => Function::new_typed_with_env(store, env, fd_pwrite),
        "fd_read" => Function::new_typed_with_env(store, env, fd_read),
        "fd_readdir" => Function::new_typed_with_env(store, env, fd_readdir),
        "fd_renumber" => Function::new_typed_with_env(store, env, fd_renumber),
        "fd_seek" => Function::new_typed_with_env(store, env, fd_seek),
        "fd_sync" => Function::new_typed_with_env(store, env, fd_sync),
        "fd_tell" => Function::new_typed_with_env(store, env, fd_tell),
        "fd_write" => Function::new_typed_with_env(store, env, fd_write),
        "path_create_directory" => Function::new_typed_with_env(store, env, path_create_directory),
        "path_filestat_get" => Function::new_typed_with_env(store, env, path_filestat_get),
        "path_filestat_set_times" => Function::new_typed_with_env(store, env, path_filestat_set_times),
        "path_link" => Function::new_typed_with_env(store, env, path_link),
        "path_open" => Function::new_typed_with_env(store, env, path_open),
        "path_readlink" => Function::new_typed_with_env(store, env, path_readlink),
        "path_remove_directory" => Function::new_typed_with_env(store, env, path_remove_directory),
        "path_rename" => Function::new_typed_with_env(store, env, path_rename),
        "path_symlink" => Function::new_typed_with_env(store, env, path_symlink),
        "path_unlink_file" => Function::new_typed_with_env(store, env, path_unlink_file),
        "poll_oneoff" => Function::new_typed_with_env(store, env, poll_oneoff),
        "proc_exit" => Function::new_typed_with_env(store, env, proc_exit),
        "proc_raise" => Function::new_typed_with_env(store, env, proc_raise),
        "random_get" => Function::new_typed_with_env(store, env, random_get),
        "sched_yield" => Function::new_typed_with_env(store, env, sched_yield),
        "sock_recv" => Function::new_typed_with_env(store, env, sock_recv),
        "sock_send" => Function::new_typed_with_env(store, env, sock_send),
        "sock_shutdown" => Function::new_typed_with_env(store, env, sock_shutdown),
    }
}

pub fn args_get(
    _env: FunctionEnvMut<WasiEnv>,
    _argv: WasmPtr<WasmPtr<u8>>,
    _argv_buf: WasmPtr<u8>,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn args_sizes_get(
    _env: FunctionEnvMut<WasiEnv>,
    _argc: WasmPtr<u32>,
    _argv_buf_size: WasmPtr<u32>,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn clock_res_get(
    env: FunctionEnvMut<WasiEnv>,
    clock_id: __wasi_clockid_t,
    resolution: WasmPtr<__wasi_timestamp_t>,
) -> __wasi_errno_t {
    let unix_clock_id = match clock_id {
        __WASI_CLOCK_MONOTONIC => CLOCK_MONOTONIC,
        __WASI_CLOCK_PROCESS_CPUTIME_ID => CLOCK_PROCESS_CPUTIME_ID,
        __WASI_CLOCK_REALTIME => CLOCK_REALTIME,
        __WASI_CLOCK_THREAD_CPUTIME_ID => CLOCK_THREAD_CPUTIME_ID,
        _ => return __WASI_EINVAL,
    };

    let (_output, timespec_out) = unsafe {
        let mut timespec_out: timespec = timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        (clock_getres(unix_clock_id, &mut timespec_out), timespec_out)
    };

    let t_out = (timespec_out.tv_sec * 1_000_000_000).wrapping_add(timespec_out.tv_nsec);

    let guard = env.data().inner.lock().unwrap();
    let memory = guard.memory.unwrap_ref().view(&env);
    let resolution = resolution.deref(&memory);
    wasi_try!(
        resolution.write(t_out as __wasi_timestamp_t).ok(),
        __WASI_EACCES
    );

    __WASI_ESUCCESS
}

pub fn clock_time_get(
    env: FunctionEnvMut<WasiEnv>,
    clock_id: __wasi_clockid_t,
    _precision: __wasi_timestamp_t,
    time: WasmPtr<__wasi_timestamp_t>,
) -> __wasi_errno_t {
    let unix_clock_id = match clock_id {
        __WASI_CLOCK_MONOTONIC => CLOCK_MONOTONIC,
        __WASI_CLOCK_PROCESS_CPUTIME_ID => CLOCK_PROCESS_CPUTIME_ID,
        __WASI_CLOCK_REALTIME => CLOCK_REALTIME,
        __WASI_CLOCK_THREAD_CPUTIME_ID => CLOCK_THREAD_CPUTIME_ID,
        _ => return __WASI_EINVAL,
    };

    let (_output, timespec_out) = unsafe {
        let mut timespec_out: timespec = timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        (
            clock_gettime(unix_clock_id, &mut timespec_out),
            timespec_out,
        )
    };

    let t_out = (timespec_out.tv_sec * 1_000_000_000).wrapping_add(timespec_out.tv_nsec);

    let guard = env.data().inner.lock().unwrap();
    let memory = guard.memory.unwrap_ref().view(&env);
    let time = time.deref(&memory);
    wasi_try!(time.write(t_out as __wasi_timestamp_t).ok(), __WASI_EACCES);

    __WASI_ESUCCESS
}

pub fn environ_get(
    _env: FunctionEnvMut<WasiEnv>,
    _environ: WasmPtr<WasmPtr<u8>>,
    _environ_buf: WasmPtr<u8>,
) -> __wasi_errno_t {
    __WASI_ESUCCESS
}

pub fn environ_sizes_get(
    env: FunctionEnvMut<WasiEnv>,
    environ_count: WasmPtr<u32>,
    environ_buf_size: WasmPtr<u32>,
) -> __wasi_errno_t {
    let guard = env.data().inner.lock().unwrap();
    let memory = guard.memory.unwrap_ref().view(&env);
    let environ_count = environ_count.deref(&memory);
    let environ_buf_size = environ_buf_size.deref(&memory);
    wasi_try!(environ_count.write(0).ok(), __WASI_EACCES);
    wasi_try!(environ_buf_size.write(0).ok(), __WASI_EACCES);
    __WASI_ESUCCESS
}

pub fn fd_advise(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: __wasi_fd_t,
    _offset: __wasi_filesize_t,
    _len: __wasi_filesize_t,
    _advice: __wasi_advice_t,
) -> __wasi_errno_t {
    __WASI_ESUCCESS
}

pub fn fd_allocate(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: __wasi_fd_t,
    _offset: __wasi_filesize_t,
    _len: __wasi_filesize_t,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn fd_close(_env: FunctionEnvMut<WasiEnv>, _fd: __wasi_fd_t) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn fd_datasync(_env: FunctionEnvMut<WasiEnv>, _fd: __wasi_fd_t) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn fd_fdstat_get(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: __wasi_fd_t,
    _buf_ptr: WasmPtr<__wasi_fdstat_t>,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn fd_fdstat_set_flags(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: __wasi_fd_t,
    _flags: __wasi_fdflags_t,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn fd_fdstat_set_rights(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: __wasi_fd_t,
    _fs_rights_base: __wasi_rights_t,
    _fs_rights_inheriting: __wasi_rights_t,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn fd_filestat_get(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: __wasi_fd_t,
    _buf: WasmPtr<__wasi_filestat_t>,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn fd_filestat_set_size(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: __wasi_fd_t,
    _st_size: __wasi_filesize_t,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn fd_filestat_set_times(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: __wasi_fd_t,
    _st_atim: __wasi_timestamp_t,
    _st_mtim: __wasi_timestamp_t,
    _fst_flags: __wasi_fstflags_t,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn fd_pread(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: __wasi_fd_t,
    _iovs: WasmPtr<__wasi_iovec_t<Memory32>>,
    _iovs_len: u32,
    _offset: __wasi_filesize_t,
    _nread: WasmPtr<u32>,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn fd_prestat_get(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: __wasi_fd_t,
    _buf: WasmPtr<__wasi_prestat_t>,
) -> __wasi_errno_t {
    __WASI_EBADF
}

pub fn fd_prestat_dir_name(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: __wasi_fd_t,
    _path: WasmPtr<u8>,
    _path_len: u32,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn fd_pwrite(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: __wasi_fd_t,
    _iovs: WasmPtr<__wasi_ciovec_t<Memory32>>,
    _iovs_len: u32,
    _offset: __wasi_filesize_t,
    _nwritten: WasmPtr<u32>,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn fd_read(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: __wasi_fd_t,
    _iovs: WasmPtr<__wasi_iovec_t<Memory32>>,
    _iovs_len: u32,
    _nread: WasmPtr<u32>,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn fd_readdir(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: __wasi_fd_t,
    _buf: WasmPtr<u8>,
    _buf_len: u32,
    _cookie: __wasi_dircookie_t,
    _bufused: WasmPtr<u32>,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn fd_renumber(
    _env: FunctionEnvMut<WasiEnv>,
    _from: __wasi_fd_t,
    _to: __wasi_fd_t,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn fd_seek(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: __wasi_fd_t,
    _offset: __wasi_filedelta_t,
    _whence: __wasi_whence_t,
    _newoffset: WasmPtr<__wasi_filesize_t>,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn fd_sync(_env: FunctionEnvMut<WasiEnv>, _fd: __wasi_fd_t) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn fd_tell(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: __wasi_fd_t,
    _offset: WasmPtr<__wasi_filesize_t>,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn fd_write(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: __wasi_fd_t,
    _iovs: WasmPtr<__wasi_ciovec_t<Memory32>>,
    _iovs_len: u32,
    _nwritten: WasmPtr<u32>,
) -> __wasi_errno_t {
    __WASI_ESUCCESS
}

pub fn path_create_directory(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: __wasi_fd_t,
    _path: WasmPtr<u8>,
    _path_len: u32,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn path_filestat_get(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: __wasi_fd_t,
    _flags: __wasi_lookupflags_t,
    _path: WasmPtr<u8>,
    _path_len: u32,
    _buf: WasmPtr<__wasi_filestat_t>,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn path_filestat_set_times(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: __wasi_fd_t,
    _flags: __wasi_lookupflags_t,
    _path: WasmPtr<u8>,
    _path_len: u32,
    _st_atim: __wasi_timestamp_t,
    _st_mtim: __wasi_timestamp_t,
    _fst_flags: __wasi_fstflags_t,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn path_link(
    _env: FunctionEnvMut<WasiEnv>,
    _old_fd: __wasi_fd_t,
    _old_flags: __wasi_lookupflags_t,
    _old_path: WasmPtr<u8>,
    _old_path_len: u32,
    _new_fd: __wasi_fd_t,
    _new_path: WasmPtr<u8>,
    _new_path_len: u32,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn path_open(
    _env: FunctionEnvMut<WasiEnv>,
    _dirfd: __wasi_fd_t,
    _dirflags: __wasi_lookupflags_t,
    _path: WasmPtr<u8>,
    _path_len: u32,
    _o_flags: __wasi_oflags_t,
    _fs_rights_base: __wasi_rights_t,
    _fs_rights_inheriting: __wasi_rights_t,
    _fs_flags: __wasi_fdflags_t,
    _fd: WasmPtr<__wasi_fd_t>,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn path_readlink(
    _env: FunctionEnvMut<WasiEnv>,
    _dir_fd: __wasi_fd_t,
    _path: WasmPtr<u8>,
    _path_len: u32,
    _buf: WasmPtr<u8>,
    _buf_len: u32,
    _buf_used: WasmPtr<u32>,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn path_remove_directory(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: __wasi_fd_t,
    _path: WasmPtr<u8>,
    _path_len: u32,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn path_rename(
    _env: FunctionEnvMut<WasiEnv>,
    _old_fd: __wasi_fd_t,
    _old_path: WasmPtr<u8>,
    _old_path_len: u32,
    _new_fd: __wasi_fd_t,
    _new_path: WasmPtr<u8>,
    _new_path_len: u32,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn path_symlink(
    _env: FunctionEnvMut<WasiEnv>,
    _old_path: WasmPtr<u8>,
    _old_path_len: u32,
    _fd: __wasi_fd_t,
    _new_path: WasmPtr<u8>,
    _new_path_len: u32,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn path_unlink_file(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: __wasi_fd_t,
    _path: WasmPtr<u8>,
    _path_len: u32,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn poll_oneoff(
    _env: FunctionEnvMut<WasiEnv>,
    _in_: WasmPtr<__wasi_subscription_t>,
    _out_: WasmPtr<__wasi_event_t>,
    _nsubscriptions: u32,
    _nevents: WasmPtr<u32>,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn proc_exit(_env: FunctionEnvMut<WasiEnv>, code: __wasi_exitcode_t) -> Result<(), WasiError> {
    Err(WasiError::Exited(code))
}

pub fn proc_raise(_env: FunctionEnvMut<WasiEnv>, _sig: __wasi_signal_t) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn random_get(
    mut env: FunctionEnvMut<WasiEnv>,
    buf: u32,
    buf_len: u32,
) -> Result<__wasi_errno_t> {
    let inner = env.data().inner.clone();
    let mut env_guard = inner.lock().unwrap();

    let inner = &mut *env_guard;
    let mut u8_buffer = vec![0; buf_len as usize];
    inner.make_mut(&mut env).getrandom(&mut u8_buffer)?;
    inner
        .memory
        .unwrap_ref()
        .view(&env)
        .write(buf as _, &u8_buffer)
        .or(Err(OcallError::InvalidAddress))?;
    Ok(__WASI_ESUCCESS)
}

pub fn sched_yield(_env: FunctionEnvMut<WasiEnv>) -> __wasi_errno_t {
    __WASI_ESUCCESS
}

pub fn sock_recv(
    _env: FunctionEnvMut<WasiEnv>,
    _sock: __wasi_fd_t,
    _ri_data: WasmPtr<__wasi_iovec_t<Memory32>>,
    _ri_data_len: u32,
    _ri_flags: __wasi_riflags_t,
    _ro_datalen: WasmPtr<u32>,
    _ro_flags: WasmPtr<__wasi_roflags_t>,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn sock_send(
    _env: FunctionEnvMut<WasiEnv>,
    _sock: __wasi_fd_t,
    _si_data: WasmPtr<__wasi_ciovec_t<Memory32>>,
    _si_data_len: u32,
    _si_flags: __wasi_siflags_t,
    _so_datalen: WasmPtr<u32>,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn sock_shutdown(
    _env: FunctionEnvMut<WasiEnv>,
    _sock: __wasi_fd_t,
    _how: __wasi_sdflags_t,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

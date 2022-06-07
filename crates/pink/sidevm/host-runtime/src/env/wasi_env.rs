use super::{Env as WasiEnv, Result};
use libc::{
    clock_getres, clock_gettime, timespec, CLOCK_MONOTONIC, CLOCK_PROCESS_CPUTIME_ID,
    CLOCK_REALTIME, CLOCK_THREAD_CPUTIME_ID,
};
use pink_sidevm_env::OcallFuncs;
use ptr::WasmPtr;
use thiserror::Error;
use wasmer::{namespace, Array, Exports, Function, Store};
use wasmer_wasi_types::*;

mod ptr;

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

pub(crate) fn wasi_imports(store: &Store, env: &WasiEnv) -> Exports {
    namespace! {
        "args_get" => Function::new_native_with_env(store, env.clone(), args_get),
        "args_sizes_get" => Function::new_native_with_env(store, env.clone(), args_sizes_get),
        "clock_res_get" => Function::new_native_with_env(store, env.clone(), clock_res_get),
        "clock_time_get" => Function::new_native_with_env(store, env.clone(), clock_time_get),
        "environ_get" => Function::new_native_with_env(store, env.clone(), environ_get),
        "environ_sizes_get" => Function::new_native_with_env(store, env.clone(), environ_sizes_get),
        "fd_advise" => Function::new_native_with_env(store, env.clone(), fd_advise),
        "fd_allocate" => Function::new_native_with_env(store, env.clone(), fd_allocate),
        "fd_close" => Function::new_native_with_env(store, env.clone(), fd_close),
        "fd_datasync" => Function::new_native_with_env(store, env.clone(), fd_datasync),
        "fd_fdstat_get" => Function::new_native_with_env(store, env.clone(), fd_fdstat_get),
        "fd_fdstat_set_flags" => Function::new_native_with_env(store, env.clone(), fd_fdstat_set_flags),
        "fd_fdstat_set_rights" => Function::new_native_with_env(store, env.clone(), fd_fdstat_set_rights),
        "fd_filestat_get" => Function::new_native_with_env(store, env.clone(), fd_filestat_get),
        "fd_filestat_set_size" => Function::new_native_with_env(store, env.clone(), fd_filestat_set_size),
        "fd_filestat_set_times" => Function::new_native_with_env(store, env.clone(), fd_filestat_set_times),
        "fd_pread" => Function::new_native_with_env(store, env.clone(), fd_pread),
        "fd_prestat_get" => Function::new_native_with_env(store, env.clone(), fd_prestat_get),
        "fd_prestat_dir_name" => Function::new_native_with_env(store, env.clone(), fd_prestat_dir_name),
        "fd_pwrite" => Function::new_native_with_env(store, env.clone(), fd_pwrite),
        "fd_read" => Function::new_native_with_env(store, env.clone(), fd_read),
        "fd_readdir" => Function::new_native_with_env(store, env.clone(), fd_readdir),
        "fd_renumber" => Function::new_native_with_env(store, env.clone(), fd_renumber),
        "fd_seek" => Function::new_native_with_env(store, env.clone(), fd_seek),
        "fd_sync" => Function::new_native_with_env(store, env.clone(), fd_sync),
        "fd_tell" => Function::new_native_with_env(store, env.clone(), fd_tell),
        "fd_write" => Function::new_native_with_env(store, env.clone(), fd_write),
        "path_create_directory" => Function::new_native_with_env(store, env.clone(), path_create_directory),
        "path_filestat_get" => Function::new_native_with_env(store, env.clone(), path_filestat_get),
        "path_filestat_set_times" => Function::new_native_with_env(store, env.clone(), path_filestat_set_times),
        "path_link" => Function::new_native_with_env(store, env.clone(), path_link),
        "path_open" => Function::new_native_with_env(store, env.clone(), path_open),
        "path_readlink" => Function::new_native_with_env(store, env.clone(), path_readlink),
        "path_remove_directory" => Function::new_native_with_env(store, env.clone(), path_remove_directory),
        "path_rename" => Function::new_native_with_env(store, env.clone(), path_rename),
        "path_symlink" => Function::new_native_with_env(store, env.clone(), path_symlink),
        "path_unlink_file" => Function::new_native_with_env(store, env.clone(), path_unlink_file),
        "poll_oneoff" => Function::new_native_with_env(store, env.clone(), poll_oneoff),
        "proc_exit" => Function::new_native_with_env(store, env.clone(), proc_exit),
        "proc_raise" => Function::new_native_with_env(store, env.clone(), proc_raise),
        "random_get" => Function::new_native_with_env(store, env.clone(), random_get),
        "sched_yield" => Function::new_native_with_env(store, env.clone(), sched_yield),
        "sock_recv" => Function::new_native_with_env(store, env.clone(), sock_recv),
        "sock_send" => Function::new_native_with_env(store, env.clone(), sock_send),
        "sock_shutdown" => Function::new_native_with_env(store, env.clone(), sock_shutdown),
    }
}

pub fn args_get(
    _env: &WasiEnv,
    _argv: WasmPtr<WasmPtr<u8, Array>, Array>,
    _argv_buf: WasmPtr<u8, Array>,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn args_sizes_get(
    _env: &WasiEnv,
    _argc: WasmPtr<u32>,
    _argv_buf_size: WasmPtr<u32>,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn clock_res_get(
    env: &WasiEnv,
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

    let guard = env.inner.lock().unwrap();
    let memory = guard.memory.unwrap_ref();
    let resolution = wasi_try!(resolution.deref(memory));
    resolution.set(t_out as __wasi_timestamp_t);

    __WASI_ESUCCESS
}

pub fn clock_time_get(
    env: &WasiEnv,
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

    let guard = env.inner.lock().unwrap();
    let memory = guard.memory.unwrap_ref();
    let time = wasi_try!(time.deref(memory));
    time.set(t_out as __wasi_timestamp_t);

    __WASI_ESUCCESS
}

pub fn environ_get(
    _env: &WasiEnv,
    _environ: WasmPtr<WasmPtr<u8, Array>, Array>,
    _environ_buf: WasmPtr<u8, Array>,
) -> __wasi_errno_t {
    __WASI_ESUCCESS
}

pub fn environ_sizes_get(
    env: &WasiEnv,
    environ_count: WasmPtr<u32>,
    environ_buf_size: WasmPtr<u32>,
) -> __wasi_errno_t {
    let guard = env.inner.lock().unwrap();
    let memory = guard.memory.unwrap_ref();
    let environ_count = wasi_try!(environ_count.deref(memory));
    let environ_buf_size = wasi_try!(environ_buf_size.deref(memory));
    environ_count.set(0);
    environ_buf_size.set(0);
    __WASI_ESUCCESS
}

pub fn fd_advise(
    _env: &WasiEnv,
    _fd: __wasi_fd_t,
    _offset: __wasi_filesize_t,
    _len: __wasi_filesize_t,
    _advice: __wasi_advice_t,
) -> __wasi_errno_t {
    __WASI_ESUCCESS
}

pub fn fd_allocate(
    _env: &WasiEnv,
    _fd: __wasi_fd_t,
    _offset: __wasi_filesize_t,
    _len: __wasi_filesize_t,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn fd_close(_env: &WasiEnv, _fd: __wasi_fd_t) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn fd_datasync(_env: &WasiEnv, _fd: __wasi_fd_t) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn fd_fdstat_get(
    _env: &WasiEnv,
    _fd: __wasi_fd_t,
    _buf_ptr: WasmPtr<__wasi_fdstat_t>,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn fd_fdstat_set_flags(
    _env: &WasiEnv,
    _fd: __wasi_fd_t,
    _flags: __wasi_fdflags_t,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn fd_fdstat_set_rights(
    _env: &WasiEnv,
    _fd: __wasi_fd_t,
    _fs_rights_base: __wasi_rights_t,
    _fs_rights_inheriting: __wasi_rights_t,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn fd_filestat_get(
    _env: &WasiEnv,
    _fd: __wasi_fd_t,
    _buf: WasmPtr<__wasi_filestat_t>,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn fd_filestat_set_size(
    _env: &WasiEnv,
    _fd: __wasi_fd_t,
    _st_size: __wasi_filesize_t,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn fd_filestat_set_times(
    _env: &WasiEnv,
    _fd: __wasi_fd_t,
    _st_atim: __wasi_timestamp_t,
    _st_mtim: __wasi_timestamp_t,
    _fst_flags: __wasi_fstflags_t,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn fd_pread(
    _env: &WasiEnv,
    _fd: __wasi_fd_t,
    _iovs: WasmPtr<__wasi_iovec_t, Array>,
    _iovs_len: u32,
    _offset: __wasi_filesize_t,
    _nread: WasmPtr<u32>,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn fd_prestat_get(
    _env: &WasiEnv,
    _fd: __wasi_fd_t,
    _buf: WasmPtr<__wasi_prestat_t>,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn fd_prestat_dir_name(
    _env: &WasiEnv,
    _fd: __wasi_fd_t,
    _path: WasmPtr<u8, Array>,
    _path_len: u32,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn fd_pwrite(
    _env: &WasiEnv,
    _fd: __wasi_fd_t,
    _iovs: WasmPtr<__wasi_ciovec_t, Array>,
    _iovs_len: u32,
    _offset: __wasi_filesize_t,
    _nwritten: WasmPtr<u32>,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn fd_read(
    _env: &WasiEnv,
    _fd: __wasi_fd_t,
    _iovs: WasmPtr<__wasi_iovec_t, Array>,
    _iovs_len: u32,
    _nread: WasmPtr<u32>,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn fd_readdir(
    _env: &WasiEnv,
    _fd: __wasi_fd_t,
    _buf: WasmPtr<u8, Array>,
    _buf_len: u32,
    _cookie: __wasi_dircookie_t,
    _bufused: WasmPtr<u32>,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn fd_renumber(_env: &WasiEnv, _from: __wasi_fd_t, _to: __wasi_fd_t) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn fd_seek(
    _env: &WasiEnv,
    _fd: __wasi_fd_t,
    _offset: __wasi_filedelta_t,
    _whence: __wasi_whence_t,
    _newoffset: WasmPtr<__wasi_filesize_t>,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn fd_sync(_env: &WasiEnv, _fd: __wasi_fd_t) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn fd_tell(
    _env: &WasiEnv,
    _fd: __wasi_fd_t,
    _offset: WasmPtr<__wasi_filesize_t>,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn fd_write(
    _env: &WasiEnv,
    _fd: __wasi_fd_t,
    _iovs: WasmPtr<__wasi_ciovec_t, Array>,
    _iovs_len: u32,
    _nwritten: WasmPtr<u32>,
) -> __wasi_errno_t {
    __WASI_ESUCCESS
}

pub fn path_create_directory(
    _env: &WasiEnv,
    _fd: __wasi_fd_t,
    _path: WasmPtr<u8, Array>,
    _path_len: u32,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn path_filestat_get(
    _env: &WasiEnv,
    _fd: __wasi_fd_t,
    _flags: __wasi_lookupflags_t,
    _path: WasmPtr<u8, Array>,
    _path_len: u32,
    _buf: WasmPtr<__wasi_filestat_t>,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn path_filestat_set_times(
    _env: &WasiEnv,
    _fd: __wasi_fd_t,
    _flags: __wasi_lookupflags_t,
    _path: WasmPtr<u8, Array>,
    _path_len: u32,
    _st_atim: __wasi_timestamp_t,
    _st_mtim: __wasi_timestamp_t,
    _fst_flags: __wasi_fstflags_t,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn path_link(
    _env: &WasiEnv,
    _old_fd: __wasi_fd_t,
    _old_flags: __wasi_lookupflags_t,
    _old_path: WasmPtr<u8, Array>,
    _old_path_len: u32,
    _new_fd: __wasi_fd_t,
    _new_path: WasmPtr<u8, Array>,
    _new_path_len: u32,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn path_open(
    _env: &WasiEnv,
    _dirfd: __wasi_fd_t,
    _dirflags: __wasi_lookupflags_t,
    _path: WasmPtr<u8, Array>,
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
    _env: &WasiEnv,
    _dir_fd: __wasi_fd_t,
    _path: WasmPtr<u8, Array>,
    _path_len: u32,
    _buf: WasmPtr<u8, Array>,
    _buf_len: u32,
    _buf_used: WasmPtr<u32>,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn path_remove_directory(
    _env: &WasiEnv,
    _fd: __wasi_fd_t,
    _path: WasmPtr<u8, Array>,
    _path_len: u32,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn path_rename(
    _env: &WasiEnv,
    _old_fd: __wasi_fd_t,
    _old_path: WasmPtr<u8, Array>,
    _old_path_len: u32,
    _new_fd: __wasi_fd_t,
    _new_path: WasmPtr<u8, Array>,
    _new_path_len: u32,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn path_symlink(
    _env: &WasiEnv,
    _old_path: WasmPtr<u8, Array>,
    _old_path_len: u32,
    _fd: __wasi_fd_t,
    _new_path: WasmPtr<u8, Array>,
    _new_path_len: u32,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn path_unlink_file(
    _env: &WasiEnv,
    _fd: __wasi_fd_t,
    _path: WasmPtr<u8, Array>,
    _path_len: u32,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn poll_oneoff(
    _env: &WasiEnv,
    _in_: WasmPtr<__wasi_subscription_t, Array>,
    _out_: WasmPtr<__wasi_event_t, Array>,
    _nsubscriptions: u32,
    _nevents: WasmPtr<u32>,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn proc_exit(_env: &WasiEnv, code: __wasi_exitcode_t) -> Result<(), WasiError> {
    Err(WasiError::Exited(code))
}

pub fn proc_raise(_env: &WasiEnv, _sig: __wasi_signal_t) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn random_get(env: &WasiEnv, buf: u32, buf_len: u32) -> Result<__wasi_errno_t> {
    let mut env_guard = env.inner.lock().unwrap();
    let env = &mut *env_guard;
    let memory = env.memory.unwrap_ref();
    let mut u8_buffer = vec![0; buf_len as usize];
    env.state.getrandom(&mut u8_buffer)?;
    unsafe {
        memory
            .uint8view()
            .subarray(buf as u32, buf as u32 + buf_len as u32)
            .copy_from(&u8_buffer);
    }
    Ok(__WASI_ESUCCESS)
}

pub fn sched_yield(_env: &WasiEnv) -> __wasi_errno_t {
    __WASI_ESUCCESS
}

pub fn sock_recv(
    _env: &WasiEnv,
    _sock: __wasi_fd_t,
    _ri_data: WasmPtr<__wasi_iovec_t, Array>,
    _ri_data_len: u32,
    _ri_flags: __wasi_riflags_t,
    _ro_datalen: WasmPtr<u32>,
    _ro_flags: WasmPtr<__wasi_roflags_t>,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn sock_send(
    _env: &WasiEnv,
    _sock: __wasi_fd_t,
    _si_data: WasmPtr<__wasi_ciovec_t, Array>,
    _si_data_len: u32,
    _si_flags: __wasi_siflags_t,
    _so_datalen: WasmPtr<u32>,
) -> __wasi_errno_t {
    __WASI_ENOSYS
}

pub fn sock_shutdown(_env: &WasiEnv, _sock: __wasi_fd_t, _how: __wasi_sdflags_t) -> __wasi_errno_t {
    __WASI_ENOSYS
}

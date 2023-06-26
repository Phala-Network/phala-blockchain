use super::{Env as WasiEnv, Result};
use libc::{clock_getres, clock_gettime, timespec, CLOCK_MONOTONIC, CLOCK_REALTIME};
use sidevm_env::{OcallError, OcallFuncs};
use thiserror::Error;
use tracing::{error, info};
use wasmer::{
    namespace, AsStoreMut, Exports, Function, FunctionEnv, FunctionEnvMut, Memory32,
    MemoryAccessError, WasmPtr,
};
use wasmer_wasix_types::{
    types::*,
    wasi::{self, Errno, ExitCode},
};

/// This is returned in `RuntimeError`.
/// Use `downcast` or `downcast_ref` to retrieve the `ExitCode`.
#[derive(Error, Debug)]
pub enum WasiError {
    #[error("WASI exited with code: {0}")]
    Exited(ExitCode),
}

macro_rules! wasi_try {
    ($expr:expr) => {{
        let res: Result<_, Errno> = $expr;
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

/// Like the `try!` macro or `?` syntax: returns the value if the computation
/// succeeded or returns the error value. Results are wrapped in an Ok
macro_rules! wasi_try_ok {
    ($expr:expr) => {{
        let res: Result<_, Errno> = $expr;
        match res {
            Ok(val) => val,
            Err(err) => {
                return Ok(err);
            }
        }
    }};
}

/// Like `wasi_try` but converts a `MemoryAccessError` to a `wasi::Errno`.
macro_rules! wasi_try_mem_ok {
    ($expr:expr) => {{
        wasi_try_ok!($expr.map_err(mem_error_to_wasi))
    }};
}

/// Like `wasi_try` but converts a `MemoryAccessError` to a `wasi::Errno`.
macro_rules! wasi_try_block_ok {
    ($expr:expr) => {{
        wasi_try_ok!(|| -> Result<_, Errno> { Ok($expr) }())
    }};
}

fn mem_error_to_wasi(err: MemoryAccessError) -> Errno {
    match err {
        MemoryAccessError::HeapOutOfBounds => Errno::Memviolation,
        MemoryAccessError::Overflow => Errno::Overflow,
        MemoryAccessError::NonUtf8String => Errno::Inval,
        _ => Errno::Unknown,
    }
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
) -> Errno {
    Errno::Nosys
}

pub fn args_sizes_get(
    _env: FunctionEnvMut<WasiEnv>,
    _argc: WasmPtr<u32>,
    _argv_buf_size: WasmPtr<u32>,
) -> Errno {
    Errno::Nosys
}

pub fn clock_res_get(
    env: FunctionEnvMut<WasiEnv>,
    clock_id: wasi::Clockid,
    resolution: WasmPtr<wasi::Timestamp>,
) -> Errno {
    use wasi::Clockid::*;
    let unix_clock_id = match clock_id {
        Realtime => CLOCK_REALTIME,
        Monotonic => CLOCK_MONOTONIC,
        ProcessCputimeId | ThreadCputimeId => return Errno::Notsup,
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
        resolution.write(t_out as wasi::Timestamp).ok(),
        Errno::Fault
    );

    Errno::Success
}

pub fn clock_time_get(
    env: FunctionEnvMut<WasiEnv>,
    clock_id: wasi::Clockid,
    _precision: wasi::Timestamp,
    time: WasmPtr<wasi::Timestamp>,
) -> Errno {
    use wasi::Clockid::*;
    let unix_clock_id = match clock_id {
        Realtime => CLOCK_REALTIME,
        Monotonic => CLOCK_MONOTONIC,
        ProcessCputimeId | ThreadCputimeId => return Errno::Notsup,
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
    wasi_try!(time.write(t_out as wasi::Timestamp).ok(), Errno::Fault);

    Errno::Success
}

pub fn environ_get(
    _env: FunctionEnvMut<WasiEnv>,
    _environ: WasmPtr<WasmPtr<u8>>,
    _environ_buf: WasmPtr<u8>,
) -> Errno {
    Errno::Success
}

pub fn environ_sizes_get(
    env: FunctionEnvMut<WasiEnv>,
    environ_count: WasmPtr<u32>,
    environ_buf_size: WasmPtr<u32>,
) -> Errno {
    let guard = env.data().inner.lock().unwrap();
    let memory = guard.memory.unwrap_ref().view(&env);
    let environ_count = environ_count.deref(&memory);
    let environ_buf_size = environ_buf_size.deref(&memory);
    wasi_try!(environ_count.write(0).ok(), Errno::Fault);
    wasi_try!(environ_buf_size.write(0).ok(), Errno::Fault);
    Errno::Success
}

pub fn fd_advise(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: wasi::Fd,
    _offset: wasi::Filesize,
    _len: wasi::Filesize,
    _advice: wasi::Advice,
) -> Errno {
    Errno::Success
}

pub fn fd_allocate(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: wasi::Fd,
    _offset: wasi::Filesize,
    _len: wasi::Filesize,
) -> Errno {
    Errno::Nosys
}

pub fn fd_close(_env: FunctionEnvMut<WasiEnv>, _fd: wasi::Fd) -> Errno {
    Errno::Nosys
}

pub fn fd_datasync(_env: FunctionEnvMut<WasiEnv>, _fd: wasi::Fd) -> Errno {
    Errno::Nosys
}

pub fn fd_fdstat_get(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: wasi::Fd,
    _buf_ptr: WasmPtr<wasi::Fdstat>,
) -> Errno {
    Errno::Nosys
}

pub fn fd_fdstat_set_flags(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: wasi::Fd,
    _flags: wasi::Fdflags,
) -> Errno {
    Errno::Nosys
}

pub fn fd_fdstat_set_rights(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: wasi::Fd,
    _fs_rights_base: wasi::Rights,
    _fs_rights_inheriting: wasi::Rights,
) -> Errno {
    Errno::Nosys
}

pub fn fd_filestat_get(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: wasi::Fd,
    _buf: WasmPtr<wasi::Filestat>,
) -> Errno {
    Errno::Nosys
}

pub fn fd_filestat_set_size(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: wasi::Fd,
    _st_size: wasi::Filesize,
) -> Errno {
    Errno::Nosys
}

pub fn fd_filestat_set_times(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: wasi::Fd,
    _st_atim: wasi::Timestamp,
    _st_mtim: wasi::Timestamp,
    _fst_flags: wasi::Fstflags,
) -> Errno {
    Errno::Nosys
}

pub fn fd_pread(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: wasi::Fd,
    _iovs: WasmPtr<__wasi_iovec_t<Memory32>>,
    _iovs_len: u32,
    _offset: wasi::Filesize,
    _nread: WasmPtr<u32>,
) -> Errno {
    Errno::Nosys
}

pub fn fd_prestat_get(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: wasi::Fd,
    _buf: WasmPtr<wasi::Prestat>,
) -> Errno {
    Errno::Badf
}

pub fn fd_prestat_dir_name(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: wasi::Fd,
    _path: WasmPtr<u8>,
    _path_len: u32,
) -> Errno {
    Errno::Nosys
}

pub fn fd_pwrite(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: wasi::Fd,
    _iovs: WasmPtr<__wasi_ciovec_t<Memory32>>,
    _iovs_len: u32,
    _offset: wasi::Filesize,
    _nwritten: WasmPtr<u32>,
) -> Errno {
    Errno::Nosys
}

pub fn fd_read(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: wasi::Fd,
    _iovs: WasmPtr<__wasi_iovec_t<Memory32>>,
    _iovs_len: u32,
    _nread: WasmPtr<u32>,
) -> Errno {
    Errno::Nosys
}

pub fn fd_readdir(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: wasi::Fd,
    _buf: WasmPtr<u8>,
    _buf_len: u32,
    _cookie: wasi::Dircookie,
    _bufused: WasmPtr<u32>,
) -> Errno {
    Errno::Nosys
}

pub fn fd_renumber(_env: FunctionEnvMut<WasiEnv>, _from: wasi::Fd, _to: wasi::Fd) -> Errno {
    Errno::Nosys
}

pub fn fd_seek(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: wasi::Fd,
    _offset: wasi::FileDelta,
    _whence: wasi::Whence,
    _newoffset: WasmPtr<wasi::Filesize>,
) -> Errno {
    Errno::Nosys
}

pub fn fd_sync(_env: FunctionEnvMut<WasiEnv>, _fd: wasi::Fd) -> Errno {
    Errno::Nosys
}

pub fn fd_tell(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: wasi::Fd,
    _offset: WasmPtr<wasi::Filesize>,
) -> Errno {
    Errno::Nosys
}

pub fn fd_write(
    ctx: FunctionEnvMut<WasiEnv>,
    fd: wasi::Fd,
    iovs: WasmPtr<__wasi_ciovec_t<Memory32>>,
    iovs_len: u32,
    nwritten: WasmPtr<u32>,
) -> Result<Errno, WasiError> {
    if fd != 1 && fd != 2 {
        return Ok(Errno::Badf);
    }
    fd_write_stdio(ctx, fd, iovs, iovs_len, nwritten)
}

fn fd_write_stdio(
    ctx: FunctionEnvMut<'_, WasiEnv>,
    fd: wasi::Fd,
    iovs: WasmPtr<__wasi_ciovec_t<Memory32>>,
    iovs_len: u32,
    nwritten: WasmPtr<u32>,
) -> Result<Errno, WasiError> {
    let env = ctx.data();
    let memory = env.memory();
    let memory = memory.view(&ctx);
    let iovs_arr = wasi_try_mem_ok!(iovs.slice(&memory, iovs_len));
    let iovs_arr = wasi_try_mem_ok!(iovs_arr.access());
    let mut written = 0usize;
    for iovs in iovs_arr.iter() {
        let buf = wasi_try_block_ok! {
            WasmPtr::<u8>::new(iovs.buf)
                .slice(&memory, iovs.buf_len)
                .map_err(mem_error_to_wasi)?
                .access()
                .map_err(mem_error_to_wasi)?
        };
        let bytes = buf.as_ref();
        for line in String::from_utf8_lossy(&bytes[..bytes.len().min(1024)]).lines() {
            info!(fd, "guest write: {}", line);
        }
        written += buf.len();
    }
    let nwritten_ref = nwritten.deref(&memory);
    let written: u32 = wasi_try_ok!(written.try_into().map_err(|_| Errno::Overflow));
    wasi_try_mem_ok!(nwritten_ref.write(written));
    Ok(Errno::Success)
}

pub fn path_create_directory(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: wasi::Fd,
    _path: WasmPtr<u8>,
    _path_len: u32,
) -> Errno {
    Errno::Nosys
}

pub fn path_filestat_get(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: wasi::Fd,
    _flags: wasi::LookupFlags,
    _path: WasmPtr<u8>,
    _path_len: u32,
    _buf: WasmPtr<wasi::Filestat>,
) -> Errno {
    Errno::Nosys
}

#[allow(clippy::too_many_arguments)]
pub fn path_filestat_set_times(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: wasi::Fd,
    _flags: wasi::LookupFlags,
    _path: WasmPtr<u8>,
    _path_len: u32,
    _st_atim: wasi::Timestamp,
    _st_mtim: wasi::Timestamp,
    _fst_flags: wasi::Fstflags,
) -> Errno {
    Errno::Nosys
}

#[allow(clippy::too_many_arguments)]
pub fn path_link(
    _env: FunctionEnvMut<WasiEnv>,
    _old_fd: wasi::Fd,
    _old_flags: wasi::LookupFlags,
    _old_path: WasmPtr<u8>,
    _old_path_len: u32,
    _new_fd: wasi::Fd,
    _new_path: WasmPtr<u8>,
    _new_path_len: u32,
) -> Errno {
    Errno::Nosys
}

#[allow(clippy::too_many_arguments)]
pub fn path_open(
    _env: FunctionEnvMut<WasiEnv>,
    _dirfd: wasi::Fd,
    _dirflags: wasi::LookupFlags,
    _path: WasmPtr<u8>,
    _path_len: u32,
    _o_flags: wasi::Oflags,
    _fs_rights_base: wasi::Rights,
    _fs_rights_inheriting: wasi::Rights,
    _fs_flags: wasi::Fdflags,
    _fd: WasmPtr<wasi::Fd>,
) -> Errno {
    Errno::Nosys
}

pub fn path_readlink(
    _env: FunctionEnvMut<WasiEnv>,
    _dir_fd: wasi::Fd,
    _path: WasmPtr<u8>,
    _path_len: u32,
    _buf: WasmPtr<u8>,
    _buf_len: u32,
    _buf_used: WasmPtr<u32>,
) -> Errno {
    Errno::Nosys
}

pub fn path_remove_directory(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: wasi::Fd,
    _path: WasmPtr<u8>,
    _path_len: u32,
) -> Errno {
    Errno::Nosys
}

pub fn path_rename(
    _env: FunctionEnvMut<WasiEnv>,
    _old_fd: wasi::Fd,
    _old_path: WasmPtr<u8>,
    _old_path_len: u32,
    _new_fd: wasi::Fd,
    _new_path: WasmPtr<u8>,
    _new_path_len: u32,
) -> Errno {
    Errno::Nosys
}

pub fn path_symlink(
    _env: FunctionEnvMut<WasiEnv>,
    _old_path: WasmPtr<u8>,
    _old_path_len: u32,
    _fd: wasi::Fd,
    _new_path: WasmPtr<u8>,
    _new_path_len: u32,
) -> Errno {
    Errno::Nosys
}

pub fn path_unlink_file(
    _env: FunctionEnvMut<WasiEnv>,
    _fd: wasi::Fd,
    _path: WasmPtr<u8>,
    _path_len: u32,
) -> Errno {
    Errno::Nosys
}

pub fn poll_oneoff(
    _env: FunctionEnvMut<WasiEnv>,
    _in_: WasmPtr<wasi::Subscription>,
    _out_: WasmPtr<wasi::Event>,
    _nsubscriptions: u32,
    _nevents: WasmPtr<u32>,
) -> Errno {
    Errno::Nosys
}

pub fn proc_exit(_env: FunctionEnvMut<WasiEnv>, code: ExitCode) -> Result<(), WasiError> {
    Err(WasiError::Exited(code))
}

pub fn proc_raise(_env: FunctionEnvMut<WasiEnv>, _sig: wasi::Signal) -> Errno {
    Errno::Nosys
}

pub fn random_get(mut env: FunctionEnvMut<WasiEnv>, buf: u32, buf_len: u32) -> Result<Errno> {
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
    Ok(Errno::Success)
}

pub fn sched_yield(_env: FunctionEnvMut<WasiEnv>) -> Errno {
    Errno::Success
}

pub fn sock_recv(
    _env: FunctionEnvMut<WasiEnv>,
    _sock: wasi::Fd,
    _ri_data: WasmPtr<__wasi_iovec_t<Memory32>>,
    _ri_data_len: u32,
    _ri_flags: RiFlags,
    _ro_datalen: WasmPtr<u32>,
    _ro_flags: WasmPtr<RoFlags>,
) -> Errno {
    Errno::Nosys
}

pub fn sock_send(
    _env: FunctionEnvMut<WasiEnv>,
    _sock: wasi::Fd,
    _si_data: WasmPtr<__wasi_ciovec_t<Memory32>>,
    _si_data_len: u32,
    _si_flags: SiFlags,
    _so_datalen: WasmPtr<u32>,
) -> Errno {
    Errno::Nosys
}

pub fn sock_shutdown(_env: FunctionEnvMut<WasiEnv>, _sock: wasi::Fd, _how: SdFlags) -> Errno {
    Errno::Nosys
}

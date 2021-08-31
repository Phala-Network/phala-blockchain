use libc::{self, c_char, c_int, c_uint, c_ulong, c_void, iovec, size_t, ssize_t};
use sgx_libc::{self, ocall};
use sgx_types::{sgx_read_rand, sgx_status_t};
use sgx_unwind as _;
use std::{
    ffi::CStr,
    sync::atomic::{AtomicU16, Ordering},
};
use log::{error, info};

macro_rules! assert_eq_size {
    ($x:ty, $($xs:ty),+ $(,)?) => {
        const _: fn() = || {
            $(let _ = std::mem::transmute::<$x, $xs>;)+
        };
    };
}

macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }};
}

macro_rules! not_allowed {
    () => {
        not_allowed!(-1)
    };
    ($rv: expr) => {{
        error!("NOT ALLOED TO CALL {}", function!());
        set_errno(libc::EPERM);
        ($rv) as _
    }};
}

fn set_errno(errno: libc::c_int) {
    unsafe {
        libc::__errno_location().write(errno);
    }
}

pub fn init() {
    use std::io;
    let _ = (io::stdin(), io::stdout(), io::stderr());
}

#[no_mangle]
pub extern "C" fn posix_memalign(memptr: *mut *mut c_void, align: size_t, size: size_t) -> c_int {
    unsafe {
        let ptr = sgx_libc::memalign(align, size);
        // man posix_memalign:
        // posix_memalign() returns zero on success, or one of the error values listed in the next section on failure.  The value of errno is not set.  On Linux (and other systems),
        // posix_memalign() does not modify memptr on failure.  A requirement standardizing this behavior was added in POSIX.1-2016.
        if ptr.is_null() && size > 0 {
            libc::ENOMEM
        } else {
            *memptr = ptr;
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn read(fd: c_int, buf: *mut c_void, count: size_t) -> ssize_t {
    if fd != 0 {
        not_allowed!()
    } else {
        unsafe { ocall::read(fd, buf, count) }
    }
}

#[no_mangle]
pub extern "C" fn readv(fd: c_int, iov: *const iovec, iovcnt: c_int) -> ssize_t {
    assert_eq_size!(libc::iovec, sgx_libc::iovec);

    if fd != 0 {
        not_allowed!()
    } else {
        unsafe { ocall::readv(fd, iov as _, iovcnt) }
    }
}

#[no_mangle]
pub extern "C" fn write(fd: c_int, buf: *const c_void, count: size_t) -> ssize_t {
    if !matches!(fd, 1 | 2) {
        not_allowed!()
    } else {
        unsafe { ocall::write(fd, buf, count) }
    }
}

#[no_mangle]
pub extern "C" fn writev(fd: c_int, iov: *const iovec, iovcnt: c_int) -> ssize_t {
    assert_eq_size!(libc::iovec, sgx_libc::iovec);

    if !matches!(fd, 1 | 2) {
        not_allowed!()
    } else {
        unsafe { ocall::writev(fd, iov as _, iovcnt) }
    }
}

#[no_mangle]
pub extern "C" fn getcwd(mut buf: *mut c_char, size: size_t) -> *mut c_char {
    // Enclave have no working directory, let's return "(unreachable)"
    /*
    man getcwd:
       If the current directory is not below the root directory of the current process (e.g., because the process set a new filesystem root  using  chroot(2)  without
       changing its current directory into the new root), then, since Linux 2.6.36, the returned path will be prefixed with the string "(unreachable)".  Such behavior
       can also be caused by an unprivileged user by changing the current directory into another mount namespace.  When dealing with  paths  from  untrusted  sources,
       callers of these functions should consider checking whether the returned path starts with '/' or '(' to avoid misinterpreting an unreachable path as a relative
       path
    */
    let path = b"(unreachable)\0";
    if size < path.len() {
        set_errno(libc::ERANGE);
        return core::ptr::null_mut();
    }
    unsafe {
        if buf.is_null() {
            buf = sgx_libc::malloc(size) as *mut c_char;
            if buf.is_null() {
                set_errno(libc::ENOMEM);
                return core::ptr::null_mut();
            }
        }
        path.as_ptr()
            .copy_to_nonoverlapping(buf as *mut u8, path.len());
    }
    return buf;
}

#[no_mangle]
pub unsafe extern "C" fn syscall(num: libc::c_long, mut args: ...) -> libc::c_long {
    if num == libc::SYS_getrandom {
        let buf = args.arg::<*mut c_void>();
        let buflen = args.arg::<size_t>();
        let flags = args.arg::<c_uint>();
        return getrandom(buf, buflen, flags) as _;
    }
    eprintln!("unsupported syscall({})", num);
    loop {}
}

#[no_mangle]
pub extern "C" fn memrchr(s: *mut c_void, c: c_int, n: size_t) -> *mut c_void {
    unsafe { sgx_libc::memrchr(s as _, c as _, n) as _ }
}

// sgx_mutex will ignore attributes
#[no_mangle]
pub extern "C" fn pthread_mutexattr_init(_attr: *mut libc::pthread_mutexattr_t) -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn pthread_mutexattr_settype(
    _attr: *mut libc::pthread_mutexattr_t,
    typ: c_int,
) -> c_int {
    // sgx pthread_mutex only support normal mutex, but the stdout() requires PTHREAD_MUTEX_RECURSIVE type.
    // assert_eq!(typ, libc::PTHREAD_MUTEX_NORMAL);

    static RE: AtomicU16 = AtomicU16::new(0);

    if typ != libc::PTHREAD_MUTEX_NORMAL {
        // workaround for stdout(), stderr() and stdin()
        let count = RE.fetch_add(1, Ordering::Relaxed);
        if count > 3 {
            panic!("only PTHREAD_MUTEX_NORMAL supported");
        }
    }

    0
}

#[no_mangle]
pub extern "C" fn pthread_mutexattr_destroy(_attr: *mut libc::pthread_mutexattr_t) -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn sched_yield() -> c_int {
    unsafe { ocall::sched_yield() }
}

extern "C" {
    fn strerror_r(errnum: c_int, buf: *mut c_char, buflen: size_t) -> c_int;
}

#[no_mangle]
pub extern "C" fn __xpg_strerror_r(errnum: c_int, buf: *mut c_char, buflen: size_t) -> c_int {
    unsafe { strerror_r(errnum, buf, buflen) }
}

#[no_mangle]
pub extern "C" fn clock_gettime(clk_id: libc::clockid_t, tp: *mut libc::timespec) -> c_int {
    assert_eq_size!(libc::timespec, sgx_libc::timespec);

    unsafe { ocall::clock_gettime(clk_id, tp as _) }
}

#[no_mangle]
pub extern "C" fn close(_fd: c_int) -> c_int {
    not_allowed!()
}

#[no_mangle]
pub extern "C" fn getrandom(buf: *mut c_void, buflen: size_t, flags: c_uint) -> ssize_t {
    if buflen == 0 {
        return 0;
    }

    let rv = unsafe { sgx_read_rand(buf as _, buflen) };

    match rv {
        sgx_status_t::SGX_SUCCESS => buflen as _,
        _ => {
            if flags & libc::GRND_NONBLOCK != 0 {
                set_errno(libc::EAGAIN);
            } else {
                set_errno(libc::EINTR);
            }
            -1
        }
    }
}

#[no_mangle]
pub extern "C" fn dlsym(_handle: *const c_void, symbol: *const c_char) -> *const c_void {
    let symbol = unsafe { CStr::from_ptr(symbol) };
    unsafe {
        if symbol == CStr::from_bytes_with_nul_unchecked(b"getrandom\0") {
            return getrandom as _;
        }
    }
    eprintln!("WARN: dlsym unable to load symbol {:?}", symbol);
    core::ptr::null()
}

#[no_mangle]
pub extern "C" fn getenv(_name: *const c_char) -> *const c_char {
    // The enclave is not allowed to access environment variables, so return a NULL.
    core::ptr::null()
}

#[no_mangle]
pub extern "C" fn open64(_path: *const c_char, _oflag: c_int, _mode: c_int) -> c_int {
    // unsafe { ocall::open64(path, oflag, mode) }
    not_allowed!()
}

#[no_mangle]
pub extern "C" fn isatty(_fd: c_int) -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn poll(fds: *mut libc::pollfd, nfds: libc::nfds_t, timeout: c_int) -> c_int {
    assert_eq_size!(libc::pollfd, sgx_libc::pollfd);
    unsafe { ocall::poll(fds as _, nfds, timeout) }
}

#[no_mangle]
pub unsafe extern "C" fn ioctl(fd: c_int, request: c_ulong, mut args: ...) -> c_int {
    match request {
        libc::FIONBIO => {
            let arg = args.arg::<*mut c_int>();
            return ocall::ioctl_arg1(fd, request as _, arg);
        }
        _ => {
            eprintln!("ioctl fd={} request={}", fd, request);
            not_allowed!()
        }
    }
}

#[no_mangle]
pub extern "C" fn pthread_atfork(
    _prepare: Option<unsafe extern "C" fn()>,
    _parent: Option<unsafe extern "C" fn()>,
    _child: Option<unsafe extern "C" fn()>,
) -> c_int {
    // NOTE: The rand crate uses pthread_atfork to track forks, and reseeds on forks detected. Since we don't
    // allow dynamic thread creating, It's OK to provide a dummy implementation.
    info!("dummy pthread_atfork called");
    0
}

// mmap, munmap, readlink, fstat64 are required by the stacktrace.

#[no_mangle]
pub extern "C" fn mmap(
    addr: *mut c_void,
    len: size_t,
    prot: c_int,
    flags: c_int,
    fd: c_int,
    offset: libc::off_t,
) -> *mut c_void {
    // The GlobalAlloc in std uses libc::malloc && libc::memalign to alloc memory, but some third-party crate will call mmap to alloc
    // page-aligned memory directly. So we implement the alloc feature and delegate it to memalign.
    if (flags & libc::MAP_ANONYMOUS) != 0 && fd == -1 {
        lazy_static::lazy_static! {
            static ref PAGE_SIZE: size_t = unsafe { ocall::sysconf(libc::_SC_PAGESIZE) } as _;
        }
        return unsafe { sgx_libc::memalign(*PAGE_SIZE, len) };
    }
    info!(
        "mmap(addr={:?}, len={}, prot={}, flags={}, fd={}, offset={})",
        addr, len, prot, flags, fd, offset
    );
    // On success, mmap() returns a pointer to the mapped area.  On error, the value MAP_FAILED (that is, (void *) -1) is returned,
    // and errno is set to indicate the cause of the error.
    not_allowed!()
}

#[no_mangle]
pub extern "C" fn munmap(addr: *mut c_void, _len: size_t) -> c_int {
    if !addr.is_null() {
        // All addrs should be from sgx_libc::memalign.
        unsafe { sgx_libc::free(addr) };
        return 0;
    }
    not_allowed!()
}

#[no_mangle]
pub extern "C" fn readlink(_path: *const c_char, _buf: *mut c_char, _bufsz: size_t) -> ssize_t {
    // unsafe { ocall::readlink(path, buf, bufsz) }
    not_allowed!()
}

#[no_mangle]
pub extern "C" fn fstat64(_fildes: c_int, _buf: *mut libc::stat64) -> c_int {
    // assert_eq_size!(libc::stat64, sgx_libc::stat64);
    // unsafe { ocall::fstat64(fildes, buf as _) }
    not_allowed!()
}

// #[cfg(feature = "net")]
mod net {
    use super::*;

    #[no_mangle]
    pub extern "C" fn freeaddrinfo(res: *mut sgx_libc::addrinfo) {
        unsafe { ocall::freeaddrinfo(res) }
    }

    #[no_mangle]
    pub extern "C" fn socket(domain: c_int, ty: c_int, protocol: c_int) -> c_int {
        unsafe { ocall::socket(domain, ty, protocol) }
    }

    #[no_mangle]
    pub extern "C" fn connect(
        socket: c_int,
        address: *const libc::sockaddr,
        len: libc::socklen_t,
    ) -> c_int {
        assert_eq_size!(libc::sockaddr, sgx_libc::sockaddr);
        unsafe { ocall::connect(socket, address as _, len) }
    }

    #[no_mangle]
    pub extern "C" fn getsockopt(
        sockfd: c_int,
        level: c_int,
        optname: c_int,
        optval: *mut c_void,
        optlen: *mut libc::socklen_t,
    ) -> c_int {
        unsafe { ocall::getsockopt(sockfd, level, optname, optval, optlen) }
    }

    #[no_mangle]
    pub extern "C" fn send(
        socket: c_int,
        buf: *const c_void,
        len: size_t,
        flags: c_int,
    ) -> ssize_t {
        unsafe { ocall::send(socket, buf, len, flags) }
    }

    #[no_mangle]
    pub extern "C" fn recv(socket: c_int, buf: *mut c_void, len: size_t, flags: c_int) -> ssize_t {
        unsafe { ocall::recv(socket, buf, len, flags) }
    }

    #[no_mangle]
    pub extern "C" fn setsockopt(
        socket: c_int,
        level: c_int,
        name: c_int,
        value: *const c_void,
        option_len: libc::socklen_t,
    ) -> c_int {
        unsafe { ocall::setsockopt(socket, level, name, value, option_len) }
    }

    #[no_mangle]
    pub extern "C" fn getaddrinfo(
        node: *const c_char,
        service: *const c_char,
        hints: *const libc::addrinfo,
        res: *mut *mut libc::addrinfo,
    ) -> c_int {
        assert_eq_size!(libc::addrinfo, sgx_libc::addrinfo);
        unsafe { ocall::getaddrinfo(node, service, hints as _, res as _) }
    }

    #[no_mangle]
    pub extern "C" fn __res_init() -> c_int {
        eprintln!("WARN: __res_init not implemented");
        0
    }

    #[no_mangle]
    pub extern "C" fn gai_strerror(_errcode: c_int) -> *const c_char {
        // different memory space
        // ocall::gai_strerror(errcode)
        b"gai_strerror not supported\0".as_ptr() as _
    }
}

// For debugging
#[cfg(feature = "libc_placeholders")]
mod placeholders {
    #[no_mangle]
    pub extern "C" fn sigaltstack() {}
    #[no_mangle]
    pub extern "C" fn sysconf() {}
    #[no_mangle]
    pub extern "C" fn mprotect() {}
    #[no_mangle]
    pub extern "C" fn pthread_attr_init() {}
    #[no_mangle]
    pub extern "C" fn pthread_attr_setstacksize() {}
    #[no_mangle]
    pub extern "C" fn pthread_attr_destroy() {}
    #[no_mangle]
    pub extern "C" fn prctl() {}
    #[no_mangle]
    pub extern "C" fn pthread_detach() {}
    #[no_mangle]
    pub extern "C" fn pthread_getattr_np() {}
    #[no_mangle]
    pub extern "C" fn pthread_attr_getguardsize() {}
    #[no_mangle]
    pub extern "C" fn pthread_attr_getstack() {}
    #[no_mangle]
    pub extern "C" fn sched_getaffinity() {}
}

#[cfg(feature = "tests")]
pub(crate) mod tests {
    use super::*;

    fn test_getcwd() {
        assert_eq!(
            std::env::current_dir().unwrap(),
            std::path::PathBuf::from("(unreachable)")
        );
    }

    fn test_seterrno() {
        set_errno(42);
        unsafe {
            assert_eq!(libc::__errno_location().read(), 42);
        }
    }

    pub(crate) fn test_all() {
        test_getcwd();
        test_seterrno();
    }
}

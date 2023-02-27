use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::sync::RwLock;

use log::error;

static WORKER_PUBKEY: RwLock<[u8; 32]> = RwLock::new([0; 32]);

pub fn set_worker_pubkey(key: [u8; 32]) {
    *WORKER_PUBKEY.write().unwrap() = key;
}

/// Load given version of lib pink library using dlopen and return a handle to it.
fn load_pink_library((major, minor): (u32, u32)) -> *mut libc::c_void {
    let runtime_dir = match std::env::var("PINK_RUNTIME_PATH") {
        Ok(path) => std::path::Path::new(&path).to_owned(),
        Err(_) => std::env::current_exe()
            .expect("Failed to get current exe path")
            .parent()
            .unwrap()
            .to_owned(),
    };
    let filename = format!("libpink.so.{major}.{minor}");
    let path = runtime_dir.join(filename);
    let Ok(path) = CString::new(path.as_os_str().as_bytes()) else {
        return std::ptr::null_mut();
    };
    let handle = unsafe { libc::dlopen(path.as_ptr(), libc::RTLD_NOW) };
    if handle.is_null() {
        let err = unsafe { std::ffi::CStr::from_ptr(libc::dlerror()) };
        error!(
            "Failed to load {}: {}",
            path.to_string_lossy(),
            err.to_string_lossy()
        );
    }
    handle
}

/// Check if we are running in an enclave according to the existence of /dev/attestation/user_report_data
fn in_enclave() -> i32 {
    let path = std::path::Path::new("/dev/attestation/user_report_data");
    if path.exists() {
        1
    } else {
        0
    }
}

pub mod v1;

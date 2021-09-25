#![feature(decl_macro)]

use std::thread;

extern crate env_logger;
extern crate mio;
extern crate sgx_types;
extern crate sgx_urts;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
extern crate rocket_cors;

extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate parity_scale_codec;
extern crate phactory_api;
extern crate structopt;

#[cfg(test)]
mod tests;
#[cfg(test)]
extern crate base64;
#[cfg(test)]
extern crate hex_literal;
#[cfg(test)]
extern crate ring_compat;

mod attestation;
mod contract_input;
mod contract_output;

use colored::Colorize;
use sgx_types::*;
use sgx_urts::SgxEnclave;

use std::env;
use std::path;
use std::str;
use std::sync::RwLock;

use rocket::data::Data;
use rocket::http::Method;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket_contrib::json::{Json, JsonValue};
use rocket_cors::{AllowedHeaders, AllowedMethods, AllowedOrigins, CorsOptions};
use structopt::StructOpt;

use contract_input::ContractInput;
use parity_scale_codec::Encode;
use phactory_api::{
    actions,
    ecall_args::{git_revision, InitArgs},
    prpc,
};

#[derive(StructOpt, Debug)]
#[structopt(name = "pruntime", about = "The Phala TEE worker app.")]
struct Args {
    /// Number of CPU cores to be used for mining.
    #[structopt(short, long)]
    cores: Option<u32>,

    /// Run benchmark at startup.
    #[structopt(long)]
    init_bench: bool,

    /// Enable geolocaltion report
    #[structopt(long)]
    enable_geoprobing: bool,

    #[structopt(long, default_value = "./GeoLite2-City.mmdb")]
    geoip_city_db: String,
}

static ENCLAVE_FILE: &'static str = "enclave.signed.so";
static ENCLAVE_STATE_FILE: &'static str = "enclave.token";

const ENCLAVE_OUTPUT_BUF_MAX_LEN: usize = 10 * 2048 * 1024 as usize;

lazy_static! {
    static ref ENCLAVE: RwLock<Option<SgxEnclave>> = RwLock::new(None);
    static ref ENCLAVE_STATE_FILE_PATH: &'static str = {
        Box::leak(
            env::var("STATE_FILE_PATH")
                .unwrap_or_else(|_| "./".to_string())
                .into_boxed_str(),
        )
    };
    static ref ALLOW_CORS: bool =
        { env::var("ALLOW_CORS").unwrap_or_else(|_| "".to_string()) != "" };
    static ref ENABLE_KICK_API: bool =
        { env::var("ENABLE_KICK_API").unwrap_or_else(|_| "".to_string()) != "" };
}

fn destroy_enclave() {
    let enclave = ENCLAVE.write().unwrap().take().unwrap();
    enclave.destroy();
}

fn get_eid() -> u64 {
    ENCLAVE.read().unwrap().as_ref().unwrap().geteid()
}

extern "C" {
    fn ecall_handle(
        eid: sgx_enclave_id_t,
        retval: *mut sgx_status_t,
        action: u8,
        input_ptr: *const u8,
        input_len: usize,
        output_ptr: *mut u8,
        output_len_ptr: *mut usize,
        output_buf_len: usize,
    ) -> sgx_status_t;

    fn ecall_init(
        eid: sgx_enclave_id_t,
        retval: *mut sgx_status_t,
        args: *const u8,
        args_len: usize,
    ) -> sgx_status_t;

    fn ecall_bench_run(
        eid: sgx_enclave_id_t,
        retval: *mut sgx_status_t,
        index: u32,
    ) -> sgx_status_t;

    fn ecall_prpc_request(
        eid: sgx_enclave_id_t,
        retval: *mut sgx_status_t,
        path: *const uint8_t,
        path_len: usize,
        data: *const uint8_t,
        data_len: usize,
        status_code: *mut u16,
        output_ptr: *mut uint8_t,
        output_buf_len: usize,
        output_len_ptr: *mut usize,
    ) -> sgx_status_t;

    fn ecall_async_reactor_run(eid: sgx_enclave_id_t) -> sgx_status_t;
    fn ecall_async_executor_run(eid: sgx_enclave_id_t) -> sgx_status_t;
}

const IAS_SPID_STR: &str = env!("IAS_SPID");
const IAS_API_KEY_STR: &str = env!("IAS_API_KEY");

#[no_mangle]
pub extern "C" fn ocall_load_ias_spid(
    key_ptr: *mut u8,
    key_len_ptr: *mut usize,
    key_buf_len: usize,
) -> sgx_status_t {
    let key_len = IAS_SPID_STR.len();

    unsafe {
        if key_len <= key_buf_len {
            std::ptr::copy_nonoverlapping(IAS_SPID_STR.as_ptr(), key_ptr, key_len);
        } else {
            panic!("IAS_SPID_STR too long. Buffer overflow.");
        }
        std::ptr::copy_nonoverlapping(
            &key_len as *const usize,
            key_len_ptr,
            std::mem::size_of_val(&key_len),
        );
    }

    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub extern "C" fn ocall_load_ias_key(
    key_ptr: *mut u8,
    key_len_ptr: *mut usize,
    key_buf_len: usize,
) -> sgx_status_t {
    let key_len = IAS_API_KEY_STR.len();

    unsafe {
        if key_len <= key_buf_len {
            std::ptr::copy_nonoverlapping(IAS_API_KEY_STR.as_ptr(), key_ptr, key_len);
        } else {
            panic!("IAS_API_KEY_STR too long. Buffer overflow.");
        }
        std::ptr::copy_nonoverlapping(
            &key_len as *const usize,
            key_len_ptr,
            std::mem::size_of_val(&key_len),
        );
    }

    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub extern "C" fn ocall_sgx_init_quote(
    ret_ti: *mut sgx_target_info_t,
    ret_gid: *mut sgx_epid_group_id_t,
) -> sgx_status_t {
    info!("Entering ocall_sgx_init_quote");
    unsafe { sgx_init_quote(ret_ti, ret_gid) }
}

#[no_mangle]
pub extern "C" fn ocall_get_quote(
    p_sigrl: *const u8,
    sigrl_len: u32,
    p_report: *const sgx_report_t,
    quote_type: sgx_quote_sign_type_t,
    p_spid: *const sgx_spid_t,
    p_nonce: *const sgx_quote_nonce_t,
    p_qe_report: *mut sgx_report_t,
    p_quote: *mut u8,
    _maxlen: u32,
    p_quote_len: *mut u32,
) -> sgx_status_t {
    info!("Entering ocall_get_quote");

    let mut real_quote_len: u32 = 0;

    let ret = unsafe { sgx_calc_quote_size(p_sigrl, sigrl_len, &mut real_quote_len as *mut u32) };

    if ret != sgx_status_t::SGX_SUCCESS {
        warn!("sgx_calc_quote_size returned {}", ret);
        return ret;
    }

    info!("quote size = {}", real_quote_len);
    unsafe {
        *p_quote_len = real_quote_len;
    }

    let ret = unsafe {
        sgx_get_quote(
            p_report,
            quote_type,
            p_spid,
            p_nonce,
            p_sigrl,
            sigrl_len,
            p_qe_report,
            p_quote as *mut sgx_quote_t,
            real_quote_len,
        )
    };

    if ret != sgx_status_t::SGX_SUCCESS {
        warn!("sgx_calc_quote_size returned {}", ret);
        return ret;
    }

    info!("sgx_calc_quote_size returned {}", ret);
    ret
}

#[no_mangle]
pub extern "C" fn ocall_get_update_info(
    platform_blob: *const sgx_platform_info_t,
    enclave_trusted: i32,
    update_info: *mut sgx_update_info_bit_t,
) -> sgx_status_t {
    unsafe { sgx_report_attestation_status(platform_blob, enclave_trusted, update_info) }
}

#[no_mangle]
pub extern "C" fn ocall_eventfd(
    errno: *mut libc::c_int,
    init: libc::c_uint,
    flags: libc::c_int,
) -> libc::c_int {
    unsafe {
        let rv = libc::eventfd(init, flags);
        *errno = *libc::__errno_location();
        rv
    }
}

#[no_mangle]
pub extern "C" fn ocall_timerfd_create(
    errno: *mut libc::c_int,
    clockid: libc::c_int,
    flags: libc::c_int,
) -> libc::c_int {
    unsafe {
        let rv = libc::timerfd_create(clockid, flags);
        *errno = *libc::__errno_location();
        rv
    }
}

#[no_mangle]
pub extern "C" fn ocall_timerfd_settime(
    errno: *mut libc::c_int,
    fd: libc::c_int,
    flags: libc::c_int,
    new_value: *const libc::itimerspec,
    old_value: *mut libc::itimerspec,
) -> libc::c_int {
    unsafe {
        let rv = libc::timerfd_settime(fd, flags, new_value, old_value);
        *errno = *libc::__errno_location();
        rv
    }
}

#[no_mangle]
pub extern "C" fn ocall_timerfd_gettime(
    errno: *mut libc::c_int,
    fd: libc::c_int,
    curr_value: *mut libc::itimerspec,
) -> libc::c_int {
    unsafe {
        let rv = libc::timerfd_gettime(fd, curr_value);
        *errno = *libc::__errno_location();
        rv
    }
}

fn init_enclave() -> SgxResult<SgxEnclave> {
    let mut launch_token: sgx_launch_token_t = [0; 1024];
    let mut launch_token_updated: i32 = 0;
    // call sgx_create_enclave to initialize an enclave instance
    // Debug Support: set 2nd parameter to 1
    let debug = option_env!("SGX_DEBUG").unwrap_or("1");

    let mut misc_attr = sgx_misc_attribute_t {
        secs_attr: sgx_attributes_t { flags: 0, xfrm: 0 },
        misc_select: 0,
    };
    SgxEnclave::create(
        ENCLAVE_FILE,
        if debug == "0" { 0 } else { 1 },
        &mut launch_token,
        &mut launch_token_updated,
        &mut misc_attr,
    )
}

macro_rules! do_ecall_handle {
    ($num: expr, $content: expr) => {{
        let eid = crate::get_eid();

        let mut return_output_buf = vec![0; crate::ENCLAVE_OUTPUT_BUF_MAX_LEN].into_boxed_slice();
        let mut output_len : usize = 0;
        let output_slice = &mut return_output_buf;
        let output_ptr = output_slice.as_mut_ptr();
        let output_len_ptr = &mut output_len as *mut usize;

        let mut retval = crate::sgx_status_t::SGX_SUCCESS;
        let result = unsafe {
            crate::ecall_handle(
                eid, &mut retval,
                $num,
                $content.as_ptr(), $content.len(),
                output_ptr, output_len_ptr, crate::ENCLAVE_OUTPUT_BUF_MAX_LEN
            )
        };

        match result {
            crate::sgx_status_t::SGX_SUCCESS => {
                let output_slice = unsafe { std::slice::from_raw_parts(output_ptr, output_len) };
                let output_value: serde_json::value::Value = serde_json::from_slice(output_slice).unwrap();
                json!(output_value)
            },
            _ => {
                error!("[-] ECALL Enclave Failed {}!", result.as_str());
                json!({
                    "status": "error",
                    "payload": format!("[-] ECALL Enclave Failed {}!", result.as_str())
                })
            }
        }
    }};
}

macro_rules! proxy_post {
    ($rpc: literal, $name: ident, $num: expr) => {
        #[post($rpc, format = "json", data = "<contract_input>")]
        fn $name(contract_input: Json<ContractInput>) -> JsonValue {
            debug!(
                "{}",
                ::serde_json::to_string_pretty(&*contract_input).unwrap()
            );

            let input_string = serde_json::to_string(&*contract_input).unwrap();
            do_ecall_handle!($num, input_string)
        }
    };
}

macro_rules! proxy_get {
    ($rpc: literal, $name: ident, $num: expr) => {
        #[get($rpc)]
        fn $name() -> JsonValue {
            let input_string = r#"{ "input": {} }"#.to_string();
            do_ecall_handle!($num, input_string)
        }
    };
}

macro_rules! proxy {
    (post, $rpc: literal, $name: ident, $num: expr) => {
        proxy_post!($rpc, $name, $num)
    };
    (get, $rpc: literal, $name: ident, $num: expr) => {
        proxy_get!($rpc, $name, $num)
    };
}

fn read_data(data: Data) -> Option<Vec<u8>> {
    use std::io::Read;
    let mut stream = data.open();
    let mut data = Vec::new();
    stream.read_to_end(&mut data).ok()?;
    Some(data)
}

macro_rules! proxy_bin {
    ($rpc: literal, $name: ident, $num: expr) => {
        #[post($rpc, data = "<data>")]
        fn $name(data: Data) -> JsonValue {
            let data = match read_data(data) {
                Some(data) => data,
                None => {
                    return json!({
                        "status": "error",
                        "payload": "Io error: Read input data failed"
                    })
                }
            };
            do_ecall_handle!($num, data)
        }
    };
}

macro_rules! proxy_routes {
    ($(($m: ident, $rpc: literal, $name: ident, $num: expr),)+) => {{
        $(proxy!($m, $rpc, $name, $num);)+
        routes![$($name),+]
    }};
}

macro_rules! proxy_bin_routes {
    ($(($rpc: literal, $name: ident, $num: expr),)+) => {{
        $(proxy_bin!($rpc, $name, $num);)+
        routes![$($name),+]
    }};
}

#[post("/kick")]
fn kick() {
    // TODO: we should improve this
    info!("Kick API received, destroying enclave...");
    destroy_enclave();

    std::process::exit(0);
}

#[post("/<method>", data = "<data>")]
fn prpc_proxy(method: String, data: Data) -> Custom<Vec<u8>> {
    let eid = crate::get_eid();

    let path_bytes = method.as_bytes();
    let path_len = path_bytes.len();
    let path_ptr = path_bytes.as_ptr();

    let data = match crate::read_data(data) {
        Some(data) => data,
        None => {
            return Custom(Status::BadRequest, b"Read body failed".to_vec());
        }
    };
    let data_len = data.len();
    let data_ptr = data.as_ptr();

    let mut output_buf = vec![0; crate::ENCLAVE_OUTPUT_BUF_MAX_LEN];
    let output_buf_len = output_buf.len();
    let output_ptr = output_buf.as_mut_ptr();

    let mut output_len: usize = 0;
    let output_len_ptr = &mut output_len as *mut usize;

    let mut status_code: u16 = 500;

    let mut retval = crate::sgx_status_t::SGX_SUCCESS;

    let result = unsafe {
        crate::ecall_prpc_request(
            eid,
            &mut retval,
            path_ptr,
            path_len,
            data_ptr,
            data_len,
            &mut status_code,
            output_ptr,
            output_buf_len,
            output_len_ptr,
        )
    };

    match result {
        crate::sgx_status_t::SGX_SUCCESS => {
            let output_slice = unsafe { std::slice::from_raw_parts(output_ptr, output_len) };
            if let Some(status) = Status::from_code(status_code) {
                Custom(status, output_slice.to_vec())
            } else {
                error!("[-] prpc: Invalid status code: {}!", status_code);
                Custom(Status::ServiceUnavailable, vec![])
            }
        }
        _ => {
            error!("[-] ECALL Enclave Failed {}!", result.as_str());
            Custom(Status::ServiceUnavailable, vec![])
        }
    }
}

fn cors_options() -> CorsOptions {
    let allowed_origins = AllowedOrigins::all();
    let allowed_methods: AllowedMethods = vec![Method::Get, Method::Post]
        .into_iter()
        .map(From::from)
        .collect();

    // You can also deserialize this
    rocket_cors::CorsOptions {
        allowed_origins,
        allowed_methods,
        allowed_headers: AllowedHeaders::all(),
        allow_credentials: true,
        ..Default::default()
    }
}

fn print_rpc_methods(prefix: &str, methods: &[&str]) {
    info!("Methods under {}:", prefix);
    for method in methods {
        info!("    {}", format!("{}/{}", prefix, method).blue());
    }
}

fn rocket() -> rocket::Rocket {
    let mut server = rocket::ignite()
        .mount(
            "/",
            proxy_routes![
                (get, "/get_info", get_info, actions::ACTION_GET_INFO),
                (post, "/get_info", get_info_post, actions::ACTION_GET_INFO),
            ],
        )
        .mount(
            "/bin_api",
            proxy_bin_routes![
                ("/sync_header", sync_header, actions::BIN_ACTION_SYNC_HEADER),
                (
                    "/dispatch_block",
                    dispatch_block,
                    actions::BIN_ACTION_DISPATCH_BLOCK
                ),
                (
                    "/sync_para_header",
                    sync_para_header,
                    actions::BIN_ACTION_SYNC_PARA_HEADER
                ),
                (
                    "/sync_combined_headers",
                    sync_combined_headers,
                    actions::BIN_ACTION_SYNC_COMBINED_HEADERS
                ),
            ],
        );

    if *ENABLE_KICK_API {
        info!("ENABLE `kick` API");

        server = server.mount("/", routes![kick]);
    }

    server = server.mount("/prpc", routes![prpc_proxy]);
    print_rpc_methods("/prpc", prpc::phactory_api_server::supported_methods());

    if *ALLOW_CORS {
        info!("Allow CORS");

        server
            .mount("/", rocket_cors::catch_all_options_routes()) // mount the catch all routes
            .attach(cors_options().to_cors().expect("To not fail"))
            .manage(cors_options().to_cors().expect("To not fail"))
    } else {
        server
    }
}

fn main() {
    let args = Args::from_args();

    env::set_var("RUST_BACKTRACE", "1");
    env::set_var("ROCKET_ENV", "dev");

    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    let enclave = match init_enclave() {
        Ok(r) => {
            info!("[+] Init Enclave Successful, pid={}!", r.geteid());
            r
        }
        Err(x) => {
            panic!("[-] Init Enclave Failed {}!", x.as_str());
        }
    };

    ENCLAVE.write().unwrap().replace(enclave);

    let eid = get_eid();
    let mut retval = sgx_status_t::SGX_SUCCESS;
    let executable = env::current_exe().unwrap();
    let path = executable.parent().unwrap();
    let sealing_path: path::PathBuf = path.join(*ENCLAVE_STATE_FILE_PATH);
    let sealing_path = String::from(sealing_path.to_str().unwrap());
    let log_filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into());
    let init_args = InitArgs {
        sealing_path,
        log_filter,
        init_bench: args.init_bench,
        version: env!("CARGO_PKG_VERSION").into(),
        git_revision: git_revision(),
        enable_geoprobing: args.enable_geoprobing,
        geoip_city_db: args.geoip_city_db,
    };
    info!("init_args: {:#?}", init_args);
    let encoded_args = init_args.encode();
    let result = unsafe { ecall_init(eid, &mut retval, encoded_args.as_ptr(), encoded_args.len()) };

    if result != sgx_status_t::SGX_SUCCESS {
        panic!("Initialize Failed");
    }

    let _reactor = thread::Builder::new()
        .name("async-reactor".into())
        .spawn(move || {
            let result = unsafe { ecall_async_reactor_run(eid) };
            if result != sgx_status_t::SGX_SUCCESS {
                panic!("[-] ecall_async_reactor_run failed {}!", result);
            }
        })
        .expect("Failed to spawn async-reactor");

    let _executor = thread::Builder::new()
        .name("async-executor".into())
        .spawn(move || {
            let result = unsafe { ecall_async_executor_run(eid) };
            if result != sgx_status_t::SGX_SUCCESS {
                panic!("[-] ecall_async_executor_run failed {}!", result);
            }
        })
        .expect("Failed to spawn async-executor");

    let bench_cores: u32 = args.cores.unwrap_or_else(|| num_cpus::get() as _);
    info!("Bench cores: {}", bench_cores);

    let rocket = thread::Builder::new()
        .name("rocket".into())
        .spawn(move || {
            rocket().launch();
        })
        .expect("Failed to launch Rocket");

    let mut v = vec![];
    for i in 0..bench_cores {
        let child = thread::Builder::new()
            .name(format!("bench-{}", i))
            .spawn(move || {
                set_thread_idle_policy();
                loop {
                    let result = unsafe { ecall_bench_run(eid, &mut retval, i) };
                    if result != sgx_status_t::SGX_SUCCESS {
                        panic!("Run benchmark {} failed", i);
                    }
                    std::thread::sleep(std::time::Duration::from_millis(200));
                }
            })
            .expect("Failed to launch benchmark thread");
        v.push(child);
    }

    let _ = rocket.join();
    for child in v {
        let _ = child.join();
    }

    info!("Quit signal received, destroying enclave...");
    destroy_enclave();

    std::process::exit(0);
}

fn set_thread_idle_policy() {
    let param = libc::sched_param { sched_priority: 0 };
    unsafe {
        let rv = libc::sched_setscheduler(0, libc::SCHED_IDLE, &param);
        if rv != 0 {
            error!("Failed to set thread schedule prolicy to IDLE");
        }
    }
}

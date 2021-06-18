#![feature(decl_macro)]

extern crate env_logger;
extern crate sgx_types;
extern crate sgx_urts;
extern crate mio;
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
extern crate enclave_api;

#[cfg(test)]
mod tests;
#[cfg(test)]
extern crate ring_compat;
#[cfg(test)]
extern crate base64;
#[cfg(test)]
extern crate hex_literal;

mod attestation;
mod contract_input;
mod contract_output;

use sgx_types::*;
use sgx_urts::SgxEnclave;

use std::fs;
use std::path;
use std::net::SocketAddr;
use std::str;
use std::io::Read;
use std::sync::{Arc, RwLock};
use std::env;

use rocket::http::Method;
use rocket_contrib::json::{Json, JsonValue};
use rocket_cors::{AllowedHeaders, AllowedOrigins, AllowedMethods, Cors, CorsOptions};

use contract_input::ContractInput;
use contract_output::ContractOutput;
use attestation::Attestation;
use enclave_api::actions;

static ENCLAVE_FILE: &'static str = "enclave.signed.so";
static ENCLAVE_STATE_FILE: &'static str = "enclave.token";

const ENCLAVE_OUTPUT_BUF_MAX_LEN: usize = 2*2048*1024 as usize;

lazy_static! {
    static ref ENCLAVE: RwLock<Option<SgxEnclave>> = RwLock::new(None);
    static ref ENCLAVE_STATE_FILE_PATH: &'static str = {
        Box::leak(
            env::var("STATE_FILE_PATH").unwrap_or_else(|_| "./".to_string()).into_boxed_str()
        )
    };
    static ref ALLOW_CORS: bool = {
        env::var("ALLOW_CORS").unwrap_or_else(|_| "".to_string()) != ""
    };
    static ref ENABLE_KICK_API: bool = {
        env::var("ENABLE_KICK_API").unwrap_or_else(|_| "".to_string()) != ""
    };
}

fn destroy_enclave() {
    let enclave = ENCLAVE.write().unwrap().take().unwrap();
    enclave.destroy();
}

fn get_eid() -> u64 {
    ENCLAVE.read().unwrap().as_ref().unwrap().geteid()
}

extern {
    fn ecall_handle(
        eid: sgx_enclave_id_t, retval: *mut sgx_status_t,
        action: u8,
        input_ptr: *const u8, input_len: usize,
        output_ptr : *mut u8, output_len_ptr: *mut usize, output_buf_len: usize
    ) -> sgx_status_t;

    fn ecall_init(
        eid: sgx_enclave_id_t, retval: *mut sgx_status_t
    ) -> sgx_status_t;
}

const IAS_SPID_STR: &str = env!("IAS_SPID");
const IAS_API_KEY_STR: &str = env!("IAS_API_KEY");

#[no_mangle]
pub extern "C"
fn ocall_load_ias_spid(
    key_ptr : *mut u8,
    key_len_ptr: *mut usize,
    key_buf_len: usize
) -> sgx_status_t {
    let key_len = IAS_SPID_STR.len();

    unsafe {
        if key_len <= key_buf_len {
            std::ptr::copy_nonoverlapping(IAS_SPID_STR.as_ptr(),
                                          key_ptr,
                                          key_len);
        } else {
            panic!("IAS_SPID_STR too long. Buffer overflow.");
        }
        std::ptr::copy_nonoverlapping(&key_len as *const usize,
                                      key_len_ptr,
                                      std::mem::size_of_val(&key_len));
    }

    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub extern "C"
fn ocall_load_ias_key(
    key_ptr : *mut u8,
    key_len_ptr: *mut usize,
    key_buf_len: usize
) -> sgx_status_t {
    let key_len = IAS_API_KEY_STR.len();

    unsafe {
        if key_len <= key_buf_len {
            std::ptr::copy_nonoverlapping(IAS_API_KEY_STR.as_ptr(),
                                          key_ptr,
                                          key_len);
        } else {
            panic!("IAS_API_KEY_STR too long. Buffer overflow.");
        }
        std::ptr::copy_nonoverlapping(&key_len as *const usize,
                                      key_len_ptr,
                                      std::mem::size_of_val(&key_len));
    }

    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub extern "C"
fn ocall_sgx_init_quote(ret_ti: *mut sgx_target_info_t,
                        ret_gid : *mut sgx_epid_group_id_t) -> sgx_status_t {
    info!("Entering ocall_sgx_init_quote");
    unsafe { sgx_init_quote(ret_ti, ret_gid) }
}

#[no_mangle]
pub extern "C"
fn ocall_get_quote (p_sigrl            : *const u8,
                    sigrl_len          : u32,
                    p_report           : *const sgx_report_t,
                    quote_type         : sgx_quote_sign_type_t,
                    p_spid             : *const sgx_spid_t,
                    p_nonce            : *const sgx_quote_nonce_t,
                    p_qe_report        : *mut sgx_report_t,
                    p_quote            : *mut u8,
                    _maxlen            : u32,
                    p_quote_len        : *mut u32) -> sgx_status_t {
    info!("Entering ocall_get_quote");

    let mut real_quote_len : u32 = 0;

    let ret = unsafe {
        sgx_calc_quote_size(p_sigrl, sigrl_len, &mut real_quote_len as *mut u32)
    };

    if ret != sgx_status_t::SGX_SUCCESS {
        warn!("sgx_calc_quote_size returned {}", ret);
        return ret;
    }

    info!("quote size = {}", real_quote_len);
    unsafe { *p_quote_len = real_quote_len; }

    let ret = unsafe {
        sgx_get_quote(p_report,
                      quote_type,
                      p_spid,
                      p_nonce,
                      p_sigrl,
                      sigrl_len,
                      p_qe_report,
                      p_quote as *mut sgx_quote_t,
                      real_quote_len)
    };

    if ret != sgx_status_t::SGX_SUCCESS {
        warn!("sgx_calc_quote_size returned {}", ret);
        return ret;
    }

    info!("sgx_calc_quote_size returned {}", ret);
    ret
}

#[no_mangle]
pub extern "C"
fn ocall_get_update_info(
    platform_blob: * const sgx_platform_info_t,
    enclave_trusted: i32,
    update_info: * mut sgx_update_info_bit_t
) -> sgx_status_t {
    unsafe{
        sgx_report_attestation_status(platform_blob, enclave_trusted, update_info)
    }
}

#[no_mangle]
pub extern "C"
fn ocall_dump_state(
    _output_ptr : *mut u8,
    _output_len_ptr: *mut usize,
    _output_buf_len: usize
) -> sgx_status_t {
    // TODO:

    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub extern "C"
fn ocall_save_persistent_data(
    input_ptr: *const u8,
    input_len: usize
) -> sgx_status_t {
    let input_slice = unsafe { std::slice::from_raw_parts(input_ptr, input_len) };
    debug!("Sealed data {:}: {:?}", input_len, hex::encode(input_slice));

    let executable = env::current_exe().unwrap();
    let path = executable.parent().unwrap();
    let state_path: path::PathBuf = path.join(*ENCLAVE_STATE_FILE_PATH).join(ENCLAVE_STATE_FILE);
    info!("Save seal data to {}", state_path.as_path().to_str().unwrap());

    fs::write(state_path.as_path().to_str().unwrap(), input_slice)
        .expect("Failed to write persistent data");

    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub extern "C"
fn ocall_load_persistent_data(
    output_ptr : *mut u8,
    output_len_ptr: *mut usize,
    output_buf_len: usize
) -> sgx_status_t {
    let executable = env::current_exe().unwrap();
    let path = executable.parent().unwrap();
    let state_path: path::PathBuf = path.join(*ENCLAVE_STATE_FILE_PATH).join(ENCLAVE_STATE_FILE);

    let state = match fs::read(state_path.as_path().to_str().unwrap()) {
        Ok(data) => data,
        _ => Vec::<u8>::new()
    };
    let state_len = state.len();

    if state_len == 0 {
        return sgx_status_t::SGX_SUCCESS
    }

    info!("Loaded sealed data {:}: {:?}", state_len, state);

    unsafe {
        if state_len <= output_buf_len {
            std::ptr::copy_nonoverlapping(state.as_ptr(),
                                          output_ptr,
                                          state_len);
        } else {
            panic!("State too long. Buffer overflow.");
        }
        std::ptr::copy_nonoverlapping(&state_len as *const usize,
                                      output_len_ptr,
                                      std::mem::size_of_val(&state_len));
    }

    sgx_status_t::SGX_SUCCESS
}

fn init_enclave() -> SgxResult<SgxEnclave> {
    let mut launch_token: sgx_launch_token_t = [0; 1024];
    let mut launch_token_updated: i32 = 0;
    // call sgx_create_enclave to initialize an enclave instance
    // Debug Support: set 2nd parameter to 1
    let debug = option_env!("SGX_DEBUG").unwrap_or("1");

    let mut misc_attr = sgx_misc_attribute_t {secs_attr: sgx_attributes_t {flags:0, xfrm:0}, misc_select:0};
    SgxEnclave::create(ENCLAVE_FILE,
                       if debug == "0" { 0 } else { 1 },
                       &mut launch_token,
                       &mut launch_token_updated,
                       &mut misc_attr)
}

macro_rules! delegate_rpc {
    ($rpc: literal, $name: ident, $num: expr) => {
        #[post($rpc, format = "json", data = "<contract_input>")]
        fn $name(contract_input: Json<ContractInput>) -> JsonValue {
            debug!("{}", ::serde_json::to_string_pretty(&*contract_input).unwrap());

            let eid = get_eid();

            let input_string = serde_json::to_string(&*contract_input).unwrap();

            let mut return_output_buf = vec![0; ENCLAVE_OUTPUT_BUF_MAX_LEN].into_boxed_slice();
            let mut output_len : usize = 0;
            let output_slice = &mut return_output_buf;
            let output_ptr = output_slice.as_mut_ptr();
            let output_len_ptr = &mut output_len as *mut usize;

            let mut retval = sgx_status_t::SGX_SUCCESS;
            let result = unsafe {
                ecall_handle(
                    eid, &mut retval,
                    $num,
                    input_string.as_ptr(), input_string.len(),
                    output_ptr, output_len_ptr, ENCLAVE_OUTPUT_BUF_MAX_LEN
                )
            };

            match result {
                sgx_status_t::SGX_SUCCESS => {
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
        }
    };
}

delegate_rpc!("/test", test, actions::ACTION_TEST);
delegate_rpc!("/init_runtime", init_runtime, actions::ACTION_INIT_RUNTIME);
delegate_rpc!("/get_info", get_info, actions::ACTION_GET_INFO);
delegate_rpc!("/get_runtime_info", get_runtime_info, actions::ACTION_GET_RUNTIME_INFO);
delegate_rpc!("/dump_states", dump_states, actions::ACTION_DUMP_STATES);
delegate_rpc!("/load_states", load_states, actions::ACTION_LOAD_STATES);
delegate_rpc!("/sync_header", sync_header, actions::ACTION_SYNC_HEADER);
delegate_rpc!("/query", query, actions::ACTION_QUERY);
delegate_rpc!("/dispatch_block", dispatch_block, actions::ACTION_DISPATCH_BLOCK);
delegate_rpc!("/set", set, actions::ACTION_SET);
delegate_rpc!("/get", get, actions::ACTION_GET);
// TODO.kevin: becareful the limitation of ENCLAVE_OUTPUT_BUF_MAX_LEN
delegate_rpc!("/get_egress_messages", get_egress_messages, actions::ACTION_GET_EGRESS_MESSAGES);
delegate_rpc!("/test_ink", test_ink, actions::ACTION_TEST_INK);

#[post("/kick")]
fn kick() {
    // TODO: we should improve this
    info!("Kick API received, destroying enclave...");
    destroy_enclave();

    std::process::exit(0);
}

fn cors_options() -> CorsOptions {
    let allowed_origins = AllowedOrigins::all();
    let allowed_methods: AllowedMethods = vec![Method::Get, Method::Post].into_iter().map(From::from).collect();

    // You can also deserialize this
    rocket_cors::CorsOptions {
        allowed_origins,
        allowed_methods,
        allowed_headers: AllowedHeaders::all(),
        allow_credentials: true,
        ..Default::default()
    }
}

fn rocket() -> rocket::Rocket {
    let mut server = rocket::ignite()
        .mount("/", routes![
            test, init_runtime, get_info,
            dump_states, load_states,
            sync_header, dispatch_block, query,
            set, get, get_runtime_info, test_ink]);

    if *ENABLE_KICK_API {
        info!("ENABLE `kick` API");

        server = server.mount("/", routes![kick]);
    }

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
    env::set_var("RUST_BACKTRACE", "1");
    env::set_var("ROCKET_ENV", "dev");

    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    let enclave = match init_enclave() {
        Ok(r) => {
            info!("[+] Init Enclave Successful {}!", r.geteid());
            r
        },
        Err(x) => {
            panic!("[-] Init Enclave Failed {}!", x.as_str());
        },
    };

    ENCLAVE.write().unwrap().replace(enclave);

    let eid = get_eid();
    let mut retval = sgx_status_t::SGX_SUCCESS;
    let result = unsafe {
        ecall_init(eid, &mut retval)
    };

    if result != sgx_status_t::SGX_SUCCESS {
        panic!("Initialize Failed");
    }

    rocket().launch();

    info!("Quit signal received, destroying enclave...");
    destroy_enclave();

    std::process::exit(0);
}

#![feature(proc_macro_hygiene, decl_macro)]

extern crate sgx_types;
extern crate sgx_urts;
extern crate dirs;
extern crate mio;
#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
extern crate rocket_cors;

extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

mod attestation;
mod contract_input;
mod contract_output;

use sgx_types::*;
use sgx_urts::SgxEnclave;

use std::os::unix::io::{IntoRawFd};
use std::fs;
use std::path;
use std::net::{TcpStream, SocketAddr};
use std::str;
use std::io::{Read, Write};
use std::sync::{Arc, RwLock};
use std::env;

use rocket::http::Method;
use rocket_contrib::json::{Json, JsonValue};
use rocket_cors::{AllowedHeaders, AllowedOrigins, AllowedMethods, Cors, CorsOptions};

use contract_input::ContractInput;
use contract_output::ContractOutput;
use attestation::Attestation;

static ENCLAVE_FILE: &'static str = "enclave.signed.so";
static ENCLAVE_TOKEN: &'static str = "enclave.token";

const ENCLAVE_OUTPUT_BUF_MAX_LEN: usize = 32760 as usize;

lazy_static! {
    static ref ENCLAVE: RwLock<Option<SgxEnclave>> = RwLock::new(None);
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

    fn ecall_set_state(
        eid: sgx_enclave_id_t, retval: *mut sgx_status_t,
        input_ptr: *const u8, input_len: usize,
    ) -> sgx_status_t;
}

pub fn lookup_ipv4(host: &str, port: u16) -> SocketAddr {
    use std::net::ToSocketAddrs;

    let addrs = (host, port).to_socket_addrs().unwrap();
    for addr in addrs {
        if let SocketAddr::V4(_) = addr {
            return addr;
        }
    }

    unreachable!("Cannot lookup address");
}

#[no_mangle]
pub extern "C"
fn ocall_sgx_init_quote(ret_ti: *mut sgx_target_info_t,
                        ret_gid : *mut sgx_epid_group_id_t) -> sgx_status_t {
    println!("Entering ocall_sgx_init_quote");
    unsafe {sgx_init_quote(ret_ti, ret_gid)}
}

#[no_mangle]
pub extern "C"
fn ocall_get_ias_socket(ret_fd : *mut c_int) -> sgx_status_t {
    let port = 443;
    let hostname = "api.trustedservices.intel.com";
    let addr = lookup_ipv4(hostname, port);
    let sock = TcpStream::connect(&addr).expect("[-] Connect tls server failed!");

    unsafe {*ret_fd = sock.into_raw_fd();}

    sgx_status_t::SGX_SUCCESS
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
    println!("Entering ocall_get_quote");

    let mut real_quote_len : u32 = 0;

    let ret = unsafe {
        sgx_calc_quote_size(p_sigrl, sigrl_len, &mut real_quote_len as *mut u32)
    };

    if ret != sgx_status_t::SGX_SUCCESS {
        println!("sgx_calc_quote_size returned {}", ret);
        return ret;
    }

    println!("quote size = {}", real_quote_len);
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
        println!("sgx_calc_quote_size returned {}", ret);
        return ret;
    }

    println!("sgx_calc_quote_size returned {}", ret);
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
    output_ptr : *mut u8,
    output_len_ptr: *mut usize,
    output_buf_len: usize
) -> sgx_status_t {
    // TODO:

    sgx_status_t::SGX_SUCCESS
}


fn init_enclave() -> SgxResult<SgxEnclave> {
    let mut launch_token: sgx_launch_token_t = [0; 1024];
    let mut launch_token_updated: i32 = 0;
    // Step 1: try to retrieve the launch token saved by last transaction
    //         if there is no token, then create a new one.
    //
    // try to get the token saved in $HOME */
    let mut home_dir = path::PathBuf::new();
    let use_token = match dirs::home_dir() {
        Some(path) => {
            println!("[+] Home dir is {}", path.display());
            home_dir = path;
            true
        },
        None => {
            println!("[-] Cannot get home dir");
            false
        }
    };

    let token_file: path::PathBuf = home_dir.join(ENCLAVE_TOKEN);
    if use_token == true {
        match fs::File::open(&token_file) {
            Err(_) => {
                println!("[-] Open token file {} error! Will create one.", token_file.as_path().to_str().unwrap());
            },
            Ok(mut f) => {
                println!("[+] Open token file success! ");
                match f.read(&mut launch_token) {
                    Ok(1024) => {
                        println!("[+] Token file valid!");
                    },
                    _ => println!("[+] Token file invalid, will create new token file"),
                }
            }
        }
    }

    // Step 2: call sgx_create_enclave to initialize an enclave instance
    // Debug Support: set 2nd parameter to 1
    let debug = 1;
    let mut misc_attr = sgx_misc_attribute_t {secs_attr: sgx_attributes_t { flags:0, xfrm:0}, misc_select:0};
    let enclave = SgxEnclave::create(
        ENCLAVE_FILE,
        debug,
        &mut launch_token,
        &mut launch_token_updated,
        &mut misc_attr
    )?;

    // Step 3: save the launch token if it is updated
    if use_token == true && launch_token_updated != 0 {
        // reopen the file with write capablity
        match fs::File::create(&token_file) {
            Ok(mut f) => {
                match f.write_all(&launch_token) {
                    Ok(()) => println!("[+] Saved updated launch token!"),
                    Err(_) => println!("[-] Failed to save updated launch token!"),
                }
            },
            Err(_) => {
                println!("[-] Failed to save updated enclave token, but doesn't matter");
            },
        }
    }

    Ok(enclave)
}

#[post("/test", format = "json", data = "<contract_input>")]
fn test(contract_input: Json<ContractInput>) -> JsonValue {
    println!("{}", ::serde_json::to_string_pretty(&*contract_input).unwrap());

    let eid = get_eid();
    let mut retval = sgx_status_t::SGX_SUCCESS;

    let input_string = serde_json::to_string(&*contract_input).unwrap();
    let mut return_output_buf: [u8; ENCLAVE_OUTPUT_BUF_MAX_LEN] = [0; ENCLAVE_OUTPUT_BUF_MAX_LEN];
    let mut output_len : usize = 0;
    let output_slice = &mut return_output_buf;
    let output_ptr = output_slice.as_mut_ptr();
    let output_len_ptr = &mut output_len as *mut usize;

    let mut retval = sgx_status_t::SGX_SUCCESS;
    let result = unsafe {
        ecall_handle(
            eid, &mut retval,
            0,
            input_string.as_ptr(), input_string.len(),
            output_ptr, output_len_ptr, ENCLAVE_OUTPUT_BUF_MAX_LEN
        )
    };

    let output_slice = unsafe { std::slice::from_raw_parts(output_ptr, output_len) };
    let output_value: serde_json::value::Value = serde_json::from_slice(output_slice).unwrap();

    match result {
        sgx_status_t::SGX_SUCCESS => {
            json!(output_value)
        },
        _ => {
            println!("[-] ECALL Enclave Failed {}!", result.as_str());
            json!({
                "status": "error",
                "payload": format!("[-] ECALL Enclave Failed {}!", result.as_str())
            })
        }
    }
}

#[post("/init_runtime", format = "json", data = "<contract_input>")]
fn init_runtime(contract_input: Json<ContractInput>) -> JsonValue {
    println!("{}", ::serde_json::to_string_pretty(&*contract_input).unwrap());

    let eid = get_eid();
    let mut retval = sgx_status_t::SGX_SUCCESS;

    let input_string = serde_json::to_string(&*contract_input).unwrap();
    let mut return_output_buf: [u8; ENCLAVE_OUTPUT_BUF_MAX_LEN] = [0; ENCLAVE_OUTPUT_BUF_MAX_LEN];
    let mut output_len : usize = 0;
    let output_slice = &mut return_output_buf;
    let output_ptr = output_slice.as_mut_ptr();
    let output_len_ptr = &mut output_len as *mut usize;

    let mut retval = sgx_status_t::SGX_SUCCESS;
    let result = unsafe {
        ecall_handle(
            eid, &mut retval,
            1,
            input_string.as_ptr(), input_string.len(),
            output_ptr, output_len_ptr, ENCLAVE_OUTPUT_BUF_MAX_LEN
        )
    };

    let output_slice = unsafe { std::slice::from_raw_parts(output_ptr, output_len) };
    let output_value: serde_json::value::Value = serde_json::from_slice(output_slice).unwrap();

    match result {
        sgx_status_t::SGX_SUCCESS => {
            json!(output_value)
        },
        _ => {
            println!("[-] ECALL Enclave Failed {}!", result.as_str());
            json!({
                "status": "error",
                "payload": format!("[-] ECALL Enclave Failed {}!", result.as_str())
            })
        }
    }
}

#[post("/get_info", format = "json", data = "<contract_input>")]
fn get_info(contract_input: Json<ContractInput>) -> JsonValue {
    println!("{}", ::serde_json::to_string_pretty(&*contract_input).unwrap());

    let eid = get_eid();
    let mut retval = sgx_status_t::SGX_SUCCESS;

    let input_string = serde_json::to_string(&*contract_input).unwrap();
    let mut return_output_buf: [u8; ENCLAVE_OUTPUT_BUF_MAX_LEN] = [0; ENCLAVE_OUTPUT_BUF_MAX_LEN];
    let mut output_len : usize = 0;
    let output_slice = &mut return_output_buf;
    let output_ptr = output_slice.as_mut_ptr();
    let output_len_ptr = &mut output_len as *mut usize;

    let mut retval = sgx_status_t::SGX_SUCCESS;
    let result = unsafe {
        ecall_handle(
            eid, &mut retval,
            2,
            input_string.as_ptr(), input_string.len(),
            output_ptr, output_len_ptr, ENCLAVE_OUTPUT_BUF_MAX_LEN
        )
    };

    let output_slice = unsafe { std::slice::from_raw_parts(output_ptr, output_len) };
    let output_value: serde_json::value::Value = serde_json::from_slice(output_slice).unwrap();

    match result {
        sgx_status_t::SGX_SUCCESS => {
            json!(output_value)
        },
        _ => {
            println!("[-] ECALL Enclave Failed {}!", result.as_str());
            json!({
                "status": "error",
                "payload": format!("[-] ECALL Enclave Failed {}!", result.as_str())
            })
        }
    }
}

#[post("/dump_states", format = "json", data = "<contract_input>")]
fn dump_states(contract_input: Json<ContractInput>) -> JsonValue {
    println!("{}", ::serde_json::to_string_pretty(&*contract_input).unwrap());

    let eid = get_eid();
    let mut retval = sgx_status_t::SGX_SUCCESS;

    let input_string = serde_json::to_string(&*contract_input).unwrap();
    let mut return_output_buf: [u8; ENCLAVE_OUTPUT_BUF_MAX_LEN] = [0; ENCLAVE_OUTPUT_BUF_MAX_LEN];
    let mut output_len : usize = 0;
    let output_slice = &mut return_output_buf;
    let output_ptr = output_slice.as_mut_ptr();
    let output_len_ptr = &mut output_len as *mut usize;

    let mut retval = sgx_status_t::SGX_SUCCESS;
    let result = unsafe {
        ecall_handle(
            eid, &mut retval,
            3,
            input_string.as_ptr(), input_string.len(),
            output_ptr, output_len_ptr, ENCLAVE_OUTPUT_BUF_MAX_LEN
        )
    };

    let output_slice = unsafe { std::slice::from_raw_parts(output_ptr, output_len) };
    let output_value: serde_json::value::Value = serde_json::from_slice(output_slice).unwrap();

    match result {
        sgx_status_t::SGX_SUCCESS => {
            json!(output_value)
        },
        _ => {
            println!("[-] ECALL Enclave Failed {}!", result.as_str());
            json!({
                "status": "error",
                "payload": format!("[-] ECALL Enclave Failed {}!", result.as_str())
            })
        }
    }
}

#[post("/load_states", format = "json", data = "<contract_input>")]
fn load_states(contract_input: Json<ContractInput>) -> JsonValue {
    println!("{}", ::serde_json::to_string_pretty(&*contract_input).unwrap());

    let eid = get_eid();
    let mut retval = sgx_status_t::SGX_SUCCESS;

    let input_string = serde_json::to_string(&*contract_input).unwrap();
    let mut return_output_buf: [u8; ENCLAVE_OUTPUT_BUF_MAX_LEN] = [0; ENCLAVE_OUTPUT_BUF_MAX_LEN];
    let mut output_len : usize = 0;
    let output_slice = &mut return_output_buf;
    let output_ptr = output_slice.as_mut_ptr();
    let output_len_ptr = &mut output_len as *mut usize;

    let mut retval = sgx_status_t::SGX_SUCCESS;
    let result = unsafe {
        ecall_handle(
            eid, &mut retval,
            4,
            input_string.as_ptr(), input_string.len(),
            output_ptr, output_len_ptr, ENCLAVE_OUTPUT_BUF_MAX_LEN
        )
    };

    let output_slice = unsafe { std::slice::from_raw_parts(output_ptr, output_len) };
    let output_value: serde_json::value::Value = serde_json::from_slice(output_slice).unwrap();

    match result {
        sgx_status_t::SGX_SUCCESS => {
            json!(output_value)
        },
        _ => {
            println!("[-] ECALL Enclave Failed {}!", result.as_str());
            json!({
                "status": "error",
                "payload": format!("[-] ECALL Enclave Failed {}!", result.as_str())
            })
        }
    }
}

#[post("/sync_block", format = "json", data = "<contract_input>")]
fn sync_block(contract_input: Json<ContractInput>) -> JsonValue {
    println!("{}", ::serde_json::to_string_pretty(&*contract_input).unwrap());

    let eid = get_eid();
    let mut retval = sgx_status_t::SGX_SUCCESS;

    let input_string = serde_json::to_string(&*contract_input).unwrap();
    let mut return_output_buf: [u8; ENCLAVE_OUTPUT_BUF_MAX_LEN] = [0; ENCLAVE_OUTPUT_BUF_MAX_LEN];
    let mut output_len : usize = 0;
    let output_slice = &mut return_output_buf;
    let output_ptr = output_slice.as_mut_ptr();
    let output_len_ptr = &mut output_len as *mut usize;

    let mut retval = sgx_status_t::SGX_SUCCESS;
    let result = unsafe {
        ecall_handle(
            eid, &mut retval,
            5,
            input_string.as_ptr(), input_string.len(),
            output_ptr, output_len_ptr, ENCLAVE_OUTPUT_BUF_MAX_LEN
        )
    };

    let output_slice = unsafe { std::slice::from_raw_parts(output_ptr, output_len) };
    let output_value: serde_json::value::Value = serde_json::from_slice(output_slice).unwrap();

    match result {
        sgx_status_t::SGX_SUCCESS => {
            json!(output_value)
        },
        _ => {
            println!("[-] ECALL Enclave Failed {}!", result.as_str());
            json!({
                "status": "error",
                "payload": format!("[-] ECALL Enclave Failed {}!", result.as_str())
            })
        }
    }
}

#[post("/sync_events", format = "json", data = "<contract_input>")]
fn sync_events(contract_input: Json<ContractInput>) -> JsonValue {
	println!("{}", ::serde_json::to_string_pretty(&*contract_input).unwrap());

	let eid = get_eid();
	let mut retval = sgx_status_t::SGX_SUCCESS;

	let input_string = serde_json::to_string(&*contract_input).unwrap();
	let mut return_output_buf: [u8; ENCLAVE_OUTPUT_BUF_MAX_LEN] = [0; ENCLAVE_OUTPUT_BUF_MAX_LEN];
	let mut output_len : usize = 0;
	let output_slice = &mut return_output_buf;
	let output_ptr = output_slice.as_mut_ptr();
	let output_len_ptr = &mut output_len as *mut usize;

	let mut retval = sgx_status_t::SGX_SUCCESS;
	let result = unsafe {
		ecall_handle(
			eid, &mut retval,
			7,
			input_string.as_ptr(), input_string.len(),
			output_ptr, output_len_ptr, ENCLAVE_OUTPUT_BUF_MAX_LEN
		)
	};

	let output_slice = unsafe { std::slice::from_raw_parts(output_ptr, output_len) };
	let output_value: serde_json::value::Value = serde_json::from_slice(output_slice).unwrap();

	match result {
		sgx_status_t::SGX_SUCCESS => {
			json!(output_value)
		},
		_ => {
			println!("[-] ECALL Enclave Failed {}!", result.as_str());
			json!({
                "status": "error",
                "payload": format!("[-] ECALL Enclave Failed {}!", result.as_str())
            })
		}
	}
}

#[post("/query", format = "json", data = "<contract_input>")]
fn query(contract_input: Json<ContractInput>) -> JsonValue {
    println!("{}", ::serde_json::to_string_pretty(&*contract_input).unwrap());

    let eid = get_eid();
    let mut retval = sgx_status_t::SGX_SUCCESS;

    let input_string = serde_json::to_string(&*contract_input).unwrap();

    let mut return_output_buf: [u8; ENCLAVE_OUTPUT_BUF_MAX_LEN] = [0; ENCLAVE_OUTPUT_BUF_MAX_LEN];
    let mut output_len : usize = 0;
    let output_slice = &mut return_output_buf;
    let output_ptr = output_slice.as_mut_ptr();
    let output_len_ptr = &mut output_len as *mut usize;

    let mut retval = sgx_status_t::SGX_SUCCESS;
    let result = unsafe {
        ecall_handle(
            eid, &mut retval,
            6,
            input_string.as_ptr(), input_string.len(),
            output_ptr, output_len_ptr, ENCLAVE_OUTPUT_BUF_MAX_LEN
        )
    };

    let output_slice = unsafe { std::slice::from_raw_parts(output_ptr, output_len) };
    let output_value: serde_json::value::Value = serde_json::from_slice(output_slice).unwrap();

    match result {
        sgx_status_t::SGX_SUCCESS => {
            json!(output_value)
        },
        _ => {
            println!("[-] ECALL Enclave Failed {}!", result.as_str());
            json!({
                "status": "error",
                "payload": format!("[-] ECALL Enclave Failed {}!", result.as_str())
            })
        }
    }
}

#[post("/set", format = "json", data = "<contract_input>")]
fn set(contract_input: Json<ContractInput>) -> JsonValue {
    println!("{}", ::serde_json::to_string_pretty(&*contract_input).unwrap());

    let eid = get_eid();
    let mut retval = sgx_status_t::SGX_SUCCESS;

    let input_string = serde_json::to_string(&*contract_input).unwrap();

    let mut return_output_buf: [u8; ENCLAVE_OUTPUT_BUF_MAX_LEN] = [0; ENCLAVE_OUTPUT_BUF_MAX_LEN];
    let mut output_len : usize = 0;
    let output_slice = &mut return_output_buf;
    let output_ptr = output_slice.as_mut_ptr();
    let output_len_ptr = &mut output_len as *mut usize;

    let mut retval = sgx_status_t::SGX_SUCCESS;
    let result = unsafe {
        ecall_handle(
            eid, &mut retval,
            21,
            input_string.as_ptr(), input_string.len(),
            output_ptr, output_len_ptr, ENCLAVE_OUTPUT_BUF_MAX_LEN
        )
    };

    let output_slice = unsafe { std::slice::from_raw_parts(output_ptr, output_len) };
    let output_value: serde_json::value::Value = serde_json::from_slice(output_slice).unwrap();

    match result {
        sgx_status_t::SGX_SUCCESS => {
            json!(output_value)
        },
        _ => {
            println!("[-] ECALL Enclave Failed {}!", result.as_str());
            json!({
                "status": "error",
                "payload": format!("[-] ECALL Enclave Failed {}!", result.as_str())
            })
        }
    }
}

#[post("/get", format = "json", data = "<contract_input>")]
fn get(contract_input: Json<ContractInput>) -> JsonValue {
    println!("{}", ::serde_json::to_string_pretty(&*contract_input).unwrap());

    let eid = get_eid();
    let mut retval = sgx_status_t::SGX_SUCCESS;

    let input_string = serde_json::to_string(&*contract_input).unwrap();

    let mut return_output_buf: [u8; ENCLAVE_OUTPUT_BUF_MAX_LEN] = [0; ENCLAVE_OUTPUT_BUF_MAX_LEN];
    let mut output_len : usize = 0;
    let output_slice = &mut return_output_buf;
    let output_ptr = output_slice.as_mut_ptr();
    let output_len_ptr = &mut output_len as *mut usize;

    let mut retval = sgx_status_t::SGX_SUCCESS;
    let result = unsafe {
        ecall_handle(
            eid, &mut retval,
            22,
            input_string.as_ptr(), input_string.len(),
            output_ptr, output_len_ptr, ENCLAVE_OUTPUT_BUF_MAX_LEN
        )
    };

    let output_slice = unsafe { std::slice::from_raw_parts(output_ptr, output_len) };
    let output_value: serde_json::value::Value = serde_json::from_slice(output_slice).unwrap();

    match result {
        sgx_status_t::SGX_SUCCESS => {
            json!(output_value)
        },
        _ => {
            println!("[-] ECALL Enclave Failed {}!", result.as_str());
            json!({
                "status": "error",
                "payload": format!("[-] ECALL Enclave Failed {}!", result.as_str())
            })
        }
    }
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
    rocket::ignite()
        .mount("/", routes![
            test, init_runtime, get_info,
            dump_states, load_states,
            sync_block, sync_events, query,
            set, get])
        .attach(cors_options().to_cors().expect("To not fail"))
    // .mount("/", rocket_cors::catch_all_options_routes()) // mount the catch all routes
    // .manage(cors_options().to_cors().expect("To not fail"))
}

fn main() { ;
    env::set_var("RUST_BACKTRACE", "1");
    env::set_var("ROCKET_ENV", "dev");

    let enclave = match init_enclave() {
        Ok(r) => {
            println!("[+] Init Enclave Successful {}!", r.geteid());
            r
        },
        Err(x) => {
            panic!("[-] Init Enclave Failed {}!", x.as_str());
        },
    };

    ENCLAVE.write().unwrap().replace(enclave);

    rocket().launch();

    println!("Quit signal received, destroying enclave...");
    destroy_enclave();

    std::process::exit(0);
}

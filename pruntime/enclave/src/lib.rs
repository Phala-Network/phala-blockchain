#![crate_name = "enclaveapp"]
#![crate_type = "staticlib"]

#![warn(unused_imports)]
#![warn(unused_extern_crates)]

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

#[cfg(not(target_env = "sgx"))]
#[macro_use] extern crate sgx_tstd as std;

// #[macro_use] extern crate serde;
#[macro_use] extern crate serde_json;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;

extern crate runtime as chain;

use sgx_types::*;
use sgx_tse::*;
use sgx_tcrypto::*;
use sgx_rand::*;

use crate::std::prelude::v1::*;
use crate::std::sync::Arc;
use crate::std::net::TcpStream;
use crate::std::string::String;
use crate::std::ptr;
use crate::std::str;
use crate::std::io::{Write, Read};
use crate::std::untrusted::fs;
use crate::std::vec::Vec;
use crate::std::sync::SgxMutex;
use itertools::Itertools;
use serde::{de, Serialize, Deserialize, Serializer, Deserializer};
use serde_json::{Map, Value};
use parity_scale_codec::{Encode, Decode};
use secp256k1::{SecretKey, PublicKey};
use contracts::AccountIdWrapper;
use sp_core::hashing::blake2_256;
use sp_core::H256 as Hash;
use system::{EventRecord};
use phala::{RawEvent};

mod cert;
mod hex;
mod light_validation;
mod cryptography;

use light_validation::AuthoritySetChange;
use cryptography::{ecdh, aead};
mod receipt;
use receipt::{TransactionStatus, TransactionReceipt, ReceiptStore, Request, Response, Error};

extern "C" {
    pub fn ocall_sgx_init_quote(
        ret_val: *mut sgx_status_t,
        ret_ti: *mut sgx_target_info_t,
        ret_gid: *mut sgx_epid_group_id_t
    ) -> sgx_status_t;

    pub fn ocall_get_ias_socket(
        ret_val: *mut sgx_status_t,
        ret_fd: *mut i32
    ) -> sgx_status_t;

    pub fn ocall_get_quote(
        ret_val: *mut sgx_status_t,
        p_sigrl: *const u8,
        sigrl_len: u32,
        p_report: *const sgx_report_t,
        quote_type: sgx_quote_sign_type_t,
        p_spid: *const sgx_spid_t,
        p_nonce: *const sgx_quote_nonce_t,
        p_qe_report: *mut sgx_report_t,
        p_quote: *mut u8,
        maxlen: u32,
        p_quote_len: *mut u32
    ) -> sgx_status_t;

    pub fn ocall_dump_state(
        ret_val: *mut sgx_status_t,
        output_ptr : *mut u8,
        output_len_ptr: *mut usize,
        output_buf_len: usize
    ) -> sgx_status_t;
}

type ChainLightValidation = light_validation::LightValidation::<chain::Runtime>;
type EcdhKey = ring::agreement::EphemeralPrivateKey;

#[derive(Serialize, Deserialize, Debug)]
struct RuntimeState {
    contract1: contracts::data_plaza::DataPlaza,
    contract2: contracts::balance::Balance,
    contract3: contracts::assets::Assets,
    #[serde(serialize_with = "se_to_b64", deserialize_with = "de_from_b64")]
    light_client: ChainLightValidation,
    main_bridge: u64
}

struct LocalState {
    initialized: bool,
    public_key: Box<PublicKey>,
    private_key: Box<SecretKey>,
    blocknum: u32,
    ecdh_private_key: Option<EcdhKey>,
    ecdh_public_key: Option<ring::agreement::PublicKey>,
}

#[derive(Encode, Decode)]
struct RuntimeInfo {
    machine_id: [u8; 16],
    pub_key: [u8; 33],
    score: u8,
}

fn se_to_b64<S>(value: &ChainLightValidation, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
    let data = value.encode();
    let s = base64::encode(data.as_slice());
    String::serialize(&s, serializer)
}

fn de_from_b64<'de, D>(deserializer: D) -> Result<ChainLightValidation, D::Error>
    where D: Deserializer<'de> {
    let s = String::deserialize(deserializer)?;
    let data = base64::decode(&s).map_err(de::Error::custom)?;
    ChainLightValidation::decode(&mut data.as_slice()).map_err(|_| de::Error::custom("bad data"))
}

lazy_static! {
    static ref STATE: SgxMutex<RuntimeState> = {
        SgxMutex::new(RuntimeState {
            contract1: contracts::data_plaza::DataPlaza::new(),
            contract2: contracts::balance::Balance::new(),
            contract3: contracts::assets::Assets::new(),
            light_client: ChainLightValidation::new(),
            main_bridge: 0
        })
    };

    static ref LOCAL_STATE: SgxMutex<LocalState> = {
        // Give it an uninitialized default. Will be reset when initialig pRuntime. x
        let raw_pk = hex::decode_hex("0000000000000000000000000000000000000000000000000000000000000001");
        let sk = SecretKey::parse_slice(raw_pk.as_slice()).unwrap();
        let pk = PublicKey::from_secret_key(&sk);

        SgxMutex::new(
            LocalState {
                initialized: false,
                public_key: Box::new(pk),
                private_key: Box::new(sk),
                blocknum: 0,
                ecdh_private_key: None,
                ecdh_public_key: None,
            }
        )
    };
}

pub const DEV_HOSTNAME:&'static str = "api.trustedservices.intel.com";
pub const SIGRL_SUFFIX:&'static str = "/sgx/dev/attestation/v4/sigrl/";
pub const REPORT_SUFFIX:&'static str = "/sgx/dev/attestation/v4/report";

fn parse_response_attn_report(resp : &[u8]) -> (String, String, String){
    println!("parse_response_attn_report");
    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut respp   = httparse::Response::new(&mut headers);
    let result = respp.parse(resp);
    println!("parse result {:?}", result);

    let msg : &'static str;

    match respp.code {
        Some(200) => msg = "OK Operation Successful",
        Some(401) => msg = "Unauthorized Failed to authenticate or authorize request.",
        Some(404) => msg = "Not Found GID does not refer to a valid EPID group ID.",
        Some(500) => msg = "Internal error occurred",
        Some(503) => msg = "Service is currently not able to process the request (due to
            a temporary overloading or maintenance). This is a
            temporary state – the same request can be repeated after
            some time. ",
        _ => {println!("DBG:{}", respp.code.unwrap()); msg = "Unknown error occured"},
    }

    println!("{}", msg);
    let mut len_num : u32 = 0;

    let mut sig = String::new();
    let mut cert = String::new();
    let mut attn_report = String::new();

    for i in 0..respp.headers.len() {
        let h = respp.headers[i];
        // println!("{} : {}", h.name, str::from_utf8(h.value).unwrap());
        match h.name{
            "Content-Length" => {
                let len_str = String::from_utf8(h.value.to_vec()).unwrap();
                len_num = len_str.parse::<u32>().unwrap();
                println!("content length = {}", len_num);
            }
            "X-IASReport-Signature" => sig = str::from_utf8(h.value).unwrap().to_string(),
            "X-IASReport-Signing-Certificate" => cert = str::from_utf8(h.value).unwrap().to_string(),
            _ => (),
        }
    }

    // Remove %0A from cert, and only obtain the signing cert
    cert = cert.replace("%0A", "");
    cert = cert::percent_decode(cert);
    let v: Vec<&str> = cert.split("-----").collect();
    let sig_cert = v[2].to_string();

    if len_num != 0 {
        let header_len = result.unwrap().unwrap();
        let resp_body = &resp[header_len..];
        attn_report = str::from_utf8(resp_body).unwrap().to_string();
        println!("Attestation report: {}", attn_report);
    }

    // len_num == 0
    (attn_report, sig, sig_cert)
}

fn parse_response_sigrl(resp : &[u8]) -> Vec<u8> {
    println!("parse_response_sigrl");
    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut respp   = httparse::Response::new(&mut headers);
    let result = respp.parse(resp);
    println!("parse result {:?}", result);
    println!("parse response{:?}", respp);

    let msg : &'static str;

    match respp.code {
        Some(200) => msg = "OK Operation Successful",
        Some(401) => msg = "Unauthorized Failed to authenticate or authorize request.",
        Some(404) => msg = "Not Found GID does not refer to a valid EPID group ID.",
        Some(500) => msg = "Internal error occurred",
        Some(503) => msg = "Service is currently not able to process the request (due to
            a temporary overloading or maintenance). This is a
            temporary state – the same request can be repeated after
            some time. ",
        _ => msg = "Unknown error occured",
    }

    println!("{}", msg);
    let mut len_num : u32 = 0;

    for i in 0..respp.headers.len() {
        let h = respp.headers[i];
        if h.name == "Content-Length" {
            let len_str = String::from_utf8(h.value.to_vec()).unwrap();
            len_num = len_str.parse::<u32>().unwrap();
            println!("content length = {}", len_num);
        }
    }

    if len_num != 0 {
        let header_len = result.unwrap().unwrap();
        let resp_body = &resp[header_len..];
        println!("Base64-encoded SigRL: {:?}", resp_body);

        return base64::decode(str::from_utf8(resp_body).unwrap()).unwrap();
    }

    // len_num == 0
    Vec::new()
}

pub fn make_ias_client_config() -> rustls::ClientConfig {
    let mut config = rustls::ClientConfig::new();

    config.root_store.add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);

    config
}

pub fn get_sigrl_from_intel(fd : c_int, gid : u32) -> Vec<u8> {
    println!("get_sigrl_from_intel fd = {:?}", fd);
    let config = make_ias_client_config();
    //let sigrl_arg = SigRLArg { group_id : gid };
    //let sigrl_req = sigrl_arg.to_httpreq();
    let ias_key = get_ias_api_key();

    let req = format!("GET {}{:08x} HTTP/1.1\r\nHOST: {}\r\nOcp-Apim-Subscription-Key: {}\r\nConnection: Close\r\n\r\n",
                      SIGRL_SUFFIX,
                      gid,
                      DEV_HOSTNAME,
                      ias_key);
    println!("{}", req);

    let dns_name = webpki::DNSNameRef::try_from_ascii_str(DEV_HOSTNAME).unwrap();
    let mut sess = rustls::ClientSession::new(&Arc::new(config), dns_name);
    let mut sock = TcpStream::new(fd).unwrap();
    let mut tls = rustls::Stream::new(&mut sess, &mut sock);

    let _result = tls.write(req.as_bytes());
    let mut plaintext = Vec::new();

    println!("write complete");

    match tls.read_to_end(&mut plaintext) {
        Ok(_) => (),
        Err(e) => {
            println!("get_sigrl_from_intel tls.read_to_end: {:?}", e);
            panic!("haha");
        }
    }
    println!("read_to_end complete");
    let resp_string = String::from_utf8(plaintext.clone()).unwrap();

    println!("{}", resp_string);

    parse_response_sigrl(&plaintext)
}

// TODO: support pse
pub fn get_report_from_intel(fd : c_int, quote : Vec<u8>) -> (String, String, String) {
    println!("get_report_from_intel fd = {:?}", fd);
    let config = make_ias_client_config();
    let encoded_quote = base64::encode(&quote[..]);
    let encoded_json = format!("{{\"isvEnclaveQuote\":\"{}\"}}\r\n", encoded_quote);

    let ias_key = get_ias_api_key();

    let req = format!("POST {} HTTP/1.1\r\nHOST: {}\r\nOcp-Apim-Subscription-Key:{}\r\nContent-Length:{}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{}",
                      REPORT_SUFFIX,
                      DEV_HOSTNAME,
                      ias_key,
                      encoded_json.len(),
                      encoded_json);
    println!("{}", req);
    let dns_name = webpki::DNSNameRef::try_from_ascii_str(DEV_HOSTNAME).unwrap();
    let mut sess = rustls::ClientSession::new(&Arc::new(config), dns_name);
    let mut sock = TcpStream::new(fd).unwrap();
    let mut tls = rustls::Stream::new(&mut sess, &mut sock);

    let _result = tls.write(req.as_bytes());
    let mut plaintext = Vec::new();

    println!("write complete");

    tls.read_to_end(&mut plaintext).unwrap();
    println!("read_to_end complete");
    let resp_string = String::from_utf8(plaintext.clone()).unwrap();

    println!("resp_string = {}", resp_string);

    let (attn_report, sig, cert) = parse_response_attn_report(&plaintext);

    (attn_report, sig, cert)
}

fn as_u32_le(array: &[u8; 4]) -> u32 {
    ((array[0] as u32) <<  0) +
        ((array[1] as u32) <<  8) +
        ((array[2] as u32) << 16) +
        ((array[3] as u32) << 24)
}

fn load_spid(filename: &str) -> sgx_spid_t {
    let mut spidfile = fs::File::open(filename).expect("cannot open spid file");
    let mut contents = String::new();
    spidfile.read_to_string(&mut contents).expect("cannot read the spid file");

    hex::decode_spid(&contents)
}

fn get_ias_api_key() -> String {
    let mut keyfile = fs::File::open("key.txt").expect("cannot open ias key file");
    let mut key = String::new();
    keyfile.read_to_string(&mut key).expect("cannot read the ias key file");

    key.trim_end().to_owned()
}

#[allow(const_err)]
pub fn create_attestation_report(data: &[u8], sign_type: sgx_quote_sign_type_t) -> Result<(String, String, String), sgx_status_t> {
    let data_len = data.len();
    if data_len > SGX_REPORT_DATA_SIZE {
        panic!("data length over 64 bytes");
    }

    // Workflow:
    // (1) ocall to get the target_info structure (ti) and epid group id (eg)
    // (1.5) get sigrl
    // (2) call sgx_create_report with ti+data, produce an sgx_report_t
    // (3) ocall to sgx_get_quote to generate (*mut sgx-quote_t, uint32_t)

    // (1) get ti + eg
    let mut ti : sgx_target_info_t = sgx_target_info_t::default();
    let mut eg : sgx_epid_group_id_t = sgx_epid_group_id_t::default();
    let mut rt : sgx_status_t = sgx_status_t::SGX_ERROR_UNEXPECTED;

    let res = unsafe {
        ocall_sgx_init_quote(&mut rt as *mut sgx_status_t,
                             &mut ti as *mut sgx_target_info_t,
                             &mut eg as *mut sgx_epid_group_id_t)
    };

    println!("eg = {:?}", eg);

    if res != sgx_status_t::SGX_SUCCESS {
        return Err(res);
    }

    if rt != sgx_status_t::SGX_SUCCESS {
        return Err(rt);
    }

    let eg_num = as_u32_le(&eg);

    // (1.5) get sigrl
    let mut ias_sock : i32 = 0;

    let res = unsafe {
        ocall_get_ias_socket(&mut rt as *mut sgx_status_t,
                             &mut ias_sock as *mut i32)
    };

    if res != sgx_status_t::SGX_SUCCESS {
        return Err(res);
    }

    if rt != sgx_status_t::SGX_SUCCESS {
        return Err(rt);
    }

    //println!("Got ias_sock = {}", ias_sock);

    // Now sigrl_vec is the revocation list, a vec<u8>
    let sigrl_vec : Vec<u8> = get_sigrl_from_intel(ias_sock, eg_num);

    // (2) Generate the report
    // Fill data into report_data
    let mut report_data: sgx_report_data_t = sgx_report_data_t::default();
    report_data.d[..data_len].clone_from_slice(data);

    let rep = match rsgx_create_report(&ti, &report_data) {
        Ok(r) =>{
            println!("Report creation => success {:?}", r.body.mr_signer.m);
            Some(r)
        },
        Err(e) =>{
            println!("Report creation => failed {:?}", e);
            None
        },
    };

    let mut quote_nonce = sgx_quote_nonce_t { rand : [0;16] };
    let mut os_rng = os::SgxRng::new().unwrap();
    os_rng.fill_bytes(&mut quote_nonce.rand);
    println!("rand finished");
    let mut qe_report = sgx_report_t::default();
    const RET_QUOTE_BUF_LEN : u32 = 2048;
    let mut return_quote_buf : [u8; RET_QUOTE_BUF_LEN as usize] = [0;RET_QUOTE_BUF_LEN as usize];
    let mut quote_len : u32 = 0;

    // (3) Generate the quote
    // Args:
    //       1. sigrl: ptr + len
    //       2. report: ptr 432bytes
    //       3. linkable: u32, unlinkable=0, linkable=1
    //       4. spid: sgx_spid_t ptr 16bytes
    //       5. sgx_quote_nonce_t ptr 16bytes
    //       6. p_sig_rl + sigrl size ( same to sigrl)
    //       7. [out]p_qe_report need further check
    //       8. [out]p_quote
    //       9. quote_size
    let (p_sigrl, sigrl_len) =
        if sigrl_vec.len() == 0 {
            (ptr::null(), 0)
        } else {
            (sigrl_vec.as_ptr(), sigrl_vec.len() as u32)
        };
    let p_report = (&rep.unwrap()) as * const sgx_report_t;
    let quote_type = sign_type;

    let spid : sgx_spid_t = load_spid("spid.txt");

    let p_spid = &spid as *const sgx_spid_t;
    let p_nonce = &quote_nonce as * const sgx_quote_nonce_t;
    let p_qe_report = &mut qe_report as *mut sgx_report_t;
    let p_quote = return_quote_buf.as_mut_ptr();
    let maxlen = RET_QUOTE_BUF_LEN;
    let p_quote_len = &mut quote_len as *mut u32;

    let result = unsafe {
        ocall_get_quote(&mut rt as *mut sgx_status_t,
                        p_sigrl,
                        sigrl_len,
                        p_report,
                        quote_type,
                        p_spid,
                        p_nonce,
                        p_qe_report,
                        p_quote,
                        maxlen,
                        p_quote_len)
    };

    if result != sgx_status_t::SGX_SUCCESS {
        return Err(result);
    }

    if rt != sgx_status_t::SGX_SUCCESS {
        println!("ocall_get_quote returned {}", rt);
        return Err(rt);
    }

    // Added 09-28-2018
    // Perform a check on qe_report to verify if the qe_report is valid
    match rsgx_verify_report(&qe_report) {
        Ok(()) => println!("rsgx_verify_report passed!"),
        Err(x) => {
            println!("rsgx_verify_report failed with {:?}", x);
            return Err(x);
        },
    }

    // Check if the qe_report is produced on the same platform
    if ti.mr_enclave.m != qe_report.body.mr_enclave.m ||
        ti.attributes.flags != qe_report.body.attributes.flags ||
        ti.attributes.xfrm  != qe_report.body.attributes.xfrm {
        println!("qe_report does not match current target_info!");
        return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
    }

    println!("qe_report check passed");

    // Debug
    // for i in 0..quote_len {
    //     print!("{:02X}", unsafe {*p_quote.offset(i as isize)});
    // }
    // println!("");

    // Check qe_report to defend against replay attack
    // The purpose of p_qe_report is for the ISV enclave to confirm the QUOTE
    // it received is not modified by the untrusted SW stack, and not a replay.
    // The implementation in QE is to generate a REPORT targeting the ISV
    // enclave (target info from p_report) , with the lower 32Bytes in
    // report.data = SHA256(p_nonce||p_quote). The ISV enclave can verify the
    // p_qe_report and report.data to confirm the QUOTE has not be modified and
    // is not a replay. It is optional.

    let mut rhs_vec : Vec<u8> = quote_nonce.rand.to_vec();
    rhs_vec.extend(&return_quote_buf[..quote_len as usize]);
    let rhs_hash = rsgx_sha256_slice(&rhs_vec[..]).unwrap();
    let lhs_hash = &qe_report.body.report_data.d[..32];

    println!("rhs hash = {:02X}", rhs_hash.iter().format(""));
    println!("report hs= {:02X}", lhs_hash.iter().format(""));

    if rhs_hash != lhs_hash {
        println!("Quote is tampered!");
        return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
    }

    let quote_vec : Vec<u8> = return_quote_buf[..quote_len as usize].to_vec();
    let res = unsafe {
        ocall_get_ias_socket(&mut rt as *mut sgx_status_t,
                             &mut ias_sock as *mut i32)
    };

    if res != sgx_status_t::SGX_SUCCESS {
        return Err(res);
    }

    if rt != sgx_status_t::SGX_SUCCESS {
        return Err(rt);
    }

    let (attn_report, sig, cert) = get_report_from_intel(ias_sock, quote_vec);
    Ok((attn_report, sig, cert))
}

fn generate_seal_key() -> [u8; 16] {
    let report = rsgx_self_report();
    let key_id = sgx_key_id_t::default();
    let key_request = sgx_key_request_t {
        key_name: SGX_KEYSELECT_SEAL,
        key_policy: SGX_KEYPOLICY_MRSIGNER,
        isv_svn: report.body.isv_svn,
        reserved1: 0_u16,
        cpu_svn: report.body.cpu_svn,
        attribute_mask: sgx_attributes_t{flags: TSEAL_DEFAULT_FLAGSMASK, xfrm: 0},
        key_id,
        misc_mask: TSEAL_DEFAULT_MISCMASK,
        config_svn: report.body.config_svn,
        reserved2: [0_u8; SGX_KEY_REQUEST_RESERVED2_BYTES],
    };
    let seal_key = rsgx_get_align_key(&key_request).unwrap();

    seal_key.key
}

const ACTION_TEST: u8 = 0;
const ACTION_INIT_RUNTIME: u8 = 1;
const ACTION_GET_INFO: u8 = 2;
const ACTION_DUMP_STATES: u8 = 3;
const ACTION_LOAD_STATES: u8 = 4;
const ACTION_SYNC_BLOCK: u8 = 5;
const ACTION_QUERY: u8 = 6;
const ACTION_SET: u8 = 21;
const ACTION_GET: u8 = 22;

#[derive(Serialize, Deserialize, Debug)]
struct InitRuntimeReq {
    skip_ra: bool,
    bridge_genesis_info_b64: String
}
#[derive(Serialize, Deserialize, Debug)]
pub struct InitRuntimeResp {
  pub encoded_runtime_info: Vec<u8>,
  pub public_key: String,
  pub ecdh_public_key: String,
  pub attestation: Option<InitRespAttestation>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct InitRespAttestation {
  pub version: i32,
  pub provider: String,
  pub payload: AttestationReport,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct AttestationReport {
  pub report: String,
  pub signature: String,
  pub signing_cert: String,
}
#[derive(Serialize, Deserialize, Debug)]
struct TestReq {
    test_parse_block: Option<bool>,
    test_bridge: Option<bool>,
    test_ecdh: Option<TestEcdhParam>,
}
#[derive(Serialize, Deserialize, Debug)]
struct SyncBlockReq {
    blocks_b64: Vec<String>,
    authority_set_change_b64: Option<String>,
}
#[derive(Serialize, Deserialize, Debug)]
struct TestEcdhParam {
    pubkey_hex: Option<String>,
    message_b64: Option<String>,
}

#[derive(Encode, Decode, Debug, Clone)]
struct BlockWithEvents {
    block: chain::SignedBlock,
    events: Option<Vec<u8>>,
    proof: Option<Vec<Vec<u8>>>,
    key: Option<Vec<u8>>,
}

#[no_mangle]
pub extern "C" fn ecall_set_state(
    input_ptr: *const u8, input_len: usize
) -> sgx_status_t {
    let input_slice = unsafe { std::slice::from_raw_parts(input_ptr, input_len) };
    let input_value: serde_json::value::Value = serde_json::from_slice(input_slice).unwrap();
    let _input = input_value.as_object().unwrap();

    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub extern "C" fn ecall_handle(
    action: u8,
    input_ptr: *const u8, input_len: usize,
    output_ptr : *mut u8, output_len_ptr: *mut usize, output_buf_len: usize
) -> sgx_status_t {
    let input_slice = unsafe { std::slice::from_raw_parts(input_ptr, input_len) };
    let input: serde_json::value::Value = serde_json::from_slice(input_slice).unwrap();
    let input_value = input.get("input").unwrap().clone();
    // Strong typed
    fn load_param<T: de::DeserializeOwned>(input_value: serde_json::value::Value) -> T {
        serde_json::from_value(input_value).unwrap()
    }
    let result = match action {
        ACTION_INIT_RUNTIME => init_runtime(load_param(input_value)),
        ACTION_TEST =>  test(load_param(input_value)),
        ACTION_QUERY => query(load_param(input_value)),
        ACTION_SYNC_BLOCK => sync_block(load_param(input_value)),
        _ => {
            let payload = input_value.as_object().unwrap();
            match action {
                ACTION_GET_INFO => get_info(payload),
                ACTION_DUMP_STATES => dump_states(payload),
                ACTION_LOAD_STATES => load_states(payload),
                ACTION_GET => get(payload),
                ACTION_SET => set(payload),
                _ => unknown()
            }
        }
    };

    let local_state = LOCAL_STATE.lock().unwrap();

    let output_json = match result {
        Ok(payload) => {
            let s_payload = payload.to_string();

            let hash_payload = rsgx_sha256_slice(&s_payload.as_bytes()).unwrap();
            let message = secp256k1::Message::parse_slice(&hash_payload[..32]).unwrap();
            let (signature, _recovery_id) = secp256k1::sign(&message, &local_state.private_key);

            json!({
                "status": "ok",
                "payload": s_payload,
                "signature": hex::encode_hex_compact(signature.serialize().as_ref()),
            })
        },
        Err(payload) => {
            let s_payload = payload.to_string();

            let hash_payload = rsgx_sha256_slice(&s_payload.as_bytes()).unwrap();
            let message = secp256k1::Message::parse_slice(&hash_payload[..32]).unwrap();
            let (signature, _recovery_id) = secp256k1::sign(&message, &local_state.private_key);

            json!({
                "status": "error",
                "payload": s_payload,
                "signature": hex::encode_hex_compact(signature.serialize().as_ref()),
            })
        }
    };
    println!("{}", output_json.to_string());

    let output_json_vec = serde_json::to_vec(&output_json).unwrap();
    let output_json_vec_len = output_json_vec.len();
    let output_json_vec_len_ptr = &output_json_vec_len as *const usize;

    unsafe {
        if output_json_vec_len <= output_buf_len {
            ptr::copy_nonoverlapping(output_json_vec.as_ptr(),
                                     output_ptr,
                                     output_json_vec_len);
        } else {
            println!("Too much output. Buffer overflow.");
        }
        ptr::copy_nonoverlapping(output_json_vec_len_ptr,
                                 output_len_ptr,
                                 std::mem::size_of_val(&output_json_vec_len));
    }

    sgx_status_t::SGX_SUCCESS
}


// --------------------------------

fn error_msg(msg: &str) -> Value {
    json!({ "message": msg })
}

fn unknown() -> Result<Value, Value> {
    Err(json!({
        "message": "Unknown action"
    }))
}

fn test(param: TestReq) -> Result<Value, Value> {
    if param.test_parse_block == Some(true) {
        test_parse_block();
    }
    if param.test_bridge == Some(true) {
        test_bridge();
    }
    if let Some(p) = param.test_ecdh {
        test_ecdh(p);
    }
    Ok(json!({}))
}

const SECRET: &[u8; 32] = b"24e3e78e1f15150cdbad02f3205f6dd0";

fn dump_states(_input: &Map<String, Value>) -> Result<Value, Value> {
    let sessions = STATE.lock().unwrap();
    let serialized = serde_json::to_string(&*sessions).unwrap();

    // Your private data
    let content = serialized.as_bytes().to_vec();
    println!("Content to encrypt's size {}", content.len());
    println!("{}", serialized);

    // Ring uses the same input variable as output
    let mut in_out = content.clone();
    println!("in_out len {}", in_out.len());

    // Random data must be used only once per encryption
    let iv = aead::generate_iv();
    aead::encrypt(&iv, SECRET, &mut in_out);

    Ok(json!({
        "data": hex::encode_hex_compact(in_out.as_ref()),
        "nonce": hex::encode_hex_compact(&iv)
    }))
}

fn load_states(input: &Map<String, Value>) -> Result<Value, Value> {
    let nonce_vec = hex::decode_hex(input.get("nonce").unwrap().as_str().unwrap());
    let mut in_out = hex::decode_hex(input.get("data").unwrap().as_str().unwrap());
    println!("{}", input.get("data").unwrap().as_str().unwrap());

    let decrypted_data = aead::decrypt(&nonce_vec, &*SECRET, &mut in_out);
    println!("{}", String::from_utf8(decrypted_data.to_vec()).unwrap());

    let deserialized: RuntimeState = serde_json::from_slice(decrypted_data).unwrap();

    println!("{}", serde_json::to_string_pretty(&deserialized).unwrap());

    let mut sessions = STATE.lock().unwrap();
    std::mem::replace(&mut *sessions, deserialized);

    Ok(json!({}))
}

fn init_runtime(input: InitRuntimeReq) -> Result<Value, Value> {
    // TODO: Guard only initialize once
    if LOCAL_STATE.lock().unwrap().initialized {
        return Err(json!({"message": "Already initialized"}))
    }

    // Generate Seal Key as Machine Id
    let machine_id = generate_seal_key();
    println!("Machine id: {:?}", &machine_id);

    // Generate identity
    let mut prng = rand::rngs::OsRng::default();
    let sk = SecretKey::random(&mut prng);
    let pk = PublicKey::from_secret_key(&sk);
    let serialized_pk = pk.serialize_compressed();
    let s_pk = hex::encode_hex_compact(serialized_pk.as_ref());

    // ECDH identity
    let raw_key = hex::decode_hex("e71f37bc68ad82ec70ddf75ee787bccbb1b70e7293f710faee042f97db8a3ba3");
    let ecdh_sk = ecdh::create_key(raw_key.as_slice()).expect("can't create ecdh key");
    let ecdh_pk = ecdh_sk.compute_public_key().expect("can't compute pubkey");
    let s_ecdh_pk = hex::encode_hex_compact(ecdh_pk.as_ref());
    println!("ECDH pubkey: {:?}", ecdh_pk);

    // Save identity
    let mut local_state = LOCAL_STATE.lock().unwrap();
    (*local_state).initialized = true;
    *local_state.public_key = pk.clone();
    *local_state.private_key = sk.clone();
    local_state.ecdh_private_key = Some(ecdh_sk);
    local_state.ecdh_public_key = Some(ecdh_pk);

    // Build RuntimeInfo
    let runtime_info = RuntimeInfo {
        machine_id,
        pub_key: serialized_pk,
        score: 10
    };
    let encoded_runtime_info = runtime_info.encode();
    let runtime_info_hash = sp_core::hashing::blake2_512(&encoded_runtime_info);

    println!("Encoded runtime info");
    println!("{:?}", encoded_runtime_info);

    // Produce remote attestation report
    let mut attestation: Option<InitRespAttestation> = None;
    if !input.skip_ra {
        let (attn_report, sig, cert) = match create_attestation_report(&runtime_info_hash, sgx_quote_sign_type_t::SGX_LINKABLE_SIGNATURE) {
            Ok(r) => r,
            Err(e) => {
                println!("Error in create_attestation_report: {:?}", e);
                return Err(json!({"message": "Error while connecting to IAS"}))
            }
        };

        attestation = Some(InitRespAttestation {
            version: 1,
            provider: "SGX".to_string(),
            payload: AttestationReport {
                report: attn_report,
                signature: sig,
                signing_cert: cert,
            }
        });
    }

    // Initialize bridge
    let raw_genesis = base64::decode(&input.bridge_genesis_info_b64)
        .expect("Bad bridge_genesis_info_b64");
    let genesis = light_validation::BridgeInitInfo::<chain::Runtime>
    ::decode(&mut raw_genesis.as_slice())
        .expect("Can't decode bridge_genesis_info_b64");

    let mut state = STATE.lock().unwrap();
    let bridge_id = state.light_client.initialize_bridge(
        genesis.block_header,
        genesis.validator_set,
        genesis.validator_set_proof)
        .expect("Bridge initialize failed");
    state.main_bridge = bridge_id;
    local_state.blocknum = 1;

    let resp = InitRuntimeResp {
        encoded_runtime_info,
        public_key: s_pk,
        ecdh_public_key: s_ecdh_pk,
        attestation
    };
    Ok(serde_json::to_value(resp).unwrap())
}

/*
bf192ec197592fda420383b1db2676e5b409a58cad489cc5c35dd94390c65a584c10987af003bd853a29992021a2f1a617b798ad3a151a3e2cbd6e4029370b7c68ec05dc5c1a11c17373b8d18d498fc1da0e94c35bdb3adc8116efda35b6025f3a080661757261206962a60f00000000056175726101013e3e4b72e1a133d129799aeaf43884493b6c30530f04be95f61089d73f21de2709108c407b8cb891ac8dbd1a498f1b6792f54e6f3b4f499d2e7010c81a902d8404280401000bf07ca2cb6e0101d90456000000000000007b968b3f7eca2df8cf39ee8c9538f02e1361f44d0b809450f8501676db28131a13000000087b968b3f7eca2df8cf39ee8c9538f02e1361f44d0b809450f8501676db28131a130000008dff9b2ca754cb5e80036647aab3fd25ccc4e4236c9fecd61968f121141149a7c402ebd3cdd43779a7e8d9afffee1989197d343ed9e936721168305a6da09d0188dc3417d5058ec4b4503e0c12ea1a0a89be200fe98922423d4334014fa6b0ee7b968b3f7eca2df8cf39ee8c9538f02e1361f44d0b809450f8501676db28131a1300000090a870eb3be217aa99476919b2864539830a622e82a9355d0f91f28a777372ecb0c3a1164f68246cfa5577682ab7505aff139681953fe0cc7457300a53807303d17c2d7823ebf260fd138f2d7e27d114c0145d968b5ff5006125f2414fadae6900
SignedBlock {
    block: Block {
        header: Header {
            parent_hash: 0xbf192ec197592fda420383b1db2676e5b409a58cad489cc5c35dd94390c65a58,
            number: 19,
            state_root: 0x10987af003bd853a29992021a2f1a617b798ad3a151a3e2cbd6e4029370b7c68,
            extrinsics_root: 0xec05dc5c1a11c17373b8d18d498fc1da0e94c35bdb3adc8116efda35b6025f3a,
            digest: Digest {
                logs: [
                    DigestItem::PreRuntime([97, 117, 114, 97], [105, 98, 166, 15, 0, 0, 0, 0]),
                    DigestItem::Seal([97, 117, 114, 97], [62, 62, 75, 114, 225, 161, 51, 209, 41, 121, 154, 234, 244, 56, 132, 73, 59, 108, 48, 83, 15, 4, 190, 149, 246, 16, 137, 215, 63, 33, 222, 39, 9, 16, 140, 64, 123, 140, 184, 145, 172, 141, 189, 26, 73, 143, 27, 103, 146, 245, 78, 111, 59, 79, 73, 157, 46, 112, 16, 200, 26, 144, 45, 132])
                ]
            }
        },
        extrinsics: [
            UncheckedExtrinsic(None, Call::Timestamp(set(1575374454000,)))
        ]
    },
    justification: Some([86, 0, 0, 0, 0, 0, 0, 0, 123, 150, 139, 63, 126, 202, 45, 248, 207, 57, 238, 140, 149, 56, 240, 46, 19, 97, 244, 77, 11, 128, 148, 80, 248, 80, 22, 118, 219, 40, 19, 26, 19, 0, 0, 0, 8, 123, 150, 139, 63, 126, 202, 45, 248, 207, 57, 238, 140, 149, 56, 240, 46, 19, 97, 244, 77, 11, 128, 148, 80, 248, 80, 22, 118, 219, 40, 19, 26, 19, 0, 0, 0, 141, 255, 155, 44, 167, 84, 203, 94, 128, 3, 102, 71, 170, 179, 253, 37, 204, 196, 228, 35, 108, 159, 236, 214, 25, 104, 241, 33, 20, 17, 73, 167, 196, 2, 235, 211, 205, 212, 55, 121, 167, 232, 217, 175, 255, 238, 25, 137, 25, 125, 52, 62, 217, 233, 54, 114, 17, 104, 48, 90, 109, 160, 157, 1, 136, 220, 52, 23, 213, 5, 142, 196, 180, 80, 62, 12, 18, 234, 26, 10, 137, 190, 32, 15, 233, 137, 34, 66, 61, 67, 52, 1, 79, 166, 176, 238, 123, 150, 139, 63, 126, 202, 45, 248, 207, 57, 238, 140, 149, 56, 240, 46, 19, 97, 244, 77, 11, 128, 148, 80, 248, 80, 22, 118, 219, 40, 19, 26, 19, 0, 0, 0, 144, 168, 112, 235, 59, 226, 23, 170, 153, 71, 105, 25, 178, 134, 69, 57, 131, 10, 98, 46, 130, 169, 53, 93, 15, 145, 242, 138, 119, 115, 114, 236, 176, 195, 161, 22, 79, 104, 36, 108, 250, 85, 119, 104, 42, 183, 80, 90, 255, 19, 150, 129, 149, 63, 224, 204, 116, 87, 48, 10, 83, 128, 115, 3, 209, 124, 45, 120, 35, 235, 242, 96, 253, 19, 143, 45, 126, 39, 209, 20, 192, 20, 93, 150, 139, 95, 245, 0, 97, 37, 242, 65, 79, 173, 174, 105, 0])
}
*/

mod contracts;
mod types;

use types::TxRef;
use contracts::{Contract, ContractId, DATA_PLAZA, BALANCE, ASSETS, SYSTEM};

fn fmt_call(call: &chain::Call) -> String {
    match call {
        chain::Call::Timestamp(chain::TimestampCall::set(t)) =>
            format!("Timestamp::set({})", t),
        chain::Call::Balances(chain::BalancesCall::transfer(to, amount)) =>
            format!("Balance::transfer({:?}, {:?})", to, amount),
        _ => String::from("<Unparsed>")
    }
}

fn print_block(signed_block: &chain::SignedBlock) {
    let header: &chain::Header = &signed_block.block.header;
    let extrinsics: &Vec<chain::UncheckedExtrinsic> = &signed_block.block.extrinsics;

    println!("SignedBlock {{");
    println!("  block {{");
    println!("    header {{");
    println!("      number: {}", header.number);
    println!("      extrinsics_root: {}", header.extrinsics_root);
    println!("      state_root: {}", header.state_root);
    println!("      parent_hash: {}", header.parent_hash);
    println!("      digest: logs[{}]", header.digest.logs.len());
    println!("  extrinsics: [");
    for extrinsic in extrinsics {
        println!("    UncheckedExtrinsic {{");
        println!("      function: {}", fmt_call(&extrinsic.function));
        println!("      signature: {:?}", extrinsic.signature);
        println!("    }}");
    }
    println!("  ]");
    println!("  justification: <skipped...>");
    println!("}}");
}

fn parse_block(data: &Vec<u8>) -> Result<chain::SignedBlock, parity_scale_codec::Error> {
    chain::SignedBlock::decode(&mut data.as_slice())
}

fn format_address(addr: &chain::Address) -> String {
    match addr {
        chain::Address::Id(id) => hex::encode_hex_compact(id.as_ref()),
        chain::Address::Index(index) => format!("index:{}", index)
    }
}

fn test_parse_block() {
    let raw_block: Vec<u8> = base64::decode("iAKMDRPbdbAZ0eev9OZ1QgaAkoEnazAp6JzH2GeRFYdsR+pFUBbOaAW0+k5K+jPtsEr/P/JKJQDSobnB98Qhf8ug8HkDygkapC5T++CNvzYORIFimatwYSu/U53t66xzpQgGYXVyYSCGvagPAAAAAAVhdXJhAQEuXZ5zy2+qk+60y+/m1r0oZv/+LEiDCxMotfkvjP9aebuUVxBTmd2LCpu645AAjpRUNhqOmVuiKreUoV1aMpWLCCgEAQALoPTZAm8BQQKE/9Q1k8cV/dMcYRQavQSpn9aCLIVYhUzN45pWhOelbaJ9AU5gayhZiGwAEAthrYW6Ucm+acGAR3whdfUk17jp4NMearo4+NxR2w0VsVkEF0gQ/U6AHggnM+BZmvrhhMdSygqlAQAABAD/jq8EFRaHc2Mmyf6hfiX8UodhNpPJEpCcsiaqR5TyakgHABCl1OgA")
        .unwrap();
    println!("SignedBlock data[{}]", raw_block.len());
    let block = match parse_block(&raw_block) {
        Ok(b) => b,
        Err(err) => {
            println!("test_parse_block: Failed to parse block ({:?})", err);
            return;
        }
    };
    print_block(&block);

    // test parse address
    let ref_sig = block.block.extrinsics[1].signature.as_ref().unwrap();
    let ref_addr = &ref_sig.0;
    println!("test_parse_block: addr = {}", format_address(ref_addr));

    let cmd_json = serde_json::to_string_pretty(
        &contracts::data_plaza::Command::List(contracts::data_plaza::ItemDetails {
            name: "nname".to_owned(),
            category: "ccategory".to_owned(),
            description: "ddesc".to_owned(),
            price: contracts::data_plaza::PricePolicy::PerRow { price: 100_u128 },
            dataset_link: "llink".to_owned(),
            dataset_preview: "pprev".to_owned()
        })).expect("jah");
    println!("sample command: {}", cmd_json);
}

fn test_bridge() {
    // 1. load genesis
    let raw_genesis = base64::decode("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAh9rd6Uku4dTja+JQVMLsOZ5GtS4nU0cdpuvgchlapeMDFwoudZe3t+PYTAU5HROaYrFX54eG2MCC8p3PTBETFAAIiNw0F9UFjsS0UD4MEuoaCom+IA/piSJCPUM0AU+msO4BAAAAAAAAANF8LXgj6/Jg/ROPLX4n0RTAFF2Wi1/1AGEl8kFPra5pAQAAAAAAAAAMoQKALhCA33I0FGiDoLZ6HBWl1uCIt+sLgUbPlfMJqUk/gukhEt6AviHkl5KFGndUmA+ClBT2kPSvmBOvZTWowWjNYfynHU6AOFjSvKwU3s/vvRg7QOrJeehLgo9nGfN91yHXkHcWLkuAUJegqkIzp2A6LPkZouRRsKgiY4Wu92V8JXrn3aSXrw2AXDYZ0c8CICTMvasQ+rEpErmfEmg+BzH19s/zJX4LP8adAWRyYW5kcGFfYXV0aG9yaXRpZXNJAQEIiNw0F9UFjsS0UD4MEuoaCom+IA/piSJCPUM0AU+msO4BAAAAAAAAANF8LXgj6/Jg/ROPLX4n0RTAFF2Wi1/1AGEl8kFPra5pAQAAAAAAAABtAYKmqACASqIhjIQMli+MpltqIZlc2FVhXCd/m9F6k9Q5u13xU3JQXHh0cmluc2ljX2luZGV4EAAAAACAc0yvcsUiYcma5kSPZKxrMxbyDufisOfMmIsX1bDxfHc=")
        .expect("Bad bridge_genesis_innfo_b64");
    let genesis = light_validation::BridgeInitInfo::<chain::Runtime>
    ::decode(&mut raw_genesis.as_slice())
        .expect("Can't decode bridge_genesis_info_b64");

    println!("bridge_genesis_info_b64: {:?}", genesis);

    let mut state = STATE.lock().unwrap();
    let id = state.light_client.initialize_bridge(
        genesis.block_header,
        genesis.validator_set,
        genesis.validator_set_proof)
        .expect("Init bridge failed; qed");

    // 2. import a few blocks
    let raw_header = base64::decode("/YNUiuLexTFhCwS9smQZzq1EggRC0DWkgGuCbyaKKSQEhrlzSqBeoaYA0f7EXN/Z0WJINIurZbvBQU/2dKyaFxA8RCHwO2aMQ+Agbl7pMtC9Yn6AH0rYW30BFRZmva2k5QgGYXVyYSAQWrUPAAAAAAVhdXJhAQFsP5YkXJ1qPXxseyMUtX5QXTQZBbIKDqYeZq1mw1f6MyROgQ3BIJpp8wCgSTlPttAQmkw4Ol4b5tJ5VaBzUd2B").unwrap();
    let header = chain::Header::decode(&mut raw_header.as_slice()).expect("Bad block1 header; qed");
    let justification = base64::decode("CgAAAAAAAADew4hPceq4QYh0sxLxlaq0lTl+SWKw88vuBatKPewDcwEAAAAI3sOIT3HquEGIdLMS8ZWqtJU5fklisPPL7gWrSj3sA3MBAAAAXWeJEfa3FLKCvN8SYsx3wBx3N78oHP4THt65DyExstiuwZpF62Ci18/8hdr4cf+jbdYkSBBeMJuL9dTUY/QzAojcNBfVBY7EtFA+DBLqGgqJviAP6YkiQj1DNAFPprDu3sOIT3HquEGIdLMS8ZWqtJU5fklisPPL7gWrSj3sA3MBAAAA8TA1VpLNnlBnetJ74i0IY/Bv6InpDTkG2q4LCy0qVPG3WQhgadGFMInCyc38vOHKKwA7X2r7FGfQmuuPlwRCA9F8LXgj6/Jg/ROPLX4n0RTAFF2Wi1/1AGEl8kFPra5pAA==").unwrap();

    state.light_client.submit_simple_header(id, header, justification)
        .expect("Submit first block failed; qed");
}

fn test_ecdh(params: TestEcdhParam) {
    let bob_pub: [u8; 65] = [0x04, 0xb8, 0xd1, 0x8e, 0x7d, 0xe4, 0xc1, 0x10, 0x69, 0x48, 0x7b, 0x5c, 0x1e, 0x6e, 0xa5, 0xdf, 0x04, 0x51, 0xf7, 0xe1, 0xa8, 0x46, 0x17, 0x5b, 0xf6, 0xfd, 0xf8, 0xe8, 0xea, 0x5c, 0x68, 0xcd, 0xfb, 0xca, 0x0e, 0x1f, 0x17, 0x1c, 0x0b, 0xee, 0x3d, 0x34, 0x71, 0x11, 0x07, 0x67, 0x2d, 0x6a, 0x13, 0x57, 0x26, 0x7d, 0x5a, 0xcb, 0x3b, 0x98, 0x4c, 0xa5, 0xbf, 0xf4, 0xbf, 0x33, 0x78, 0x32, 0x96];

    let pubkey_data = params.pubkey_hex.map(|h| hex::decode_hex(&h));
    let pk = match pubkey_data.as_ref() {
        Some(d) => d.as_slice(),
        None => bob_pub.as_ref()
    };

    let local_state = LOCAL_STATE.lock().unwrap();
    let alice_priv = &local_state.ecdh_private_key.as_ref()
        .expect("ECDH private key not initialized");
    let key = ecdh::agree(alice_priv, pk);
    println!("ECDH derived secret key: {}", hex::encode_hex_compact(&key));

    if let Some(msg_b64) = params.message_b64 {
        let mut msg = base64::decode(&msg_b64)
            .expect("Failed to decode msg_b64");
        let iv = aead::generate_iv();
        aead::encrypt(&iv, &key, &mut msg);

        println!("AES-GCM: {}", hex::encode_hex_compact(&msg));
        println!("IV: {}", hex::encode_hex_compact(&iv));
    }
}

fn handle_execution(state: &mut RuntimeState, pos: &TxRef,
                    origin: Option<(chain::Address, chain::Signature, chain::SignedExtra)>,
                    contract_id: ContractId, payload: &Vec<u8>,
                    ecdh_privkey: &EcdhKey) {
    let payload: types::Payload = serde_json::from_slice(payload.as_slice())
        .expect("Failed to decode payload");
    let inner_data = match payload {
        types::Payload::Plain(data) => data.into_bytes(),
        types::Payload::Cipher(cipher) => {
            cryptography::decrypt(&cipher, ecdh_privkey).expect("Decrypt failed").msg
        }
    };

    let origin = if let Some((chain::Address::Id(account_id), _, _)) = origin {
        account_id
    } else {
        panic!("No account id found for tx {:?}", pos);
    };

    let inner_data_string = String::from_utf8_lossy(&inner_data);
    println!("handle_execution: incominng cmd: {}", inner_data_string);

    println!("handle_execution: about to call handle_command");
    let status = match contract_id {
        DATA_PLAZA => state.contract1.handle_command(
            &origin, pos,
            serde_json::from_slice(inner_data.as_slice()).expect("Failed to deser contract1 cmd")
        ),
        BALANCE => state.contract2.handle_command(
            &origin, pos,
            serde_json::from_slice(inner_data.as_slice()).expect("Failed to deser contract2 cmd")
        ),
        ASSETS => state.contract3.handle_command(
            &origin, pos,
            serde_json::from_slice(inner_data.as_slice()).expect("Failed to deser contract3 cmd")
        ),
        _ => {
            println!("handle_execution: Skipped unknown contract: {}", contract_id);
            TransactionStatus::BadContractId
        },
    };

    let mut gr = GLOBAL_RECEIPT.lock().unwrap();
    gr.add_receipt(pos.tx_hash.clone(), TransactionReceipt {
        account: AccountIdWrapper(origin),
        block_num: pos.blocknum,
        tx_hash: pos.tx_hash.clone(),
        contract_id,
        command: inner_data_string.to_string(),
        status,
    });
}

fn dispatch(block: &chain::SignedBlock, ecdh_privkey: &EcdhKey) {
    let ref mut state = STATE.lock().unwrap();
    for (i, xt) in block.block.extrinsics.iter().enumerate() {
        if let chain::Call::PhalaModule(chain::pallet_phala::Call::push_command(contract_id, payload)) = &xt.function {
            println!("push_command(contract_id: {}, payload: data[{}])", contract_id, payload.len());
            let pos = TxRef {
                blocknum: block.block.header.number,
                index: i as u32,
                tx_hash: hex::encode_hex_compact(&blake2_256(&xt.encode())),
            };
            handle_execution(state, &pos, xt.signature.clone(), *contract_id, payload, ecdh_privkey);
        }
        // skip other unknown extrinsics
    }
}

fn sync_block(input: SyncBlockReq) -> Result<Value, Value> {
    // Parse base64 to data
    let parsed_data: Result<Vec<_>, _> = (&input.blocks_b64)
        .iter()
        .map(base64::decode)
        .collect();
    let blocks_data = parsed_data
        .map_err(|_| error_msg("Failed to parse base64 block"))?;
    // Parse data to blocks
    let parsed_blocks: Result<Vec<BlockWithEvents>, _> = blocks_data
        .iter()
        .map(|d| Decode::decode(&mut &d[..]))
        .collect();
    let blocks = parsed_blocks.map_err(|_| error_msg("Invalid block"))?;
    // Light validation when possible
    let last_block = &blocks.last().ok_or_else(|| error_msg("No block in the request"))?.block;
    {
        // 1. the last block must has justification
        let justification = last_block.justification.as_ref()
            .ok_or_else(|| error_msg("Missing justification"))?
            .clone();
        let header = last_block.block.header.clone();
        // 2. check block sequence
        for (i, block) in blocks.iter().enumerate() {
            if i > 0 && blocks[i-1].block.block.header.hash() != block.block.block.header.parent_hash {
                return Err(error_msg("Incorrect block order"));
            }
        }
        // 3. generate accenstor proof
        let mut accenstor_proof: Vec<_> = blocks[0..blocks.len()-1]
            .iter()
            .map(|b| b.block.block.header.clone())
            .collect();
        accenstor_proof.reverse();  // from high to low
        // 4. submit to light client
        let mut state = STATE.lock().unwrap();
        let bridge_id = state.main_bridge;
        let authority_set_change = input.authority_set_change_b64
            .map(|b64| parse_authority_set_change(b64))
            .transpose()?;
        state.light_client.submit_finalized_headers(
            bridge_id,
            header,
            accenstor_proof,
            justification,
            authority_set_change
        ).map_err(|e| error_msg(format!("Light validation failed {:?}", e).as_str()))?
    }
    // Passed the validation
    let mut local_state = LOCAL_STATE.lock().unwrap();
    let ecdh_privkey = ecdh::clone_key(
        local_state.ecdh_private_key.as_ref().expect("ECDH not initizlied"));
    let mut last_block = 0;
    for block_with_events in blocks.iter() {
        let block = &block_with_events.block;
        if block.block.header.number != local_state.blocknum {
            return Err(error_msg("Unexpected block"))
        }
        dispatch(block, &ecdh_privkey);

        if block_with_events.events.is_some() {
            parse_events(&block_with_events)?;
        }

        // move forward
        last_block = block.block.header.number;
        (*local_state).blocknum = last_block + 1;
    }

    Ok(json!({
        "synced_to": last_block
    }))
}

fn parse_authority_set_change(data_b64: String) -> Result<AuthoritySetChange, Value> {
    let data = base64::decode(&data_b64)
        .map_err(|_| error_msg("cannot decode authority_set_change_b64"))?;
    AuthoritySetChange::decode(&mut &data[..])
        .map_err(|_| error_msg("cannot decode authority_set_change"))
}

fn parse_events(block_with_events: &BlockWithEvents) -> Result<(), Value> {
    let mut state = STATE.lock().unwrap();
    let missing_field = error_msg("Missing field");
    let events = block_with_events.clone().events.ok_or(missing_field.clone())?;
    let proof = block_with_events.clone().proof.ok_or(missing_field.clone())?;
    let key = block_with_events.clone().key.ok_or(missing_field)?;
    let state_root = &block_with_events.block.block.header.state_root;
    state.light_client.validate_events_proof(&state_root, proof, events.clone(), key).map_err(|_| error_msg("bad storage proof"))?;

    let events = Vec::<EventRecord<chain::Event, Hash>>::decode(&mut events.as_slice());
    if let Ok(evts) = events {
        for evt in &evts {
            if let chain::Event::pallet_phala(pe) = &evt.event {
                println!("pallet_phala event: {:?}", pe);
                state.contract2.handle_event(evt.event.clone());
            }
        }

        return Ok(());
    }

    Err(error_msg("decode events error"))
}

fn get_info(_input: &Map<String, Value>) -> Result<Value, Value> {
    let local_state = LOCAL_STATE.lock().unwrap();

    let initialized = local_state.initialized;
    let pk = &local_state.public_key;
    let s_pk = hex::encode_hex_compact(pk.serialize_compressed().as_ref());
    let s_ecdh_pk = match &local_state.ecdh_public_key {
        Some(ecdh_public_key) => hex::encode_hex_compact(ecdh_public_key.as_ref()),
        None => "".to_string()
    };
    let blocknum = local_state.blocknum;

    Ok(json!({
        "initialized": initialized,
        "public_key": s_pk,
        "ecdh_public_key": s_ecdh_pk,
        "blocknum": blocknum
    }))
}

fn query(q: types::SignedQuery) -> Result<Value, Value> {
    let payload_data = q.query_payload.as_bytes();
    // Validate signature
    if let Some(origin) = &q.origin {
        if !origin.verify(payload_data).map_err(|_| error_msg("Bad signature or origin"))? {
            return Err(error_msg("Verifying signature failed"));
        }
        println!("Verifying signature passed!");
    }
    // Load and decrypt if necessary
    let payload: types::Payload = serde_json::from_slice(payload_data)
        .expect("Failed to decode payload");
    let (msg, secret, pubkey) = {
        let local_state = LOCAL_STATE.lock().unwrap();
        let ecdh_privkey = local_state.ecdh_private_key.as_ref().expect("ECDH not initizlied");
        match payload {
            types::Payload::Plain(data) => (data.into_bytes(), None, None),
            types::Payload::Cipher(cipher) => {
                println!("cipher: {:?}", cipher);
                let result = cryptography::decrypt(&cipher, ecdh_privkey).expect("Decrypt failed");
                (result.msg, Some(result.secret), local_state.ecdh_public_key.clone())
            }
        }
    };
    println!("msg: {}", String::from_utf8_lossy(&msg));
    let opaque_query: types::OpaqueQuery = serde_json::from_slice(&msg)
        .map_err(|_| error_msg("Malformed request (Query)"))?;
    // Origin
    let accid_origin = match q.origin.as_ref() {
        Some(o) => {
            let accid = contracts::account_id_from_hex(&o.origin)
                .map_err(|_| error_msg("Bad origin"))?;
            Some(accid)
        },
        None => None
    };
    // Dispatch
    let mut state = STATE.lock().unwrap();
    let res = match opaque_query.contract_id {
        DATA_PLAZA => serde_json::to_value(
            state.contract1.handle_query(
                accid_origin.as_ref(),
                types::deopaque_query(opaque_query)
                    .map_err(|_| error_msg("Malformed request (data_plaza::Request)"))?.request)
        ).unwrap(),
        BALANCE => serde_json::to_value(
            state.contract2.handle_query(
                accid_origin.as_ref(),
                types::deopaque_query(opaque_query)
                    .map_err(|_| error_msg("Malformed request (balance::Request)"))?.request)
        ).unwrap(),
        ASSETS => serde_json::to_value(
            state.contract3.handle_query(
                accid_origin.as_ref(),
                types::deopaque_query(opaque_query)
                    .map_err(|_| error_msg("Malformed request (assets::Request)"))?.request)
        ).unwrap(),
        SYSTEM => serde_json::to_value(
            handle_query_receipt(
                accid_origin.clone(),
                types::deopaque_query(opaque_query)
                    .map_err(|_| error_msg("Malformed request (system::Request)"))?.request)
        ).unwrap(),
        _ => return Err(Value::Null)
    };
    // Encrypt response if necessary
    let res_json = res.to_string();
    let res_payload = if let (Some(sk), Some(pk)) = (secret, pubkey) {
        let iv = aead::generate_iv();
        let mut msg = res_json.as_bytes().to_vec();
        aead::encrypt(&iv, &sk, &mut msg);
        types::Payload::Cipher(cryptography::AeadCipher {
            iv_b64: base64::encode(&iv),
            cipher_b64: base64::encode(&msg),
            pubkey_b64: base64::encode(&pk),
        })
    } else {
        types::Payload::Plain(res_json)
    };

    let res_value = serde_json::to_value(res_payload).unwrap();
    Ok(res_value)
}

fn handle_query_receipt(accid_origin: Option<chain::AccountId>, req: Request) -> Response {
    let inner = || -> Result<Response, Error> {
        match req {
            Request::QueryReceipt{tx_hash} => {
                let gr = GLOBAL_RECEIPT.lock().unwrap();
                match gr.get_receipt(tx_hash) {
                    Some(receipt) => {
                        if receipt.account == AccountIdWrapper(accid_origin.unwrap()) {
                            Ok(Response::QueryReceipt { receipt: receipt.clone() })
                        } else {
                            Ok(Response::Error(Error::NotAuthorized))
                        }
                    },
                    None => Ok(Response::Error(Error::Other(String::from("Transaction hash not found")))),
                }
            }
        }
    };

    match inner() {
        Err(error) => Response::Error(error),
        Ok(resp) => resp
    }
}

fn get(input: &Map<String, Value>) -> Result<Value, Value> {
    let state = STATE.lock().unwrap();
    let path = input.get("path").unwrap().as_str().unwrap();

    let data = match state.contract1.get(&path.to_string()) {
        Some(d) => d,
        None => {
            return Err(error_msg("Data doesn't exist"))
        }
    };

    let data_b64 = base64::encode(data);

    Ok(json!({
        "path": path.to_string(),
        "value": data_b64
    }))
}

fn set(input: &Map<String, Value>) -> Result<Value, Value> {
    let mut state = STATE.lock().unwrap();
    let path = input.get("path").unwrap().as_str().unwrap();
    let data_b64 = input.get("data").unwrap().as_str().unwrap();

    let data = base64::decode(data_b64).map_err(|_| error_msg("Failed to decode base64 data"))?;
    state.contract1.set(path.to_string(), data);

    Ok(json!({
        "path": path.to_string(),
        "data": data_b64.to_string()
    }))
}

lazy_static! {
    static ref GLOBAL_RECEIPT: SgxMutex<ReceiptStore> = {
        SgxMutex::new(ReceiptStore::new())
    };
}

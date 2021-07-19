#![crate_name = "enclaveapp"]
#![crate_type = "staticlib"]
#![warn(unused_imports)]
#![warn(unused_extern_crates)]
#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]
#![feature(bench_black_box)]

#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;

#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

extern crate runtime as chain;

use sgx_rand::*;
use sgx_tcrypto::*;
use sgx_tse::*;
use sgx_types::*;

use sgx_tseal::SgxSealedData;
use sgx_types::marker::ContiguousMemory;
use sgx_types::{sgx_sealed_data_t, sgx_status_t};

use crate::light_validation::LightValidation;
use crate::msg_channel::osp::{KeyPair, PeelingReceiver};
use crate::std::collections::BTreeMap;
use crate::std::prelude::v1::*;
use crate::std::ptr;
use crate::std::str;
use crate::std::string::String;
use crate::std::sync::SgxMutex;
use crate::std::vec::Vec;

use anyhow::{anyhow, Result};
use core::convert::TryInto;
use itertools::Itertools;
use log::{debug, error, info, warn};
use parity_scale_codec::{Decode, Encode};
use ring::rand::SecureRandom;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde_cbor;
use serde_json::{Map, Value};
use sp_core::{crypto::Pair, ecdsa};

use http_req::request::{Method, Request};
use std::time::Duration;

use pink::InkModule;

use enclave_api::storage_sync::{
    ParachainSynchronizer, SolochainSynchronizer, StorageSynchronizer,
};
use enclave_api::{
    actions::*,
    blocks::{self, SyncParachainHeaderReq},
};

use phala_crypto::{aead, ecdh::EcdhKey, secp256k1::KDF};
use phala_mq::{BindTopic, MessageDispatcher, MessageOrigin, MessageSendQueue};
use phala_pallets::pallet_mq;
use phala_types::PRuntimeInfo;

mod benchmark;
mod cert;
mod contracts;
mod cryptography;
mod light_validation;
mod msg_channel;
mod rpc_types;
mod storage;
mod system;
mod types;
mod utils;

use crate::light_validation::utils::{storage_map_prefix_twox_64_concat, storage_prefix};
use contracts::{ContractId, ExecuteEnv, SYSTEM};
use rpc_types::*;
use storage::{Storage, StorageExt};
use system::{GatekeeperRole, TransactionStatus};
use types::BlockInfo;
use types::Error;

// TODO: Completely remove the reference to Phala/Khala runtime. Instead we can create a minimal
// runtime definition locally.
type RuntimeHasher = <chain::Runtime as frame_system::Config>::Hashing;

extern "C" {
    pub fn ocall_load_ias_spid(
        ret_val: *mut sgx_status_t,
        key_ptr: *mut u8,
        key_len_ptr: *mut usize,
        key_buf_len: usize,
    ) -> sgx_status_t;

    pub fn ocall_load_ias_key(
        ret_val: *mut sgx_status_t,
        key_ptr: *mut u8,
        key_len_ptr: *mut usize,
        key_buf_len: usize,
    ) -> sgx_status_t;

    pub fn ocall_sgx_init_quote(
        ret_val: *mut sgx_status_t,
        ret_ti: *mut sgx_target_info_t,
        ret_gid: *mut sgx_epid_group_id_t,
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
        p_quote_len: *mut u32,
    ) -> sgx_status_t;

    pub fn ocall_dump_state(
        ret_val: *mut sgx_status_t,
        output_ptr: *mut u8,
        output_len_ptr: *mut usize,
        output_buf_len: usize,
    ) -> sgx_status_t;

    pub fn ocall_save_persistent_data(
        ret_val: *mut sgx_status_t,
        input_ptr: *const u8,
        input_len: usize,
    ) -> sgx_status_t;

    pub fn ocall_load_persistent_data(
        ret_val: *mut sgx_status_t,
        output_ptr: *mut u8,
        output_len_ptr: *mut usize,
        output_buf_len: usize,
    ) -> sgx_status_t;
}

pub const VERSION: u32 = 1;
pub const IAS_HOST: &'static str = env!("IAS_HOST");
pub const IAS_SIGRL_ENDPOINT: &'static str = env!("IAS_SIGRL_ENDPOINT");
pub const IAS_REPORT_ENDPOINT: &'static str = env!("IAS_REPORT_ENDPOINT");

struct RuntimeState {
    contracts: BTreeMap<ContractId, Box<dyn contracts::Contract + Send>>,
    send_mq: MessageSendQueue,
    recv_mq: MessageDispatcher,

    // chain storage synchonizing
    storage_synchronizer: Box<dyn StorageSynchronizer + Send>,
    chain_storage: Storage,
}

struct LocalState {
    initialized: bool,
    identity_key: Option<ecdsa::Pair>,
    ecdh_key: Option<EcdhKey>,
    machine_id: [u8; 16],
    dev_mode: bool,
    runtime_info: Option<InitRuntimeResp>,
}

struct TestContract {
    name: String,
    code: Vec<u8>,
    initial_data: Vec<u8>,
    txs: Vec<Vec<u8>>,
}

impl RuntimeState {
    fn purge_mq(&mut self) {
        self.send_mq.purge(|sender| {
            use pallet_mq::StorageMapTrait as _;
            type OffchainIngress = pallet_mq::OffchainIngress<chain::Runtime>;

            let module_prefix = OffchainIngress::module_prefix();
            let storage_prefix = OffchainIngress::storage_prefix();
            let key = storage_map_prefix_twox_64_concat(module_prefix, storage_prefix, sender);
            let sequence: u64 = self.chain_storage.get_decoded(&key).unwrap_or(0);
            debug!("purging, sequence = {}", sequence);
            sequence
        })
    }
}

fn se_to_b64<S, V: Encode>(value: &V, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let data = value.encode();
    let s = base64::encode(data.as_slice());
    String::serialize(&s, serializer)
}

fn de_from_b64<'de, D, V: Decode>(deserializer: D) -> Result<V, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let data = base64::decode(&s).map_err(de::Error::custom)?;
    V::decode(&mut data.as_slice()).map_err(|_| de::Error::custom("bad data"))
}

fn to_sealed_log_for_slice<T: Copy + ContiguousMemory>(
    sealed_data: &SgxSealedData<[T]>,
    sealed_log: *mut u8,
    sealed_log_size: u32,
) -> Option<*mut sgx_sealed_data_t> {
    unsafe {
        sealed_data.to_raw_sealed_data_t(sealed_log as *mut sgx_sealed_data_t, sealed_log_size)
    }
}

fn from_sealed_log_for_slice<'a, T: Copy + ContiguousMemory>(
    sealed_log: *mut u8,
    sealed_log_size: u32,
) -> Option<SgxSealedData<'a, [T]>> {
    unsafe {
        SgxSealedData::<[T]>::from_raw_sealed_data_t(
            sealed_log as *mut sgx_sealed_data_t,
            sealed_log_size,
        )
    }
}

lazy_static! {
    static ref STATE: SgxMutex<Option<RuntimeState>> = Default::default();
    static ref LOCAL_STATE: SgxMutex<LocalState> = {
        SgxMutex::new(LocalState {
            initialized: false,
            identity_key: None,
            ecdh_key: None,
            machine_id: [0; 16],
            dev_mode: false,
            runtime_info: None,
        })
    };
    static ref SYSTEM_STATE: SgxMutex<Option<system::System>> = Default::default();
}

fn ias_spid() -> sgx_spid_t {
    // Try load persisted sealed data
    let mut key_buf = vec![0; 256].into_boxed_slice();
    let mut key_len: usize = 0;
    let key_slice = &mut key_buf;
    let key_ptr = key_slice.as_mut_ptr();
    let key_len_ptr = &mut key_len as *mut usize;

    let mut retval = sgx_status_t::SGX_SUCCESS;
    let load_result = unsafe { ocall_load_ias_spid(&mut retval, key_ptr, key_len_ptr, 256) };

    if load_result != sgx_status_t::SGX_SUCCESS || key_len == 0 {
        panic!("Load SPID failure.");
    }

    let key_str = str::from_utf8(key_slice).unwrap();
    // println!("IAS SPID: {}", key_str.to_owned());

    utils::decode_spid(&key_str[..key_len])
}

fn ias_key() -> String {
    // Try load persisted sealed data
    let mut key_buf = vec![0; 256].into_boxed_slice();
    let mut key_len: usize = 0;
    let key_slice = &mut key_buf;
    let key_ptr = key_slice.as_mut_ptr();
    let key_len_ptr = &mut key_len as *mut usize;

    let mut retval = sgx_status_t::SGX_SUCCESS;
    let load_result = unsafe { ocall_load_ias_key(&mut retval, key_ptr, key_len_ptr, 256) };
    if load_result != sgx_status_t::SGX_SUCCESS || key_len == 0 {
        panic!("Load IAS KEY failure.");
    }

    let key_str = str::from_utf8(key_slice).unwrap();
    // println!("IAS KEY: {}", key_str.to_owned());

    key_str[..key_len].to_owned()
}

pub fn get_sigrl_from_intel(gid: u32) -> Vec<u8> {
    // println!("get_sigrl_from_intel fd = {:?}", fd);
    //let sigrl_arg = SigRLArg { group_id : gid };
    //let sigrl_req = sigrl_arg.to_httpreq();

    let mut res_body_buffer = Vec::new(); //container for body of a response
    let timeout = Some(Duration::from_secs(8));

    let url = format!("https://{}{}/{:08x}", IAS_HOST, IAS_SIGRL_ENDPOINT, gid)
        .parse()
        .unwrap();
    let res = Request::new(&url)
        .header("Connection", "Close")
        .header("Ocp-Apim-Subscription-Key", &ias_key())
        .timeout(timeout)
        .connect_timeout(timeout)
        .read_timeout(timeout)
        .send(&mut res_body_buffer)
        .unwrap();

    // parse_response_sigrl

    let status_code = u16::from(res.status_code());
    if status_code != 200 {
        let msg = match status_code {
            401 => "Unauthorized Failed to authenticate or authorize request.",
            404 => "Not Found GID does not refer to a valid EPID group ID.",
            500 => "Internal error occurred",
            503 => {
                "Service is currently not able to process the request (due to
                a temporary overloading or maintenance). This is a
                temporary state – the same request can be repeated after
                some time. "
            }
            _ => "Unknown error occured",
        };

        error!("{}", msg);
        // TODO: should return Err
        panic!("status code {}", status_code);
    }

    if res.content_len() != None && res.content_len() != Some(0) {
        let res_body = res_body_buffer.clone();
        let encoded_sigrl = str::from_utf8(&res_body).unwrap();
        info!("Base64-encoded SigRL: {:?}", encoded_sigrl);

        return base64::decode(encoded_sigrl).unwrap();
    }

    Vec::new()
}

// TODO: support pse
pub fn get_report_from_intel(quote: Vec<u8>) -> (String, String, String) {
    // println!("get_report_from_intel fd = {:?}", fd);
    let encoded_quote = base64::encode(&quote[..]);
    let encoded_json = format!("{{\"isvEnclaveQuote\":\"{}\"}}\r\n", encoded_quote);

    let ias_key = ias_key();

    let mut res_body_buffer = Vec::new(); //container for body of a response
    let timeout = Some(Duration::from_secs(8));

    let url = format!("https://{}{}", IAS_HOST, IAS_REPORT_ENDPOINT)
        .parse()
        .unwrap();
    let res = Request::new(&url)
        .header("Connection", "Close")
        .header("Content-Type", "application/json")
        .header("Content-Length", &encoded_json.len())
        .header("Ocp-Apim-Subscription-Key", &ias_key)
        .method(Method::POST)
        .body(encoded_json.as_bytes())
        .timeout(timeout)
        .connect_timeout(timeout)
        .read_timeout(timeout)
        .send(&mut res_body_buffer)
        .unwrap();

    let status_code = u16::from(res.status_code());
    if status_code != 200 {
        let msg = match status_code {
            401 => "Unauthorized Failed to authenticate or authorize request.",
            404 => "Not Found GID does not refer to a valid EPID group ID.",
            500 => "Internal error occurred",
            503 => {
                "Service is currently not able to process the request (due to
                a temporary overloading or maintenance). This is a
                temporary state – the same request can be repeated after
                some time. "
            }
            _ => "Unknown error occured",
        };

        error!("{}", msg);
        // TODO: should return Err
        panic!("status code not 200");
    }

    let content_len = match res.content_len() {
        Some(len) => len,
        _ => {
            warn!("content_length not found");
            0
        }
    };

    if content_len == 0 {
        // TODO: should return Err
        panic!("don't know how to handle content_length is 0");
    }

    let attn_report = String::from_utf8(res_body_buffer).unwrap();
    let sig = res
        .headers()
        .get("X-IASReport-Signature")
        .unwrap()
        .to_string();
    let mut cert = res
        .headers()
        .get("X-IASReport-Signing-Certificate")
        .unwrap()
        .to_string();

    // Remove %0A from cert, and only obtain the signing cert
    cert = cert.replace("%0A", "");
    cert = cert::percent_decode(cert);
    let v: Vec<&str> = cert.split("-----").collect();
    let sig_cert = v[2].to_string();

    // len_num == 0
    (attn_report, sig, sig_cert)
}

fn as_u32_le(array: &[u8; 4]) -> u32 {
    ((array[0] as u32) << 0)
        + ((array[1] as u32) << 8)
        + ((array[2] as u32) << 16)
        + ((array[3] as u32) << 24)
}

#[allow(const_err)]
pub fn create_attestation_report(
    data: &[u8],
    sign_type: sgx_quote_sign_type_t,
) -> Result<(String, String, String)> {
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
    let mut ti: sgx_target_info_t = sgx_target_info_t::default();
    let mut eg: sgx_epid_group_id_t = sgx_epid_group_id_t::default();
    let mut rt: sgx_status_t = sgx_status_t::SGX_ERROR_UNEXPECTED;

    let res = unsafe {
        ocall_sgx_init_quote(
            &mut rt as *mut sgx_status_t,
            &mut ti as *mut sgx_target_info_t,
            &mut eg as *mut sgx_epid_group_id_t,
        )
    };

    info!("eg = {:?}", eg);

    if res != sgx_status_t::SGX_SUCCESS {
        return Err(anyhow::Error::msg(res));
    }

    if rt != sgx_status_t::SGX_SUCCESS {
        return Err(anyhow::Error::msg(rt));
    }

    let eg_num = as_u32_le(&eg);

    //println!("Got ias_sock = {}", ias_sock);

    // Now sigrl_vec is the revocation list, a vec<u8>
    let sigrl_vec: Vec<u8> = get_sigrl_from_intel(eg_num);

    // (2) Generate the report
    // Fill data into report_data
    let mut report_data: sgx_report_data_t = sgx_report_data_t::default();
    report_data.d[..data_len].clone_from_slice(data);

    let rep = match rsgx_create_report(&ti, &report_data) {
        Ok(r) => {
            info!("Report creation => success {:?}", r.body.mr_signer.m);
            Some(r)
        }
        Err(e) => {
            warn!("Report creation => failed {:?}", e);
            None
        }
    };

    let mut quote_nonce = sgx_quote_nonce_t { rand: [0; 16] };
    let mut os_rng = os::SgxRng::new().unwrap();
    os_rng.fill_bytes(&mut quote_nonce.rand);
    info!("rand finished");
    let mut qe_report = sgx_report_t::default();
    const RET_QUOTE_BUF_LEN: u32 = 2048;
    let mut return_quote_buf: [u8; RET_QUOTE_BUF_LEN as usize] = [0; RET_QUOTE_BUF_LEN as usize];
    let mut quote_len: u32 = 0;

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
    let (p_sigrl, sigrl_len) = if sigrl_vec.len() == 0 {
        (ptr::null(), 0)
    } else {
        (sigrl_vec.as_ptr(), sigrl_vec.len() as u32)
    };
    let p_report = (&rep.unwrap()) as *const sgx_report_t;
    let quote_type = sign_type;

    let spid: sgx_spid_t = ias_spid();

    let p_spid = &spid as *const sgx_spid_t;
    let p_nonce = &quote_nonce as *const sgx_quote_nonce_t;
    let p_qe_report = &mut qe_report as *mut sgx_report_t;
    let p_quote = return_quote_buf.as_mut_ptr();
    let maxlen = RET_QUOTE_BUF_LEN;
    let p_quote_len = &mut quote_len as *mut u32;

    let result = unsafe {
        ocall_get_quote(
            &mut rt as *mut sgx_status_t,
            p_sigrl,
            sigrl_len,
            p_report,
            quote_type,
            p_spid,
            p_nonce,
            p_qe_report,
            p_quote,
            maxlen,
            p_quote_len,
        )
    };

    if result != sgx_status_t::SGX_SUCCESS {
        return Err(anyhow::Error::msg(result));
    }

    if rt != sgx_status_t::SGX_SUCCESS {
        error!("ocall_get_quote returned {}", rt);
        return Err(anyhow::Error::msg(rt));
    }

    // Added 09-28-2018
    // Perform a check on qe_report to verify if the qe_report is valid
    match rsgx_verify_report(&qe_report) {
        Ok(()) => info!("rsgx_verify_report passed!"),
        Err(x) => {
            error!("rsgx_verify_report failed with {:?}", x);
            return Err(anyhow::Error::msg(x));
        }
    }

    // Check if the qe_report is produced on the same platform
    if ti.mr_enclave.m != qe_report.body.mr_enclave.m
        || ti.attributes.flags != qe_report.body.attributes.flags
        || ti.attributes.xfrm != qe_report.body.attributes.xfrm
    {
        error!("qe_report does not match current target_info!");
        return Err(anyhow::Error::msg(sgx_status_t::SGX_ERROR_UNEXPECTED));
    }

    info!("qe_report check passed");

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

    let mut rhs_vec: Vec<u8> = quote_nonce.rand.to_vec();
    rhs_vec.extend(&return_quote_buf[..quote_len as usize]);
    let rhs_hash = rsgx_sha256_slice(&rhs_vec[..]).unwrap();
    let lhs_hash = &qe_report.body.report_data.d[..32];

    info!("rhs hash = {:02X}", rhs_hash.iter().format(""));
    info!("report hs= {:02X}", lhs_hash.iter().format(""));

    if rhs_hash != lhs_hash {
        error!("Quote is tampered!");
        return Err(anyhow::Error::msg(sgx_status_t::SGX_ERROR_UNEXPECTED));
    }

    let quote_vec: Vec<u8> = return_quote_buf[..quote_len as usize].to_vec();
    let (attn_report, sig, cert) = get_report_from_intel(quote_vec);
    Ok((attn_report, sig, cert))
}

fn generate_seal_key() -> [u8; 16] {
    let key_request = sgx_key_request_t {
        key_name: SGX_KEYSELECT_SEAL,
        key_policy: SGX_KEYPOLICY_MRSIGNER,
        isv_svn: 0_u16,
        reserved1: 0_u16,
        cpu_svn: sgx_cpu_svn_t { svn: [0_u8; 16] },
        attribute_mask: sgx_attributes_t { flags: 0, xfrm: 0 },
        key_id: sgx_key_id_t::default(),
        misc_mask: 0,
        config_svn: 0_u16,
        reserved2: [0_u8; SGX_KEY_REQUEST_RESERVED2_BYTES],
    };
    let seal_key = rsgx_get_align_key(&key_request).unwrap();
    seal_key.key
}

#[no_mangle]
pub extern "C" fn ecall_set_state(input_ptr: *const u8, input_len: usize) -> sgx_status_t {
    let input_slice = unsafe { std::slice::from_raw_parts(input_ptr, input_len) };
    let input_value: serde_json::value::Value = serde_json::from_slice(input_slice).unwrap();
    let _input = input_value.as_object().unwrap();

    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub extern "C" fn ecall_handle(
    action: u8,
    input_ptr: *const u8,
    input_len: usize,
    output_ptr: *mut u8,
    output_len_ptr: *mut usize,
    output_buf_len: usize,
) -> sgx_status_t {
    let input_slice = unsafe { std::slice::from_raw_parts(input_ptr, input_len) };

    let result = if action < BIN_ACTION_START {
        handle_json_api(action, input_slice)
    } else {
        handle_scale_api(action, input_slice)
    };

    let (status, payload) = match result {
        Ok(payload) => ("ok", payload),
        Err(payload) => ("error", payload),
    };

    // Sign the output payload
    let local_state = LOCAL_STATE.lock().unwrap();
    let str_payload = payload.to_string();
    let signature: Option<String> = local_state.identity_key.as_ref().map(|pair| {
        let bytes = str_payload.as_bytes();
        let sig = pair.sign(bytes).0;
        hex::encode(&sig)
    });
    let output_json = json!({
        "status": status,
        "payload": str_payload,
        "signature": signature,
    });
    info!("{}", output_json.to_string());

    let output = serde_json::to_vec(&output_json).unwrap();

    let output_len = output.len();
    if output_len <= output_buf_len {
        unsafe {
            ptr::copy_nonoverlapping(output.as_ptr(), output_ptr, output_len);
            *output_len_ptr = output_len;
            sgx_status_t::SGX_SUCCESS
        }
    } else {
        warn!("Too much output. Buffer overflow.");
        sgx_status_t::SGX_ERROR_FAAS_BUFFER_TOO_SHORT
    }
}

fn handle_json_api(action: u8, input: &[u8]) -> Result<Value, Value> {
    let input: serde_json::value::Value = serde_json::from_slice(input).unwrap();
    let input_value = input.get("input").unwrap().clone();

    // Strong typed
    fn load_param<T: de::DeserializeOwned>(input_value: serde_json::value::Value) -> T {
        serde_json::from_value(input_value).unwrap()
    }

    match action {
        ACTION_INIT_RUNTIME => init_runtime(load_param(input_value)),
        ACTION_TEST => test(load_param(input_value)),
        ACTION_QUERY => query(load_param(input_value)),
        _ => {
            let payload = input_value.as_object().unwrap();
            match action {
                ACTION_GET_INFO => get_info(payload),
                ACTION_DUMP_STATES => dump_states(payload),
                ACTION_LOAD_STATES => load_states(payload),
                ACTION_GET_RUNTIME_INFO => get_runtime_info(payload),
                ACTION_TEST_INK => test_ink(payload),
                ACTION_GET_EGRESS_MESSAGES => get_egress_messages(),
                _ => unknown(),
            }
        }
    }
}

fn handle_scale_api(action: u8, input: &[u8]) -> Result<Value, Value> {
    fn load_scale<T: Decode>(mut scale: &[u8]) -> Result<T, Value> {
        Decode::decode(&mut scale).map_err(|_| error_msg("Decode input parameter failed"))
    }

    match action {
        BIN_ACTION_SYNC_HEADER => sync_header(load_scale(input)?),
        BIN_ACTION_SYNC_PARA_HEADER => sync_para_header(load_scale(input)?),
        BIN_ACTION_DISPATCH_BLOCK => dispatch_block(load_scale(input)?),
        _ => unknown(),
    }
}

const SEAL_DATA_BUF_MAX_LEN: usize = 2048 as usize;

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
struct PersistentRuntimeData {
    version: u32,
    sk: String,
    dev_mode: bool,
}

fn save_secret_keys(ecdsa_sk: ecdsa::Pair, dev_mode: bool) -> Result<PersistentRuntimeData> {
    // Put in PresistentRuntimeData
    let serialized_sk = ecdsa_sk.to_raw_vec();

    let data = PersistentRuntimeData {
        version: 1,
        sk: hex::encode(&serialized_sk),
        dev_mode,
    };
    let encoded_vec = serde_cbor::to_vec(&data).unwrap();
    let encoded_slice = encoded_vec.as_slice();
    info!("Length of encoded slice: {}", encoded_slice.len());
    info!("Encoded slice: {:?}", hex::encode(encoded_slice));

    // Seal
    let aad: [u8; 0] = [0_u8; 0];
    let sealed_data =
        SgxSealedData::<[u8]>::seal_data(&aad, encoded_slice).map_err(anyhow::Error::msg)?;

    let mut return_output_buf = vec![0; SEAL_DATA_BUF_MAX_LEN].into_boxed_slice();
    let output_len: usize = return_output_buf.len();

    let output_slice = &mut return_output_buf;
    let output_ptr = output_slice.as_mut_ptr();

    let opt = to_sealed_log_for_slice(&sealed_data, output_ptr, output_len as u32);
    if opt.is_none() {
        return Err(anyhow::Error::msg(
            sgx_status_t::SGX_ERROR_INVALID_PARAMETER,
        ));
    }

    // TODO: check retval and result
    let mut _retval = sgx_status_t::SGX_SUCCESS;
    let _result = unsafe { ocall_save_persistent_data(&mut _retval, output_ptr, output_len) };
    info!("Persistent Runtime Data saved");
    Ok(data)
}

fn load_secret_keys() -> Result<PersistentRuntimeData> {
    // Try load persisted sealed data
    let mut sealed_data_buf = vec![0; SEAL_DATA_BUF_MAX_LEN].into_boxed_slice();
    let mut sealed_data_len: usize = 0;
    let sealed_data_slice = &mut sealed_data_buf;
    let sealed_data_ptr = sealed_data_slice.as_mut_ptr();
    let sealed_data_len_ptr = &mut sealed_data_len as *mut usize;

    let mut retval = sgx_status_t::SGX_SUCCESS;
    let load_result = unsafe {
        ocall_load_persistent_data(
            &mut retval,
            sealed_data_ptr,
            sealed_data_len_ptr,
            SEAL_DATA_BUF_MAX_LEN,
        )
    };
    if load_result != sgx_status_t::SGX_SUCCESS || sealed_data_len == 0 {
        return Err(anyhow::Error::msg(Error::PersistentRuntimeNotFound));
    }

    let opt = from_sealed_log_for_slice::<u8>(sealed_data_ptr, sealed_data_len as u32);
    let sealed_data = match opt {
        Some(x) => x,
        None => {
            panic!("Sealed data corrupted or outdated, please delete it.")
        }
    };

    let unsealed_data = sealed_data.unseal_data().map_err(anyhow::Error::msg)?;
    let encoded_slice = unsealed_data.get_decrypt_txt();
    info!("Length of encoded slice: {}", encoded_slice.len());
    info!("Encoded slice: {:?}", hex::encode(encoded_slice));

    serde_cbor::from_slice(encoded_slice).map_err(|_| anyhow::Error::msg(Error::DecodeError))
}

fn new_ecdsa_key() -> Result<ecdsa::Pair> {
    use rand::RngCore;
    let mut rng = rand::thread_rng();
    let mut seed = [0_u8; 32];
    rng.fill_bytes(&mut seed);
    ecdsa::Pair::from_seed_slice(&seed)
        .map_err(|_| anyhow!("Failed to generate a random ecdsa key"))
}

pub fn generate_random_iv() -> aead::IV {
    let mut nonce_vec = [0u8; aead::IV_BYTES];
    let rand = ring::rand::SystemRandom::new();
    rand.fill(&mut nonce_vec).unwrap();
    nonce_vec
}

fn init_secret_keys(
    local_state: &mut LocalState,
    predefined_identity_key: Option<ecdsa::Pair>,
) -> Result<PersistentRuntimeData> {
    let data = if let Some(ecdsa_sk) = predefined_identity_key {
        save_secret_keys(ecdsa_sk, true)?
    } else {
        match load_secret_keys() {
            Ok(data) => data,
            Err(e)
                if e.is::<Error>()
                    && matches!(
                        e.downcast_ref::<Error>().unwrap(),
                        Error::PersistentRuntimeNotFound
                    ) =>
            {
                warn!("Persistent data not found.");
                let ecdsa_sk = new_ecdsa_key()?;
                save_secret_keys(ecdsa_sk, false)?
            }
            other_err => return other_err,
        }
    };

    // load identity
    let ecdsa_raw_key: [u8; 32] = hex::decode(&data.sk)
        .expect("Unalbe to decode identity key hex")
        .as_slice()
        .try_into()
        .expect("slice with incorrect length");

    let ecdsa_sk = ecdsa::Pair::from_seed(&ecdsa_raw_key);
    info!("Identity pubkey: {:?}", hex::encode(&ecdsa_sk.public()));

    // derive ecdh key
    let ecdh_key = ecdsa_sk
        .derive_ecdh_key()
        .expect("Unable to derive ecdh key");
    let ecdh_hex_pk = hex::encode(ecdh_key.public().as_ref());
    info!("ECDH pubkey: {:?}", ecdh_hex_pk);

    // Generate Seal Key as Machine Id
    // This SHOULD be stable on the same CPU
    let machine_id = generate_seal_key();
    info!("Machine id: {:?}", hex::encode(&machine_id));

    // Save
    local_state.identity_key = Some(ecdsa_sk);
    local_state.ecdh_key = Some(ecdh_key);
    local_state.machine_id = machine_id.clone();
    local_state.dev_mode = data.dev_mode;

    info!("Init done.");
    Ok(data)
}

#[no_mangle]
pub extern "C" fn ecall_init() -> sgx_status_t {
    env_logger::from_env(env_logger::Env::default().default_filter_or("info")).init();

    benchmark::reset_iteration_counter();

    let mut local_state = LOCAL_STATE.lock().unwrap();
    match init_secret_keys(&mut local_state, None) {
        Err(e) if e.is::<sgx_status_t>() => e.downcast::<sgx_status_t>().unwrap(),
        _ => sgx_status_t::SGX_SUCCESS,
    }
}

#[cfg(feature = "tests")]
#[no_mangle]
pub extern "C" fn ecall_run_tests() -> sgx_status_t {
    run_all_tests();
    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub extern "C" fn ecall_bench_run(index: u32) -> sgx_status_t {
    if !benchmark::puasing() {
        info!("[{}] Benchmark thread started", index);
        benchmark::run();
    }
    sgx_status_t::SGX_SUCCESS
}

// --------------------------------

fn display(e: impl core::fmt::Display) -> Value {
    error_msg(&e.to_string())
}

fn error_msg(msg: &str) -> Value {
    json!({ "message": msg })
}

fn unknown() -> Result<Value, Value> {
    Err(json!({
        "message": "Unknown action"
    }))
}

fn dump_states(_input: &Map<String, Value>) -> Result<Value, Value> {
    todo!("@Kevin")
}

fn load_states(_input: &Map<String, Value>) -> Result<Value, Value> {
    todo!("@Kevin")
}

fn init_runtime(input: InitRuntimeReq) -> Result<Value, Value> {
    let mut local_state = LOCAL_STATE.lock().unwrap();
    if local_state.initialized {
        return Err(json!({"message": "Already initialized"}));
    }

    // load identity
    if let Some(key) = input.debug_set_key {
        if input.skip_ra == false {
            return Err(error_msg("RA is disallowed when debug_set_key is enabled"));
        }
        let raw_key = hex::decode(&key).map_err(|_| error_msg("Can't decode key hex"))?;
        let ecdsa_key = ecdsa::Pair::from_seed_slice(&raw_key)
            .map_err(|_| error_msg("can't parse private key"))?;
        init_secret_keys(&mut local_state, Some(ecdsa_key))
            .map_err(|_| error_msg("failed to update secret key"))?;
    }

    if !input.skip_ra && local_state.dev_mode {
        return Err(error_msg("RA is disallowed when debug_set_key is enabled"));
    }

    let ecdsa_pk = local_state
        .identity_key
        .as_ref()
        .expect("Identity key must be initialized; qed.")
        .public();
    let ecdsa_hex_pk = hex::encode(&ecdsa_pk);
    info!("Identity pubkey: {:?}", ecdsa_hex_pk);

    // derive ecdh key
    let ecdh_pubkey = local_state.ecdh_key.as_ref().unwrap().public();
    let ecdh_hex_pk = hex::encode(ecdh_pubkey.as_ref());
    info!("ECDH pubkey: {:?}", ecdh_hex_pk);

    // Measure machine score
    let cpu_core_num: u32 = sgx_trts::enclave::rsgx_get_cpu_core_num();
    info!("CPU cores: {}", cpu_core_num);

    let mut cpu_feature_level: u32 = 1;
    // Atom doesn't support AVX
    if is_x86_feature_detected!("avx2") {
        info!("CPU Support AVX2");
        cpu_feature_level += 1;

        // Customer-level Core doesn't support AVX512
        if is_x86_feature_detected!("avx512f") {
            info!("CPU Support AVX512");
            cpu_feature_level += 1;
        }
    }

    let operator = match input.operator_hex {
        Some(h) => {
            let raw_address =
                hex::decode(h).map_err(|_| error_msg("Error decoding operator_hex"))?;
            Some(chain::AccountId::new(
                raw_address
                    .try_into()
                    .map_err(|_| error_msg("Bad operator_hex"))?,
            ))
        }
        None => None,
    };

    // Build PRuntimeInfo
    let runtime_info = PRuntimeInfo::<chain::AccountId> {
        version: VERSION,
        machine_id: local_state.machine_id.clone(),
        pubkey: ecdsa_pk,
        ecdh_pubkey: phala_types::EcdhPublicKey(ecdh_pubkey),
        features: vec![cpu_core_num, cpu_feature_level],
        operator,
    };
    let encoded_runtime_info = runtime_info.encode();
    let runtime_info_hash = sp_core::hashing::blake2_256(&encoded_runtime_info);

    info!("Encoded runtime info");
    info!("{:?}", hex::encode(&encoded_runtime_info));

    // Produce remote attestation report
    let mut attestation: Option<InitRespAttestation> = None;
    if !input.skip_ra {
        let (attn_report, sig, cert) = match create_attestation_report(
            &runtime_info_hash,
            sgx_quote_sign_type_t::SGX_LINKABLE_SIGNATURE,
        ) {
            Ok(r) => r,
            Err(e) => {
                error!("Error in create_attestation_report: {:?}", e);
                return Err(json!({"message": "Error while connecting to IAS"}));
            }
        };

        attestation = Some(InitRespAttestation {
            version: 1,
            provider: "SGX".to_string(),
            payload: AttestationReport {
                report: attn_report,
                signature: sig,
                signing_cert: cert,
            },
        });
    }

    // Initialize bridge
    let raw_genesis =
        base64::decode(&input.bridge_genesis_info_b64).expect("Bad bridge_genesis_info_b64");
    let genesis =
        light_validation::BridgeInitInfo::<chain::Runtime>::decode(&mut raw_genesis.as_slice())
            .expect("Can't decode bridge_genesis_info_b64");
    let mut state = STATE.lock().unwrap();
    let mut light_client = LightValidation::new();
    let main_bridge = light_client
        .initialize_bridge(
            genesis.block_header,
            genesis.validator_set,
            genesis.validator_set_proof,
        )
        .expect("Bridge initialize failed");

    let storage_synchronizer = if input.is_parachain {
        Box::new(ParachainSynchronizer::new(light_client, main_bridge)) as _
    } else {
        Box::new(SolochainSynchronizer::new(light_client, main_bridge)) as _
    };

    let id_pair = local_state
        .identity_key
        .clone()
        .expect("Unexpected ecdsa key error in init_runtime");

    let send_mq = MessageSendQueue::default();
    let mut recv_mq = MessageDispatcher::default();

    // Re-init some contracts because they require the identity key
    let mut system_state = SYSTEM_STATE.lock().unwrap();
    *system_state = Some(system::System::new(&id_pair, &send_mq, &mut recv_mq));
    drop(system_state);

    let mut other_contracts: BTreeMap<ContractId, Box<dyn contracts::Contract + Send>> =
        Default::default();

    if local_state.dev_mode {
        // Install contracts when running in dev_mode.

        macro_rules! install_contract {
            ($id: expr, $inner: expr) => {{
                let sender = MessageOrigin::native_contract($id);
                let mq = send_mq.channel(sender, id_pair.clone());
                let cmd_mq = PeelingReceiver::new_plain(recv_mq.subscribe_bound());
                let evt_mq = PeelingReceiver::new_plain(recv_mq.subscribe_bound());
                let wrapped = Box::new(contracts::NativeCompatContract::new(
                    $inner,
                    mq,
                    cmd_mq,
                    evt_mq,
                    KeyPair::new(local_state.ecdh_key.as_ref().unwrap().clone()),
                ));
                other_contracts.insert($id, wrapped);
            }};
        }

        install_contract!(contracts::BALANCES, contracts::balances::Balances::new());
        install_contract!(contracts::ASSETS, contracts::assets::Assets::new());
        install_contract!(contracts::DIEM, contracts::diem::Diem::new());
        install_contract!(
            contracts::SUBSTRATE_KITTIES,
            contracts::substrate_kitties::SubstrateKitties::new()
        );
        install_contract!(
            contracts::BTC_LOTTERY,
            contracts::btc_lottery::BtcLottery::new(Some(id_pair.clone()))
        );
        install_contract!(
            contracts::WEB3_ANALYTICS,
            contracts::web3analytics::Web3Analytics::new()
        );
        install_contract!(
            contracts::DATA_PLAZA,
            contracts::data_plaza::DataPlaza::new()
        );
    }

    let mut runtime_state = RuntimeState {
        contracts: other_contracts,
        send_mq,
        recv_mq,
        storage_synchronizer,
        chain_storage: Default::default(),
    };

    let genesis_state_scl = base64::decode(input.genesis_state_b64)
        .map_err(|_| error_msg("Base64 decode genesis state failed"))?;
    let mut genesis_state_scl = &genesis_state_scl[..];
    let genesis_state: Vec<(Vec<u8>, Vec<u8>)> = Decode::decode(&mut genesis_state_scl)
        .map_err(|_| error_msg("Scale decode genesis state failed"))?;

    // Initialize other states
    runtime_state.chain_storage.load(genesis_state.into_iter());

    info!(
        "Genesis state loaded: {:?}",
        runtime_state.chain_storage.root()
    );

    *state = Some(runtime_state);

    // Response
    let resp = InitRuntimeResp {
        encoded_runtime_info,
        public_key: ecdsa_hex_pk,
        ecdh_public_key: ecdh_hex_pk,
        attestation,
    };

    local_state.runtime_info = Some(resp.clone());
    local_state.initialized = true;

    Ok(serde_json::to_value(resp).unwrap())
}

fn sync_header(input: blocks::SyncHeaderReq) -> Result<Value, Value> {
    info!(
        "sync_header from={:?} to={:?}",
        input.headers.first().map(|h| h.header.number),
        input.headers.last().map(|h| h.header.number)
    );
    let last_header = STATE
        .lock()
        .unwrap()
        .as_mut()
        .ok_or(error_msg("Runtime not initialized"))?
        .storage_synchronizer
        .sync_header(input.headers, input.authority_set_change)
        .map_err(display)?;

    Ok(json!({ "synced_to": last_header }))
}

fn sync_para_header(input: SyncParachainHeaderReq) -> Result<Value, Value> {
    info!(
        "sync_para_header from={:?} to={:?}",
        input.headers.first().map(|h| h.number),
        input.headers.last().map(|h| h.number)
    );
    let mut guard = STATE.lock().unwrap();
    let state = guard.as_mut().ok_or(error_msg("Runtime not initialized"))?;

    let para_id = state
        .chain_storage
        .para_id()
        .ok_or(error_msg("No para_id"))?;

    let storage_key =
        light_validation::utils::storage_map_prefix("Paras", "Heads", &para_id.encode());

    let last_header = state
        .storage_synchronizer
        .sync_parachain_header(input.headers, input.proof, &storage_key)
        .map_err(display)?;

    Ok(json!({ "synced_to": last_header }))
}

fn dispatch_block(input: blocks::DispatchBlockReq) -> Result<Value, Value> {
    info!(
        "dispatch_block from={:?} to={:?}",
        input.blocks.first().map(|h| h.block_header.number),
        input.blocks.last().map(|h| h.block_header.number)
    );

    let mut state = STATE.lock().unwrap();
    let state = state.as_mut().ok_or(error_msg("Runtime not initialized"))?;

    // TODO.kevin: enable e2e encryption mq for contracts
    // let _ecdh_privkey = local_state.ecdh_key.as_ref().unwrap().clone();
    let mut last_block = 0;
    for block in input.blocks.into_iter() {
        state
            .storage_synchronizer
            .feed_block(&block, &mut state.chain_storage)
            .map_err(display)?;

        state.purge_mq();
        handle_inbound_messages(block.block_header.number, state)?;
        last_block = block.block_header.number;
    }

    Ok(json!({ "dispatched_to": last_block }))
}

fn handle_inbound_messages(
    block_number: chain::BlockNumber,
    state: &mut RuntimeState,
) -> Result<(), Value> {
    // Dispatch events
    let messages = state
        .chain_storage
        .mq_messages()
        .or(Err(error_msg("Can not get mq messages from storage")))?;

    let system = &mut SYSTEM_STATE.lock().unwrap();
    let system = system
        .as_mut()
        .ok_or_else(|| error_msg("Runtime not initialized"))?;

    state.recv_mq.reset_local_index();

    for message in messages {
        use phala_types::messaging::SystemEvent;
        macro_rules! log_message {
            ($msg: expr, $t: ident) => {{
                let event: Result<$t, _> =
                    parity_scale_codec::Decode::decode(&mut &$msg.payload[..]);
                match event {
                    Ok(event) => {
                        info!(
                            "mq dispatching message: sender={:?} dest={:?} payload={:?}",
                            $msg.sender, $msg.destination, event
                        );
                    }
                    Err(_) => {
                        info!("mq dispatching message (decode failed): {:?}", $msg);
                    }
                }
            }};
        }
        match &message.destination.path()[..] {
            SystemEvent::TOPIC => {
                log_message!(message, SystemEvent);
            }
            _ => {
                info!("mq dispatching message: {:?}", message);
            }
        }
        state.recv_mq.dispatch(message);
    }

    let _guard = scopeguard::guard(&mut state.recv_mq, |mq| {
        let n_unhandled = mq.clear();
        if n_unhandled > 0 {
            warn!("There are {} unhandled messages dropped", n_unhandled);
        }
    });

    let now_ms = state
        .chain_storage
        .timestamp_now()
        .ok_or(error_msg("No timestamp found in block"))?;

    let storage = &state.chain_storage;
    let block = BlockInfo {
        block_number,
        now_ms,
        storage,
    };

    if let Err(e) = system.process_messages(&block) {
        error!("System process events failed: {:?}", e);
        return Err(error_msg("System process events failed"));
    }

    let mut env = ExecuteEnv {
        block: &block,
        system,
    };

    for contract in state.contracts.values_mut() {
        contract.process_messages(&mut env);
    }

    Ok(())
}

fn get_info(_input: &Map<String, Value>) -> Result<Value, Value> {
    let local_state = LOCAL_STATE.lock().unwrap();

    let initialized = local_state.initialized;
    let pubkey = local_state
        .identity_key
        .as_ref()
        .map(|pair| hex::encode(&pair.public()));
    let s_ecdh_pk = match &local_state.ecdh_key {
        Some(ecdh_key) => hex::encode(ecdh_key.public().as_ref()),
        None => "".to_string(),
    };
    let machine_id = local_state.machine_id;
    let dev_mode = local_state.dev_mode;
    drop(local_state);

    let state = STATE.lock().unwrap();
    let (state_root, pending_messages, counters) = match state.as_ref() {
        Some(state) => {
            let state_root = hex::encode(state.chain_storage.root());
            let pending_messages = state.send_mq.count_messages();
            let counters = state.storage_synchronizer.counters();
            (state_root, pending_messages, counters)
        }
        None => Default::default(),
    };
    drop(state);

    let (registered, role) = {
        match SYSTEM_STATE.lock().unwrap().as_ref() {
            Some(system) => (system.is_registered(), system.gatekeeper_role()),
            None => (false, GatekeeperRole::None),
        }
    };
    let score = benchmark::score();

    Ok(json!({
        "initialized": initialized,
        "registered": registered,
        "gatekeeper_role": role,
        "public_key": pubkey,
        "ecdh_public_key": s_ecdh_pk,
        "headernum": counters.next_header_number,
        "para_headernum": counters.next_para_header_number,
        "blocknum": counters.next_block_number,
        "state_root": state_root,
        "machine_id": machine_id,
        "dev_mode": dev_mode,
        "pending_messages": pending_messages,
        "score": score,
    }))
}

fn get_runtime_info(_input: &Map<String, Value>) -> Result<Value, Value> {
    let local_state = LOCAL_STATE.lock().unwrap();
    let resp = local_state
        .runtime_info
        .as_ref()
        .ok_or_else(|| error_msg("Uninitiated runtime info"))?;
    Ok(serde_json::to_value(resp).unwrap())
}

fn test_ink(_input: &Map<String, Value>) -> Result<Value, Value> {
    info!("=======Begin Ink Contract Test=======");

    let mut testcases = Vec::new();
    testcases.push(TestContract {
        name: String::from("flipper"),
        code: include_bytes!("res/flipper.wasm").to_vec(),
        initial_data: vec![248, 30, 126, 26, 0],
        txs: vec![
            vec![205, 228, 239, 169], // flip()
            vec![109, 76, 230, 60],   // get()
        ],
    });
    testcases.push(TestContract {
        name: String::from("EIP20Token"),
        code: include_bytes!("res/EIP20Token.wasm").to_vec(),
        initial_data: vec![134, 23, 49, 213],
        txs: vec![
            vec![
                102, 136, 227, 5, 128, 150, 152, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 36, 84, 101, 115, 116, 84, 111, 107, 101, 110,
                2, 8, 84, 84,
            ], // eip20 (initialAmount: u256, tokenName: String, decimalUnits: u8, tokenSymbol: String)
            vec![
                106, 70, 115, 148, 142, 175, 4, 21, 22, 135, 115, 99, 38, 201, 254, 161, 126, 37,
                252, 82, 135, 97, 54, 147, 201, 18, 144, 156, 178, 38, 170, 71, 148, 242, 106, 72,
                210, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0,
            ], // transfer (to: AccountId, value: u256)
        ],
    });

    for t in testcases {
        let mut driver = InkModule::new();

        info!("\n>>> Execute Contract {}", t.name);

        let contract_key = driver.put_code(t.code).unwrap();
        info!(">>> Code deplyed to {}", contract_key);

        let result = InkModule::instantiate(contract_key, t.initial_data);
        info!(">>> Code instantiated with result {:?}", result.unwrap());

        for tx in t.txs {
            let result = InkModule::call(contract_key, tx);
            info!(">>> Code called with result {:?}", result.unwrap());
        }
    }

    Ok(json!({}))
}

fn get_egress_messages() -> Result<Value, Value> {
    let guard = STATE.lock().unwrap();
    let messages: Vec<_> = guard
        .as_ref()
        .map(|state| state.send_mq.all_messages_grouped().into_iter().collect())
        .unwrap_or(Default::default());
    let bin_messages = Encode::encode(&messages);
    let b64_messages = base64::encode(bin_messages);
    Ok(json!({
        "messages": b64_messages,
    }))
}

fn query(q: types::SignedQuery) -> Result<Value, Value> {
    let payload_data = q.query_payload.as_bytes();
    // Validate signature
    if let Some(origin) = &q.origin {
        if !origin
            .verify(payload_data)
            .map_err(|_| error_msg("Bad signature or origin"))?
        {
            return Err(error_msg("Verifying signature failed"));
        }
        info!("Verifying signature passed!");
    }
    // Load and decrypt if necessary
    let payload: types::Payload =
        serde_json::from_slice(payload_data).map_err(|_| error_msg("Failed to decode payload"))?;
    let msg = {
        match payload {
            types::Payload::Plain(data) => data.into_bytes(),
            types::Payload::Cipher(cipher) => todo!("not supported"),
        }
    };
    debug!("msg: {}", String::from_utf8_lossy(&msg));
    let opaque_query: types::OpaqueQuery =
        serde_json::from_slice(&msg).map_err(|_| error_msg("Malformed request (Query)"))?;
    // Origin
    let accid_origin = match q.origin.as_ref() {
        Some(o) => {
            let accid =
                contracts::account_id_from_hex(&o.origin).map_err(|_| error_msg("Bad origin"))?;
            Some(accid)
        }
        None => None,
    };
    // Dispatch
    let ref_origin = accid_origin.as_ref();
    let res = match opaque_query.contract_id {
        SYSTEM => {
            let mut guard = SYSTEM_STATE.lock().unwrap();
            let system_state = guard
                .as_mut()
                .ok_or_else(|| error_msg("Runtime not initialized"))?;
            serde_json::to_value(
                system_state.handle_query(
                    ref_origin,
                    types::deopaque_query(opaque_query)
                        .map_err(|_| error_msg("Malformed request (system::Request)"))?
                        .request,
                ),
            )
            .unwrap()
        }
        _ => {
            let mut state = STATE.lock().unwrap();
            let state = state.as_mut().ok_or(error_msg("Runtime not initialized"))?;
            let contract = state
                .contracts
                .get_mut(&opaque_query.contract_id)
                .ok_or(error_msg("Contract not found"))?;
            let response = contract.handle_query(ref_origin, opaque_query)?;
            response
        }
    };
    // Encrypt response if necessary
    let res_json = res.to_string();
    let res_payload = types::Payload::Plain(res_json);

    let res_value = serde_json::to_value(res_payload).unwrap();
    Ok(res_value)
}

fn test(param: TestReq) -> Result<Value, Value> {
    Ok(json!({}))
}

mod identity {
    use super::*;
    type WorkerPublicKey = sp_core::ecdsa::Public;

    pub fn is_gatekeeper(pubkey: &WorkerPublicKey, chain_storage: &Storage) -> bool {
        let key = storage_prefix("PhalaRegistry", "Gatekeeper");
        let gatekeepers = chain_storage
            .get(&key)
            .map(|v| {
                Vec::<WorkerPublicKey>::decode(&mut &v[..])
                    .expect("Decode value of Gatekeeper Failed. (This should not happen)")
            })
            .unwrap_or(Vec::new());

        gatekeepers.contains(pubkey)
    }
}

#[cfg(feature = "tests")]
fn run_all_tests() {
    system::run_all_tests();
    info!("🎉🎉🎉🎉 All Tests Passed. 🎉🎉🎉🎉");
}

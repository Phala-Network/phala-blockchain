#![warn(unused_imports)]
#![warn(unused_extern_crates)]
#![feature(bench_black_box)]

#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

extern crate runtime as chain;

use std;

use rand::*;
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
use crate::std::sync::Mutex as SgxMutex; // TODO.kevin: take back SgxMutex?
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
use sp_core::{crypto::Pair, sr25519, H256};

use http_req::{uri::Uri, request::{Method, Request}};
use std::time::Duration;

// use pink::InkModule;

use enclave_api::prpc::InitRuntimeResponse;
use enclave_api::storage_sync::{
    ParachainSynchronizer, SolochainSynchronizer, StorageSynchronizer,
};
use enclave_api::{
    actions::*,
    blocks::{self, SyncParachainHeaderReq},
};

use phala_crypto::{
    aead,
    ecdh::EcdhKey,
    sr25519::{Persistence, Sr25519SecretKey, KDF, SEED_BYTES},
};
use phala_mq::{BindTopic, MessageDispatcher, MessageOrigin, MessageSendQueue};
use phala_pallets::pallet_mq;
use phala_types::{MasterPublicKey, WorkerPublicKey, WorkerRegistrationInfo};
use std::convert::TryFrom;

mod benchmark;
mod cert;
mod contracts;
mod cryptography;
mod light_validation;
mod msg_channel;
mod prpc_service;
mod rpc_types;
mod storage;
mod system;
mod types;
mod utils;
mod libc_hacks;

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
    genesis_block_hash: Option<H256>,
    identity_key: Option<sr25519::Pair>,
    ecdh_key: Option<EcdhKey>,
    machine_id: [u8; 16],
    dev_mode: bool,
    runtime_info: Option<InitRuntimeResponse>,
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
            genesis_block_hash: None,
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
    let url = format!("https://{}{}/{:08x}", IAS_HOST, IAS_SIGRL_ENDPOINT, gid);
    let url = Uri::try_from(url.as_str())
        .expect("Invalid IAS URI");
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

    let url = format!("https://{}{}", IAS_HOST, IAS_REPORT_ENDPOINT);
    let url = Uri::try_from(url.as_str())
        .expect("Invalid IAS URI");
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
    // TODO.kevin: is it OK to replace with thread_rng?
    // let mut os_rng = os::SgxRng::new().unwrap();
    let mut os_rng = rand::thread_rng();
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
        handle_json_api(action, input_slice, output_buf_len)
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

fn handle_json_api(action: u8, input: &[u8], output_buf_len: usize) -> Result<Value, Value> {
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
                ACTION_GET_INFO => get_info_json(),
                ACTION_DUMP_STATES => dump_states(payload),
                ACTION_LOAD_STATES => load_states(payload),
                ACTION_GET_RUNTIME_INFO => get_runtime_info(payload),
                ACTION_TEST_INK => test_ink(payload),
                ACTION_GET_EGRESS_MESSAGES => get_egress_messages(output_buf_len),
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
    genesis_block_hash: String,
    sk: String,
    dev_mode: bool,
}

fn save_secret_keys(
    genesis_block_hash: H256,
    sr25519_sk: sr25519::Pair,
    dev_mode: bool,
) -> Result<PersistentRuntimeData> {
    // Put in PresistentRuntimeData
    let serialized_sk = sr25519_sk.dump_secret_key();

    let data = PersistentRuntimeData {
        version: 1,
        genesis_block_hash: hex::encode(genesis_block_hash.as_ref()),
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

fn new_sr25519_key() -> sr25519::Pair {
    use rand::RngCore;
    let mut rng = rand::thread_rng();
    let mut seed = [0_u8; SEED_BYTES];
    rng.fill_bytes(&mut seed);
    sr25519::Pair::from_seed(&seed)
}

fn generate_random_iv() -> aead::IV {
    let mut nonce_vec = [0u8; aead::IV_BYTES];
    let rand = ring::rand::SystemRandom::new();
    rand.fill(&mut nonce_vec).unwrap();
    nonce_vec
}

fn generate_random_info() -> [u8; 32] {
    let mut nonce_vec = [0u8; 32];
    let rand = ring::rand::SystemRandom::new();
    rand.fill(&mut nonce_vec).unwrap();
    nonce_vec
}

fn init_secret_keys(
    local_state: &mut LocalState,
    genesis_block_hash: H256,
    predefined_identity_key: Option<sr25519::Pair>,
) -> Result<PersistentRuntimeData> {
    let data = if let Some(sr25519_sk) = predefined_identity_key {
        save_secret_keys(genesis_block_hash, sr25519_sk, true)?
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
                let sr25519_sk = new_sr25519_key();
                save_secret_keys(genesis_block_hash, sr25519_sk, false)?
            }
            other_err => return other_err,
        }
    };

    // check genesis block hash
    let saved_genesis_block_hash: [u8; 32] = hex::decode(&data.genesis_block_hash)
        .expect("Unable to decode genesis block hash hex")
        .as_slice()
        .try_into()
        .expect("slice with incorrect length");
    let saved_genesis_block_hash = H256::from(saved_genesis_block_hash);
    if genesis_block_hash != saved_genesis_block_hash {
        panic!(
            "Genesis block hash mismatches with saved keys, expected {}",
            saved_genesis_block_hash
        );
    }

    // load identity
    let sr25519_raw_key: Sr25519SecretKey = hex::decode(&data.sk)
        .expect("Unable to decode identity key hex")
        .as_slice()
        .try_into()
        .expect("slice with incorrect length");

    let sr25519_sk = sr25519::Pair::restore_from_secret_key(&sr25519_raw_key);
    info!("Identity pubkey: {:?}", hex::encode(&sr25519_sk.public()));

    // derive ecdh key
    let ecdh_key = sr25519_sk
        .derive_ecdh_key()
        .expect("Unable to derive ecdh key");
    let ecdh_hex_pk = hex::encode(ecdh_key.public().as_ref());
    info!("ECDH pubkey: {:?}", ecdh_hex_pk);

    // Generate Seal Key as Machine Id
    // This SHOULD be stable on the same CPU
    let machine_id = generate_seal_key();
    info!("Machine id: {:?}", hex::encode(&machine_id));

    // Save
    local_state.genesis_block_hash = Some(genesis_block_hash);
    local_state.identity_key = Some(sr25519_sk);
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

    sgx_status_t::SGX_SUCCESS
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
    // load chain genesis
    let raw_genesis =
        base64::decode(&input.bridge_genesis_info_b64).expect("Bad bridge_genesis_info_b64");
    let genesis =
        light_validation::BridgeInitInfo::<chain::Runtime>::decode(&mut raw_genesis.as_slice())
            .expect("Can't decode bridge_genesis_info_b64");

    // load identity
    let debug_set_key = if let Some(key) = input.debug_set_key {
        Some(hex::decode(&key).map_err(|_| error_msg("Can't decode key hex"))?)
    } else {
        None
    };

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

    let genesis_state_scl = base64::decode(input.genesis_state_b64)
        .map_err(|_| error_msg("Base64 decode genesis state failed"))?;
    let mut genesis_state_scl = &genesis_state_scl[..];
    let genesis_state: Vec<(Vec<u8>, Vec<u8>)> = Decode::decode(&mut genesis_state_scl)
        .map_err(|_| error_msg("Scale decode genesis state failed"))?;

    let genesis = blocks::GenesisBlockInfo {
        block_header: genesis.block_header,
        validator_set: genesis.validator_set,
        validator_set_proof: genesis.validator_set_proof,
    };

    let resp = prpc_service::init_runtime(
        input.skip_ra,
        input.is_parachain,
        genesis,
        genesis_state,
        operator,
        debug_set_key,
    )
    .map_err(display)?;
    let resp = convert_runtime_info(resp)?;
    Ok(serde_json::to_value(resp).unwrap())
}

fn sync_header(input: blocks::SyncHeaderReq) -> Result<Value, Value> {
    let resp =
        prpc_service::sync_header(input.headers, input.authority_set_change).map_err(display)?;
    Ok(json!({ "synced_to": resp.synced_to }))
}

fn sync_para_header(input: SyncParachainHeaderReq) -> Result<Value, Value> {
    let resp = prpc_service::sync_para_header(input.headers, input.proof).map_err(display)?;
    Ok(json!({ "synced_to": resp.synced_to }))
}

fn dispatch_block(input: blocks::DispatchBlockReq) -> Result<Value, Value> {
    let resp = prpc_service::dispatch_block(input.blocks).map_err(display)?;
    Ok(json!({ "dispatched_to": resp.synced_to }))
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

fn get_info_json() -> Result<Value, Value> {
    let info = prpc_service::get_info();
    let machin_id = LOCAL_STATE.lock().unwrap().machine_id;
    Ok(json!({
        "initialized": info.initialized,
        "registered": info.registered,
        "gatekeeper_role": info.gatekeeper_role,
        "genesis_block_hash": info.genesis_block_hash,
        "public_key": info.public_key,
        "ecdh_public_key": info.ecdh_public_key,
        "headernum": info.headernum,
        "para_headernum": info.para_headernum,
        "blocknum": info.blocknum,
        "state_root": info.state_root,
        "dev_mode": info.dev_mode,
        "pending_messages": info.pending_messages,
        "score": info.score,
        "machine_id": machin_id,
    }))
}

fn convert_runtime_info(info: InitRuntimeResponse) -> Result<InitRuntimeResp, Value> {
    let genesis_block_hash = info.genesis_block_hash_decoded().map_err(display)?;
    let public_key = info.public_key_decoded().map_err(display)?;
    let ecdh_public_key = info.ecdh_public_key_decoded().map_err(display)?;

    let genesis_block_hash_hex = hex::encode(genesis_block_hash);
    let ecdsa_hex_pk = hex::encode(public_key);
    let ecdh_hex_pk = hex::encode(ecdh_public_key.0);

    let attestation = info.attestation.map(|att| {
        let payload = att.payload.expect("BUG: Payload must exist");
        InitRespAttestation {
            version: att.version,
            provider: att.provider,
            payload: AttestationReport {
                report: payload.report,
                signature: base64::encode(&payload.signature),
                signing_cert: base64::encode_config(&payload.signing_cert, base64::STANDARD),
            },
        }
    });

    Ok(InitRuntimeResp {
        encoded_runtime_info: info.runtime_info,
        genesis_block_hash: genesis_block_hash_hex,
        public_key: ecdsa_hex_pk,
        ecdh_public_key: ecdh_hex_pk,
        attestation,
    })
}

fn get_runtime_info(_input: &Map<String, Value>) -> Result<Value, Value> {
    let resp = prpc_service::get_runtime_info().map_err(display)?;
    Ok(serde_json::to_value(convert_runtime_info(resp)?).unwrap())
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
        // TODO.kevin:
        // let mut driver = InkModule::new();

        // info!("\n>>> Execute Contract {}", t.name);

        // let contract_key = driver.put_code(t.code).unwrap();
        // info!(">>> Code deplyed to {}", contract_key);

        // let result = InkModule::instantiate(contract_key, t.initial_data);
        // info!(">>> Code instantiated with result {:?}", result.unwrap());

        // for tx in t.txs {
        //     let result = InkModule::call(contract_key, tx);
        //     info!(">>> Code called with result {:?}", result.unwrap());
        // }
    }

    Ok(json!({}))
}

fn get_egress_messages(output_buf_len: usize) -> Result<Value, Value> {
    let messages =
        prpc_service::get_egress_messages(output_buf_len * 3 / 4 - 1024).map_err(display)?;
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
            types::Payload::Cipher(_cipher) => todo!("not supported"),
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

fn test(_param: TestReq) -> Result<Value, Value> {
    Ok(json!({}))
}

mod gatekeeper {
    use super::*;

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

    #[allow(dead_code)]
    pub fn read_master_pubkey(chain_storage: &Storage) -> Option<MasterPublicKey> {
        let key = storage_prefix("PhalaRegistry", "GatekeeperMasterPubkey");
        chain_storage
            .get(&key)
            .map(|v| {
                Some(
                    MasterPublicKey::decode(&mut &v[..])
                        .expect("Decode value of MasterPubkey Failed. (This should not happen)"),
                )
            })
            .unwrap_or(None)
    }
}

#[cfg(feature = "tests")]
fn run_all_tests() {
    system::run_all_tests();
    info!("🎉🎉🎉🎉 All Tests Passed. 🎉🎉🎉🎉");
}

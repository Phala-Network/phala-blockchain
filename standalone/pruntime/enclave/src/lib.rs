#![crate_name = "enclaveapp"]
#![crate_type = "staticlib"]
#![warn(unused_imports)]
#![warn(unused_extern_crates)]
#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

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

use crate::std::prelude::v1::*;
use crate::std::ptr;
use crate::std::str;
use crate::std::string::String;
use crate::std::sync::SgxMutex;
use crate::std::vec::Vec;
use anyhow::Result;
use core::convert::TryInto;
use frame_system::EventRecord;
use itertools::Itertools;
use log::{debug, error, info, warn};
use parity_scale_codec::{Decode, Encode, FullCodec};
use secp256k1::{PublicKey, SecretKey};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde_cbor;
use serde_json::{Map, Value};
use sp_core::crypto::Pair;
use sp_core::H256 as Hash;

use http_req::request::{Method, Request};
use std::time::Duration;

use pink::InkModule;

extern crate pallet_phala as phala;
use phala_types::{
    pruntime::{
        BlockHeaderWithEvents as GenericBlockHeaderWithEvents, HeaderToSync as GenericHeaderToSync,
        StorageKV,
    },
    PRuntimeInfo,
};

mod cert;
mod contracts;
mod cryptography;
mod hex;
mod light_validation;
mod msg_channel;
mod rpc_types;
mod system;
mod types;

use contracts::{
    AccountIdWrapper, Contract, ContractId, ASSETS, BALANCES, DATA_PLAZA, DIEM, SYSTEM,
    WEB3_ANALYTICS, SUBSTRATE_KITTIES, BTC_LOTTERY
};
use cryptography::{aead, ecdh};
use light_validation::AuthoritySetChange;
use rpc_types::*;
use system::{CommandIndex, TransactionReceipt, TransactionStatus};
use types::{Error, TxRef};

type HeaderToSync =
    GenericHeaderToSync<chain::BlockNumber, <chain::Runtime as frame_system::Config>::Hashing>;
type BlockHeaderWithEvents = GenericBlockHeaderWithEvents<
    chain::BlockNumber,
    <chain::Runtime as frame_system::Config>::Hashing,
    chain::Balance,
>;
type OnlineWorkerSnapshot =
    phala_types::pruntime::OnlineWorkerSnapshot<chain::BlockNumber, chain::Balance>;

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

type ChainLightValidation = light_validation::LightValidation<chain::Runtime>;
type EcdhKey = ring::agreement::EphemeralPrivateKey;

#[derive(Serialize, Deserialize, Debug)]
struct RuntimeState {
    contract1: contracts::data_plaza::DataPlaza,
    contract2: contracts::balances::Balances,
    contract3: contracts::assets::Assets,
    contract4: contracts::web3analytics::Web3Analytics,
    contract5: contracts::diem::Diem,
    contract6: contracts::substrate_kitties::SubstrateKitties,
    contract7: contracts::btc_lottery::BtcLottery,
    #[serde(serialize_with = "se_to_b64", deserialize_with = "de_from_b64")]
    light_client: ChainLightValidation,
    main_bridge: u64,
}

struct LocalState {
    initialized: bool,
    public_key: Box<PublicKey>,
    private_key: Box<SecretKey>,
    headernum: u32, // the height of synced block
    blocknum: u32,  // the height of dispatched block
    block_hashes: Vec<Hash>,
    ecdh_private_key: Option<EcdhKey>,
    ecdh_public_key: Option<ring::agreement::PublicKey>,
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

fn se_to_b64<S>(value: &ChainLightValidation, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let data = value.encode();
    let s = base64::encode(data.as_slice());
    String::serialize(&s, serializer)
}

fn de_from_b64<'de, D>(deserializer: D) -> Result<ChainLightValidation, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let data = base64::decode(&s).map_err(de::Error::custom)?;
    ChainLightValidation::decode(&mut data.as_slice()).map_err(|_| de::Error::custom("bad data"))
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
    static ref STATE: SgxMutex<RuntimeState> = {
        SgxMutex::new(RuntimeState {
            contract1: contracts::data_plaza::DataPlaza::new(),
            contract2: contracts::balances::Balances::new(None),
            contract3: contracts::assets::Assets::new(),
            contract4: contracts::web3analytics::Web3Analytics::new(),
            contract5: contracts::diem::Diem::new(),
            contract6: contracts::substrate_kitties::SubstrateKitties::new(None),
            contract7: contracts::btc_lottery::BtcLottery::new(None),
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
                headernum: 0,
                blocknum: 0,
                block_hashes: Vec::new(),
                ecdh_private_key: None,
                ecdh_public_key: None,
                machine_id: [0; 16],
                dev_mode: false,
                runtime_info: None,
            }
        )
    };

    static ref SYSTEM_STATE: SgxMutex<system::System> = {
        SgxMutex::new(system::System::new())
    };
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

    hex::decode_spid(&key_str[..key_len])
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

const ACTION_TEST: u8 = 0;
const ACTION_INIT_RUNTIME: u8 = 1;
const ACTION_GET_INFO: u8 = 2;
const ACTION_DUMP_STATES: u8 = 3;
const ACTION_LOAD_STATES: u8 = 4;
const ACTION_SYNC_HEADER: u8 = 5;
const ACTION_QUERY: u8 = 6;
const ACTION_DISPATCH_BLOCK: u8 = 7;
// Reserved: 8, 9
const ACTION_GET_RUNTIME_INFO: u8 = 10;
const ACTION_SET: u8 = 21;
const ACTION_GET: u8 = 22;
const ACTION_TEST_INK: u8 = 100;

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
    let input: serde_json::value::Value = serde_json::from_slice(input_slice).unwrap();
    let input_value = input.get("input").unwrap().clone();

    // Strong typed
    fn load_param<T: de::DeserializeOwned>(input_value: serde_json::value::Value) -> T {
        serde_json::from_value(input_value).unwrap()
    }

    let result = match action {
        ACTION_INIT_RUNTIME => init_runtime(load_param(input_value)),
        ACTION_TEST => test(load_param(input_value)),
        ACTION_QUERY => query(load_param(input_value)),
        ACTION_SYNC_HEADER => sync_header(load_param(input_value)),
        ACTION_DISPATCH_BLOCK => dispatch_block(load_param(input_value)),
        _ => {
            let payload = input_value.as_object().unwrap();
            match action {
                ACTION_GET_INFO => get_info(payload),
                ACTION_DUMP_STATES => dump_states(payload),
                ACTION_LOAD_STATES => load_states(payload),
                ACTION_GET => get(payload),
                ACTION_SET => set(payload),
                ACTION_GET_RUNTIME_INFO => get_runtime_info(payload),
                ACTION_TEST_INK => test_ink(payload),
                _ => unknown(),
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
        }
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
    info!("{}", output_json.to_string());

    let output_json_vec = serde_json::to_vec(&output_json).unwrap();
    let output_json_vec_len = output_json_vec.len();
    let output_json_vec_len_ptr = &output_json_vec_len as *const usize;

    unsafe {
        if output_json_vec_len <= output_buf_len {
            ptr::copy_nonoverlapping(output_json_vec.as_ptr(), output_ptr, output_json_vec_len);
        } else {
            warn!("Too much output. Buffer overflow.");
        }
        ptr::copy_nonoverlapping(
            output_json_vec_len_ptr,
            output_len_ptr,
            std::mem::size_of_val(&output_json_vec_len),
        );
    }

    sgx_status_t::SGX_SUCCESS
}

const SEAL_DATA_BUF_MAX_LEN: usize = 2048 as usize;

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
struct PersistentRuntimeData {
    version: u32,
    sk: String,
    ecdh_sk: String,
    dev_mode: bool,
}

fn save_secret_keys(
    ecdsa_sk: SecretKey,
    ecdh_sk: EcdhKey,
    dev_mode: bool,
) -> Result<PersistentRuntimeData> {
    // Put in PresistentRuntimeData
    let serialized_sk = ecdsa_sk.serialize();
    let serialized_ecdh_sk = ecdh::dump_key(&ecdh_sk);

    let data = PersistentRuntimeData {
        version: 1,
        sk: hex::encode_hex_compact(serialized_sk.as_ref()),
        ecdh_sk: hex::encode_hex_compact(serialized_ecdh_sk.as_ref()),
        dev_mode,
    };
    let encoded_vec = serde_cbor::to_vec(&data).unwrap();
    let encoded_slice = encoded_vec.as_slice();
    info!("Length of encoded slice: {}", encoded_slice.len());
    info!(
        "Encoded slice: {:?}",
        hex::encode_hex_compact(encoded_slice)
    );

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
    info!(
        "Encoded slice: {:?}",
        hex::encode_hex_compact(encoded_slice)
    );

    serde_cbor::from_slice(encoded_slice).map_err(|_| anyhow::Error::msg(Error::DecodeError))
}

fn init_secret_keys(
    local_state: &mut LocalState,
    predefined_keys: Option<(SecretKey, EcdhKey)>,
) -> Result<PersistentRuntimeData> {
    let data = if let Some((ecdsa_sk, ecdh_sk)) = predefined_keys {
        save_secret_keys(ecdsa_sk, ecdh_sk, true)?
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
                let ecdsa_sk = SecretKey::random(&mut rand::thread_rng());
                let ecdh_sk = ecdh::generate_key();
                save_secret_keys(ecdsa_sk, ecdh_sk, false)?
            }
            other_err => return other_err,
        }
    };

    // load identity
    let ecdsa_raw_key: [u8; 32] = hex::decode_hex(&data.sk)
        .as_slice()
        .try_into()
        .expect("slice with incorrect length");
    let ecdsa_sk = SecretKey::parse(&ecdsa_raw_key).expect("can't parse private key");
    let ecdsa_pk = PublicKey::from_secret_key(&ecdsa_sk);
    let ecdsa_serialized_pk = ecdsa_pk.serialize_compressed();
    let ecdsa_hex_pk = hex::encode_hex_compact(ecdsa_serialized_pk.as_ref());
    info!("Identity pubkey: {:?}", ecdsa_hex_pk);

    // load ECDH identity
    let ecdh_raw_key = hex::decode_hex(&data.ecdh_sk);
    let ecdh_sk = ecdh::create_key(ecdh_raw_key.as_slice()).expect("can't create ecdh key");
    let ecdh_pk = ecdh_sk.compute_public_key().expect("can't compute pubkey");
    let ecdh_hex_pk = hex::encode_hex_compact(ecdh_pk.as_ref());
    info!("ECDH pubkey: {:?}", ecdh_hex_pk);

    // Generate Seal Key as Machine Id
    // This SHOULD be stable on the same CPU
    let machine_id = generate_seal_key();
    info!("Machine id: {:?}", hex::encode_hex_compact(&machine_id));

    // Save
    *local_state.public_key = ecdsa_pk.clone();
    *local_state.private_key = ecdsa_sk.clone();
    local_state.ecdh_private_key = Some(ecdh_sk);
    local_state.ecdh_public_key = Some(ecdh_pk);
    local_state.machine_id = machine_id.clone();
    local_state.dev_mode = data.dev_mode;

    info!("Init done.");
    Ok(data)
}

#[no_mangle]
pub extern "C" fn ecall_init() -> sgx_status_t {
    let mut local_state = LOCAL_STATE.lock().unwrap();
    match init_secret_keys(&mut local_state, None) {
        Err(e) if e.is::<sgx_status_t>() => e.downcast::<sgx_status_t>().unwrap(),
        _ => sgx_status_t::SGX_SUCCESS,
    }
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

const SECRET: &[u8; 32] = b"24e3e78e1f15150cdbad02f3205f6dd0";

fn dump_states(_input: &Map<String, Value>) -> Result<Value, Value> {
    let sessions = STATE.lock().unwrap();
    let serialized = serde_json::to_string(&*sessions).unwrap();

    // Your private data
    let content = serialized.as_bytes().to_vec();
    info!("Content to encrypt's size {}", content.len());
    debug!("{}", serialized);

    // Ring uses the same input variable as output
    let mut in_out = content.clone();
    info!("in_out len {}", in_out.len());

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
    debug!("{}", input.get("data").unwrap().as_str().unwrap());

    let decrypted_data = aead::decrypt(&nonce_vec, &*SECRET, &mut in_out);
    debug!("{}", String::from_utf8(decrypted_data.to_vec()).unwrap());

    let deserialized: RuntimeState = serde_json::from_slice(decrypted_data).unwrap();

    debug!("{}", serde_json::to_string_pretty(&deserialized).unwrap());

    let mut sessions = STATE.lock().unwrap();
    let _ = std::mem::replace(&mut *sessions, deserialized);

    Ok(json!({}))
}

fn init_runtime(input: InitRuntimeReq) -> Result<Value, Value> {
    // TODO: Guard only initialize once
    let mut local_state = LOCAL_STATE.lock().unwrap();
    if local_state.initialized {
        return Err(json!({"message": "Already initialized"}));
    }

    env_logger::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // load identity
    if let Some(key) = input.debug_set_key {
        if input.skip_ra == false {
            return Err(error_msg("RA is disallowed when debug_set_key is enabled"));
        }
        let raw_key = hex::decode_hex(&key);
        let ecdsa_key = SecretKey::parse_slice(raw_key.as_slice())
            .map_err(|_| error_msg("can't parse private key"))?;
        let ecdh_key =
            ecdh::create_key(raw_key.as_slice()).map_err(|_| error_msg("can't create ecdh key"))?;
        init_secret_keys(&mut local_state, Some((ecdsa_key, ecdh_key)))
            .map_err(|_| error_msg("failed to update secret key"))?;
    }
    if !input.skip_ra && local_state.dev_mode {
        return Err(error_msg("RA is disallowed when debug_set_key is enabled"));
    }

    let ecdsa_pk = &local_state.public_key;
    let ecdsa_serialized_pk = ecdsa_pk.serialize_compressed();
    let ecdsa_hex_pk = hex::encode_hex_compact(ecdsa_serialized_pk.as_ref());
    info!("Identity pubkey: {:?}", ecdsa_hex_pk);

    // load ECDH identity
    let ecdh_pk = local_state.ecdh_public_key.as_ref().unwrap();
    let ecdh_hex_pk = hex::encode_hex_compact(ecdh_pk.as_ref());
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

    // Build PRuntimeInfo
    let runtime_info = PRuntimeInfo {
        version: VERSION,
        machine_id: local_state.machine_id.clone(),
        pubkey: ecdsa_serialized_pk,
        features: vec![cpu_core_num, cpu_feature_level],
    };
    let encoded_runtime_info = runtime_info.encode();
    let runtime_info_hash = sp_core::hashing::blake2_512(&encoded_runtime_info);

    info!("Encoded runtime info");
    info!("{:?}", hex::encode_hex_compact(&encoded_runtime_info));

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
    // Set up the bridge in local state
    let mut state = STATE.lock().unwrap();
    let bridge_id = state
        .light_client
        .initialize_bridge(
            genesis.block_header,
            genesis.validator_set,
            genesis.validator_set_proof,
        )
        .expect("Bridge initialize failed");
    state.main_bridge = bridge_id;
    let ecdsa_seed = local_state.private_key.serialize();
    let id_pair = sp_core::ecdsa::Pair::from_seed_slice(&ecdsa_seed)
        .expect("Unexpected ecdsa key error in init_runtime");
    // Re-init some contracts because they require the identity key
    let mut system_state = SYSTEM_STATE.lock().unwrap();
    system_state.set_id(&id_pair);
    system_state.set_machine_id(local_state.machine_id.to_vec());
    state.contract2 = contracts::balances::Balances::new(Some(id_pair.clone()));
    state.contract6 = contracts::substrate_kitties::SubstrateKitties::new(Some(id_pair.clone()));
    state.contract7 = contracts::btc_lottery::BtcLottery::new(Some(id_pair));
    // Initialize other states
    local_state.headernum = 1;
    local_state.blocknum = 1;
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

fn fmt_call(call: &chain::Call) -> String {
    match call {
        chain::Call::Timestamp(chain::TimestampCall::set(t)) => format!("Timestamp::set({})", t),
        chain::Call::Balances(chain::BalancesCall::transfer(to, amount)) => {
            format!("Balance::transfer({:?}, {:?})", to, amount)
        }
        _ => String::from("<Unparsed>"),
    }
}

fn print_block(signed_block: &chain::SignedBlock) {
    let header: &chain::Header = &signed_block.block.header;
    let extrinsics: &Vec<chain::UncheckedExtrinsic> = &signed_block.block.extrinsics;

    debug!("SignedBlock {{");
    debug!("  block {{");
    debug!("    header {{");
    debug!("      number: {}", header.number);
    debug!("      extrinsics_root: {}", header.extrinsics_root);
    debug!("      state_root: {}", header.state_root);
    debug!("      parent_hash: {}", header.parent_hash);
    debug!("      digest: logs[{}]", header.digest.logs.len());
    debug!("  extrinsics: [");
    for extrinsic in extrinsics {
        debug!("    UncheckedExtrinsic {{");
        debug!("      function: {}", fmt_call(&extrinsic.function));
        debug!("      signature: {:?}", extrinsic.signature);
        debug!("    }}");
    }
    debug!("  ]");
    debug!("  justification: <skipped...>");
    debug!("}}");
}

fn parse_block(data: &Vec<u8>) -> Result<chain::SignedBlock> {
    chain::SignedBlock::decode(&mut data.as_slice()).map_err(anyhow::Error::msg)
}

fn format_address(addr: &chain::Address) -> String {
    match addr {
        chain::Address::Id(id) => hex::encode_hex_compact(id.as_ref()),
        chain::Address::Index(index) => format!("index:{:?}", index),
        // TODO: Verify these
        chain::Address::Raw(address) => hex::encode_hex_compact(address.as_ref()),
        chain::Address::Address32(address) => hex::encode_hex_compact(address.as_ref()),
        chain::Address::Address20(address) => hex::encode_hex_compact(address.as_ref()),
    }
}

fn handle_execution(
    system: &mut system::System,
    state: &mut RuntimeState,
    pos: &TxRef,
    origin: chain::AccountId,
    contract_id: ContractId,
    payload: &Vec<u8>,
    command_index: CommandIndex,
    ecdh_privkey: &EcdhKey,
) {
    let payload: types::Payload =
        serde_json::from_slice(payload.as_slice()).expect("Failed to decode payload");
    let inner_data = match payload {
        types::Payload::Plain(data) => data.into_bytes(),
        types::Payload::Cipher(cipher) => {
            cryptography::decrypt(&cipher, ecdh_privkey)
                .expect("Decrypt failed")
                .msg
        }
    };

    let inner_data_string = String::from_utf8_lossy(&inner_data);
    info!("handle_execution: incominng cmd: {}", inner_data_string);

    info!("handle_execution: about to call handle_command");
    let status = match contract_id {
        DATA_PLAZA => match serde_json::from_slice(inner_data.as_slice()) {
            Ok(cmd) => state.contract1.handle_command(&origin, pos, cmd),
            _ => TransactionStatus::BadCommand,
        },
        BALANCES => match serde_json::from_slice(inner_data.as_slice()) {
            Ok(cmd) => state.contract2.handle_command(&origin, pos, cmd),
            _ => TransactionStatus::BadCommand,
        },
        ASSETS => match serde_json::from_slice(inner_data.as_slice()) {
            Ok(cmd) => state.contract3.handle_command(&origin, pos, cmd),
            _ => TransactionStatus::BadCommand,
        },
        WEB3_ANALYTICS => match serde_json::from_slice(inner_data.as_slice()) {
            Ok(cmd) => state.contract4.handle_command(&origin, pos, cmd),
            _ => TransactionStatus::BadCommand,
        },
        DIEM => match serde_json::from_slice(inner_data.as_slice()) {
            Ok(cmd) => state.contract5.handle_command(&origin, pos, cmd),
            _ => TransactionStatus::BadCommand,
        },
        SUBSTRATE_KITTIES => match serde_json::from_slice(inner_data.as_slice()) {
            Ok(cmd) => state.contract6.handle_command(&origin, pos,cmd),
            _ => TransactionStatus::BadCommand,
        },
        BTC_LOTTERY => match serde_json::from_slice(inner_data.as_slice()) {
            Ok(cmd) => state.contract7.handle_command(&origin, pos,cmd),
            _ => TransactionStatus::BadCommand,
        },
        _ => {
            warn!(
                "handle_execution: Skipped unknown contract: {}",
                contract_id
            );
            TransactionStatus::BadContractId
        }
    };

    system.add_receipt(
        command_index,
        TransactionReceipt {
            account: AccountIdWrapper(origin),
            block_num: pos.blocknum,
            contract_id,
            command: inner_data_string.to_string(),
            status,
        },
    );
}

fn sync_header(input: SyncHeaderReq) -> Result<Value, Value> {
    // Parse base64 to data
    let parsed_data: Result<Vec<_>, _> = (&input.headers_b64).iter().map(base64::decode).collect();
    let headers_data = parsed_data.map_err(|_| error_msg("Failed to parse base64 header"))?;
    // Parse data to headers
    let parsed_headers: Result<Vec<HeaderToSync>, _> = headers_data
        .iter()
        .map(|d| Decode::decode(&mut &d[..]))
        .collect();
    let headers = parsed_headers.map_err(|_| error_msg("Invalid header"))?;
    // Light validation when possible
    let last_header = headers
        .last()
        .ok_or_else(|| error_msg("No header in the request"))?;
    {
        // 1. the last header must has justification
        let justification = last_header
            .justification
            .as_ref()
            .ok_or_else(|| error_msg("Missing justification"))?
            .clone();
        let last_header = last_header.header.clone();
        // 2. check header sequence
        for (i, header) in headers.iter().enumerate() {
            if i > 0 && headers[i - 1].header.hash() != header.header.parent_hash {
                return Err(error_msg("Incorrect header order"));
            }
        }
        // 3. generate accenstor proof
        let mut accenstor_proof: Vec<_> = headers[0..headers.len() - 1]
            .iter()
            .map(|h| h.header.clone())
            .collect();
        accenstor_proof.reverse(); // from high to low
                                   // 4. submit to light client
        let mut state = STATE.lock().unwrap();
        let bridge_id = state.main_bridge;
        let authority_set_change = input
            .authority_set_change_b64
            .map(|b64| parse_authority_set_change(b64))
            .transpose()?;
        state
            .light_client
            .submit_finalized_headers(
                bridge_id,
                last_header,
                accenstor_proof,
                justification,
                authority_set_change,
            )
            .map_err(|e| error_msg(format!("Light validation failed {:?}", e).as_str()))?
    }
    // Passed the validation
    let mut local_state = LOCAL_STATE.lock().unwrap();
    let mut last_header = 0;
    for header_with_events in headers.iter() {
        let header = &header_with_events.header;
        if header.number != local_state.headernum {
            return Err(error_msg("Unexpected header"));
        }

        // move forward
        last_header = header.number;
        local_state.headernum = last_header + 1;
    }

    // Save the block hashes for future dispatch
    for header in headers.iter() {
        local_state.block_hashes.push(header.header.hash());
    }

    Ok(json!({ "synced_to": last_header }))
}

fn dispatch_block(input: DispatchBlockReq) -> Result<Value, Value> {
    // Parse base64 to data
    let parsed_data: Result<Vec<_>, _> = (&input.blocks_b64).iter().map(base64::decode).collect();
    let blocks_data = parsed_data.map_err(|_| error_msg("Failed to parse base64 block"))?;
    // Parse data to blocks
    let parsed_blocks: Result<Vec<BlockHeaderWithEvents>, _> = blocks_data
        .iter()
        .map(|d| Decode::decode(&mut &d[..]))
        .collect();
    let all_blocks = parsed_blocks.map_err(|_| error_msg("Invalid block"))?;

    let mut local_state = LOCAL_STATE.lock().unwrap();
    // Ignore processed blocks
    let blocks: Vec<_> = all_blocks
        .iter()
        .filter(|b| b.block_header.number >= local_state.blocknum)
        .collect();
    // Validate blocks
    let first_block = &blocks
        .first()
        .ok_or_else(|| error_msg("No block in the request"))?;
    let last_block = &blocks
        .last()
        .ok_or_else(|| error_msg("No block in the request"))?;
    if first_block.block_header.number != local_state.blocknum {
        return Err(error_msg("Unexpected block"));
    }
    if last_block.block_header.number >= local_state.headernum {
        return Err(error_msg("Unsynced block"));
    }
    for (i, block) in blocks.iter().enumerate() {
        let expected_hash = &local_state.block_hashes[i];
        if block.block_header.hash() != *expected_hash {
            return Err(error_msg("Unexpected block hash"));
        }
    }

    let ecdh_privkey = ecdh::clone_key(
        local_state
            .ecdh_private_key
            .as_ref()
            .expect("ECDH not initizlied"),
    );
    let mut last_block = 0;
    for block in blocks.iter() {
        if block.events.is_none() {
            return Err(error_msg("Event was required"));
        }

        handle_events(&block, &ecdh_privkey, local_state.dev_mode)?;

        last_block = block.block_header.number;
        local_state.block_hashes.remove(0);
        local_state.blocknum = last_block + 1;
    }

    Ok(json!({ "dispatched_to": last_block }))
}

fn parse_authority_set_change(data_b64: String) -> Result<AuthoritySetChange, Value> {
    let data = base64::decode(&data_b64)
        .map_err(|_| error_msg("cannot decode authority_set_change_b64"))?;
    AuthoritySetChange::decode(&mut &data[..])
        .map_err(|_| error_msg("cannot decode authority_set_change"))
}

fn handle_events(
    block_with_events: &BlockHeaderWithEvents,
    ecdh_privkey: &EcdhKey,
    dev_mode: bool,
) -> Result<(), Value> {
    let ref mut state = STATE.lock().unwrap();
    let missing_field = error_msg("Missing field");
    // Validate sotrage proof for events
    let events = block_with_events
        .events
        .as_ref()
        .ok_or(missing_field.clone())?;
    let proof = block_with_events
        .proof
        .as_ref()
        .ok_or(missing_field.clone())?;
    let event_storage_key = light_validation::utils::storage_prefix("System", "Events");
    let state_root = block_with_events.block_header.state_root;
    state
        .light_client
        .validate_storage_proof(
            state_root,
            proof.clone(),
            &[(event_storage_key.as_slice(), events.as_slice())],
        )
        .map_err(|_| error_msg("Bad storage proof for events"))?;
    // Validate worker snapshot (if applicable)
    if let Some(worker_snapshot) = block_with_events.worker_snapshot.as_ref() {
        if !validate_worker_snapshot(&state.light_client, state_root, worker_snapshot) {
            return Err(error_msg("Invalid worker_snapshot storage proof"));
        }
    }
    // Dispatch events
    let events = Vec::<EventRecord<chain::Event, Hash>>::decode(&mut events.as_slice())
        .map_err(|_| error_msg("Decode events error"))?;
    let system = &mut SYSTEM_STATE.lock().unwrap();
    let mut event_handler = system.feed_event();
    for evt in &events {
        if let chain::Event::pallet_phala(pe) = &evt.event {
            // Dispatch to system contract anyway
            event_handler
                .feed(block_with_events, &pe)
                .map_err(|e| error_msg(format!("Event error {:?}", e).as_str()))?;
            // Otherwise we only dispatch the events for dev_mode pRuntime (not miners)
            if !dev_mode {
                info!("handle_events: skipped for miners");
                continue;
            }
            match pe {
                phala::RawEvent::CommandPushed(who, contract_id, payload, num) => {
                    info!(
                        "push_command(contract_id: {}, payload: data[{}])",
                        contract_id,
                        payload.len()
                    );
                    let blocknum = block_with_events.block_header.number;
                    let pos = TxRef {
                        blocknum,
                        index: *num,
                    };
                    handle_execution(
                        event_handler.system,
                        state,
                        &pos,
                        who.clone(),
                        *contract_id,
                        payload,
                        *num,
                        ecdh_privkey,
                    );
                }
                _ => {
                    state.contract2.handle_event(evt.event.clone());
                }
            }
        } else if let chain::Event::pallet_kitties(pe) = &evt.event{
            println!("pallet_kitties event: {:?}", pe);
            state.contract6.handle_event(evt.event.clone());
        } else if let chain::Event::pallet_bridge_transfer(pe) = &evt.event{
            println!("pallet_bridge_transfer event: {:?}", pe);
            state.contract7.handle_event(evt.event.clone());
        }
    }
    Ok(())
}

fn validate_worker_snapshot(
    light_client: &ChainLightValidation,
    state_root: Hash,
    snapshot: &OnlineWorkerSnapshot,
) -> bool {
    info!("validate_worker_snapshot()");
    use light_validation::utils::storage_prefix;
    use phala_types::WorkerStateEnum;
    let prefix_onlineworkers = storage_prefix("Phala", "OnlineWorkers");
    let prefix_computeworkers = storage_prefix("Phala", "ComputeWorkers");
    let prefix_workerstate = storage_prefix("Phala", "WorkerState");
    let prefix_stakereceived = storage_prefix("MiningStaking", "StakeReceived");

    let cond = [
        // Check keys
        snapshot.online_workers_kv.key() == prefix_onlineworkers.as_slice(),
        snapshot.compute_workers_kv.key() == prefix_computeworkers.as_slice(),
        snapshot
            .worker_state_kv
            .iter()
            .all(|kv| kv.key().starts_with(prefix_workerstate.as_slice())),
        // WorkerState key is real
        snapshot
            .stake_received_kv
            .iter()
            .all(|kv| kv.key().starts_with(prefix_stakereceived.as_slice())),
        // There's no missing entry in WorkerState
        snapshot.online_workers_kv.value() == &(snapshot.worker_state_kv.len() as u32),
        // Compute worker is enabled
        snapshot.compute_workers_kv.value() != &0,
        // All the workers are online
        snapshot
            .worker_state_kv
            .iter()
            .all(|kv| match kv.value().state {
                WorkerStateEnum::Mining(_) => true,
                _ => false,
            }),
    ];
    if !cond.iter().all(|x| *x) {
        info!("Checks: {:?}", cond);
        info!(
            "stake_received key: {}",
            hex::encode_hex_compact(snapshot.stake_received_kv[0].key())
        );
        info!("snapshot: {:?}", snapshot);
        return false;
    }

    // Validate the storage proof
    fn raw_kv<'a, T: FullCodec + Clone>(kv: &'a StorageKV<T>) -> (&'a [u8], Vec<u8>) {
        (&kv.0, kv.1.encode())
    }
    let mut raw_items = Vec::<(&[u8], Vec<u8>)>::new();
    raw_items.extend(snapshot.worker_state_kv.iter().map(raw_kv));
    raw_items.extend(snapshot.stake_received_kv.iter().map(raw_kv));
    raw_items.push(raw_kv(&snapshot.online_workers_kv));
    raw_items.push(raw_kv(&snapshot.compute_workers_kv));
    let raw_items_ref: Vec<_> = raw_items.iter().map(|(k, v)| (*k, v.as_slice())).collect();
    let r = light_client.validate_storage_proof(
        state_root,
        snapshot.proof.clone(),
        raw_items_ref.as_slice(),
    );
    if r.is_err() {
        error!("Snapshot light validation: {:?}", r);
        return false;
    }
    true
}

fn get_info(_input: &Map<String, Value>) -> Result<Value, Value> {
    let local_state = LOCAL_STATE.lock().unwrap();

    let initialized = local_state.initialized;
    let pk = &local_state.public_key;
    let s_pk = hex::encode_hex_compact(pk.serialize_compressed().as_ref());
    let s_ecdh_pk = match &local_state.ecdh_public_key {
        Some(ecdh_public_key) => hex::encode_hex_compact(ecdh_public_key.as_ref()),
        None => "".to_string(),
    };
    let headernum = local_state.headernum;
    let blocknum = local_state.blocknum;
    let machine_id = local_state.machine_id;

    let system_state = SYSTEM_STATE.lock().unwrap();
    let sys_seq_start = system_state.egress.sequence;
    let sys_len = system_state.egress.queue.len();

    Ok(json!({
        "initialized": initialized,
        "public_key": s_pk,
        "ecdh_public_key": s_ecdh_pk,
        "headernum": headernum,
        "blocknum": blocknum,
        "machine_id": machine_id,
        "dev_mode": local_state.dev_mode,
        "system_egress": {
            "sequence": sys_seq_start,
            "len": sys_len,
        }
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
        serde_json::from_slice(payload_data).expect("Failed to decode payload");
    let (msg, secret, pubkey) = {
        let local_state = LOCAL_STATE.lock().unwrap();
        match payload {
            types::Payload::Plain(data) => (data.into_bytes(), None, None),
            types::Payload::Cipher(cipher) => {
                info!("cipher: {:?}", cipher);
                let ecdh_privkey = local_state
                    .ecdh_private_key
                    .as_ref()
                    .expect("ECDH not initizlied");
                let result = cryptography::decrypt(&cipher, ecdh_privkey).expect("Decrypt failed");
                (
                    result.msg,
                    Some(result.secret),
                    local_state.ecdh_public_key.clone(),
                )
            }
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
    let mut state = STATE.lock().unwrap();
    let ref_origin = accid_origin.as_ref();
    let res = match opaque_query.contract_id {
        DATA_PLAZA => serde_json::to_value(
            state.contract1.handle_query(
                ref_origin,
                types::deopaque_query(opaque_query)
                    .map_err(|_| error_msg("Malformed request (data_plaza::Request)"))?
                    .request,
            ),
        )
        .unwrap(),
        BALANCES => serde_json::to_value(
            state.contract2.handle_query(
                ref_origin,
                types::deopaque_query(opaque_query)
                    .map_err(|_| error_msg("Malformed request (balances::Request)"))?
                    .request,
            ),
        )
        .unwrap(),
        ASSETS => serde_json::to_value(
            state.contract3.handle_query(
                ref_origin,
                types::deopaque_query(opaque_query)
                    .map_err(|_| error_msg("Malformed request (assets::Request)"))?
                    .request,
            ),
        )
        .unwrap(),
        WEB3_ANALYTICS => serde_json::to_value(
            state.contract4.handle_query(
                ref_origin,
                types::deopaque_query(opaque_query)
                    .map_err(|_| error_msg("Malformed request (w3a::Request)"))?
                    .request,
            ),
        )
        .unwrap(),
        DIEM => serde_json::to_value(
            state.contract5.handle_query(
                ref_origin,
                types::deopaque_query(opaque_query)
                    .map_err(|_| error_msg("Malformed request (diem::Request)"))?
                    .request,
            ),
        )
        .unwrap(),
        SUBSTRATE_KITTIES => serde_json::to_value(
            state.contract6.handle_query(
                ref_origin,
                types::deopaque_query(opaque_query)
                    .map_err(|_| error_msg("Malformed request (substrate_kitties::Request)"))?
                    .request,
            ),
        )
        .unwrap(),
        BTC_LOTTERY => serde_json::to_value(
            state.contract7.handle_query(
                ref_origin,
                types::deopaque_query(opaque_query)
                    .map_err(|_| error_msg("Malformed request (btc_lottery::Request)"))?
                    .request,
            ),
        )
        .unwrap(),
        SYSTEM => {
            let mut system_state = SYSTEM_STATE.lock().unwrap();
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
        _ => return Err(Value::Null),
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

fn get(input: &Map<String, Value>) -> Result<Value, Value> {
    let state = STATE.lock().unwrap();
    let path = input.get("path").unwrap().as_str().unwrap();

    let data = match state.contract1.get(&path.to_string()) {
        Some(d) => d,
        None => return Err(error_msg("Data doesn't exist")),
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

fn test_bridge() {
    // 1. load genesis
    let raw_genesis = base64::decode("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAh9rd6Uku4dTja+JQVMLsOZ5GtS4nU0cdpuvgchlapeMDFwoudZe3t+PYTAU5HROaYrFX54eG2MCC8p3PTBETFAAIiNw0F9UFjsS0UD4MEuoaCom+IA/piSJCPUM0AU+msO4BAAAAAAAAANF8LXgj6/Jg/ROPLX4n0RTAFF2Wi1/1AGEl8kFPra5pAQAAAAAAAAAMoQKALhCA33I0FGiDoLZ6HBWl1uCIt+sLgUbPlfMJqUk/gukhEt6AviHkl5KFGndUmA+ClBT2kPSvmBOvZTWowWjNYfynHU6AOFjSvKwU3s/vvRg7QOrJeehLgo9nGfN91yHXkHcWLkuAUJegqkIzp2A6LPkZouRRsKgiY4Wu92V8JXrn3aSXrw2AXDYZ0c8CICTMvasQ+rEpErmfEmg+BzH19s/zJX4LP8adAWRyYW5kcGFfYXV0aG9yaXRpZXNJAQEIiNw0F9UFjsS0UD4MEuoaCom+IA/piSJCPUM0AU+msO4BAAAAAAAAANF8LXgj6/Jg/ROPLX4n0RTAFF2Wi1/1AGEl8kFPra5pAQAAAAAAAABtAYKmqACASqIhjIQMli+MpltqIZlc2FVhXCd/m9F6k9Q5u13xU3JQXHh0cmluc2ljX2luZGV4EAAAAACAc0yvcsUiYcma5kSPZKxrMxbyDufisOfMmIsX1bDxfHc=")
        .expect("Bad bridge_genesis_innfo_b64");
    let genesis =
        light_validation::BridgeInitInfo::<chain::Runtime>::decode(&mut raw_genesis.as_slice())
            .expect("Can't decode bridge_genesis_info_b64");

    debug!("bridge_genesis_info_b64: {:?}", genesis);

    let mut state = STATE.lock().unwrap();
    let id = state
        .light_client
        .initialize_bridge(
            genesis.block_header,
            genesis.validator_set,
            genesis.validator_set_proof,
        )
        .expect("Init bridge failed; qed");

    // 2. import a few blocks
    let raw_header = base64::decode("/YNUiuLexTFhCwS9smQZzq1EggRC0DWkgGuCbyaKKSQEhrlzSqBeoaYA0f7EXN/Z0WJINIurZbvBQU/2dKyaFxA8RCHwO2aMQ+Agbl7pMtC9Yn6AH0rYW30BFRZmva2k5QgGYXVyYSAQWrUPAAAAAAVhdXJhAQFsP5YkXJ1qPXxseyMUtX5QXTQZBbIKDqYeZq1mw1f6MyROgQ3BIJpp8wCgSTlPttAQmkw4Ol4b5tJ5VaBzUd2B").unwrap();
    let header = chain::Header::decode(&mut raw_header.as_slice()).expect("Bad block1 header; qed");
    let justification = base64::decode("CgAAAAAAAADew4hPceq4QYh0sxLxlaq0lTl+SWKw88vuBatKPewDcwEAAAAI3sOIT3HquEGIdLMS8ZWqtJU5fklisPPL7gWrSj3sA3MBAAAAXWeJEfa3FLKCvN8SYsx3wBx3N78oHP4THt65DyExstiuwZpF62Ci18/8hdr4cf+jbdYkSBBeMJuL9dTUY/QzAojcNBfVBY7EtFA+DBLqGgqJviAP6YkiQj1DNAFPprDu3sOIT3HquEGIdLMS8ZWqtJU5fklisPPL7gWrSj3sA3MBAAAA8TA1VpLNnlBnetJ74i0IY/Bv6InpDTkG2q4LCy0qVPG3WQhgadGFMInCyc38vOHKKwA7X2r7FGfQmuuPlwRCA9F8LXgj6/Jg/ROPLX4n0RTAFF2Wi1/1AGEl8kFPra5pAA==").unwrap();

    state
        .light_client
        .submit_simple_header(id, header, justification)
        .expect("Submit first block failed; qed");
}

fn test_parse_block() {
    let raw_block: Vec<u8> = base64::decode("iAKMDRPbdbAZ0eev9OZ1QgaAkoEnazAp6JzH2GeRFYdsR+pFUBbOaAW0+k5K+jPtsEr/P/JKJQDSobnB98Qhf8ug8HkDygkapC5T++CNvzYORIFimatwYSu/U53t66xzpQgGYXVyYSCGvagPAAAAAAVhdXJhAQEuXZ5zy2+qk+60y+/m1r0oZv/+LEiDCxMotfkvjP9aebuUVxBTmd2LCpu645AAjpRUNhqOmVuiKreUoV1aMpWLCCgEAQALoPTZAm8BQQKE/9Q1k8cV/dMcYRQavQSpn9aCLIVYhUzN45pWhOelbaJ9AU5gayhZiGwAEAthrYW6Ucm+acGAR3whdfUk17jp4NMearo4+NxR2w0VsVkEF0gQ/U6AHggnM+BZmvrhhMdSygqlAQAABAD/jq8EFRaHc2Mmyf6hfiX8UodhNpPJEpCcsiaqR5TyakgHABCl1OgA")
        .unwrap();
    debug!("SignedBlock data[{}]", raw_block.len());
    let block = match parse_block(&raw_block) {
        Ok(b) => b,
        Err(err) => {
            error!("test_parse_block: Failed to parse block ({:?})", err);
            return;
        }
    };
    print_block(&block);

    // test parse address
    let ref_sig = block.block.extrinsics[1].signature.as_ref().unwrap();
    let ref_addr = &ref_sig.0;
    debug!("test_parse_block: addr = {}", format_address(ref_addr));

    let cmd_json = serde_json::to_string_pretty(&contracts::data_plaza::Command::List(
        contracts::data_plaza::ItemDetails {
            name: "nname".to_owned(),
            category: "ccategory".to_owned(),
            description: "ddesc".to_owned(),
            price: contracts::data_plaza::PricePolicy::PerRow { price: 100_u128 },
            dataset_link: "llink".to_owned(),
            dataset_preview: "pprev".to_owned(),
        },
    ))
    .expect("jah");
    debug!("sample command: {}", cmd_json);
}

fn test_ecdh(params: TestEcdhParam) {
    let bob_pub: [u8; 65] = [
        0x04, 0xb8, 0xd1, 0x8e, 0x7d, 0xe4, 0xc1, 0x10, 0x69, 0x48, 0x7b, 0x5c, 0x1e, 0x6e, 0xa5,
        0xdf, 0x04, 0x51, 0xf7, 0xe1, 0xa8, 0x46, 0x17, 0x5b, 0xf6, 0xfd, 0xf8, 0xe8, 0xea, 0x5c,
        0x68, 0xcd, 0xfb, 0xca, 0x0e, 0x1f, 0x17, 0x1c, 0x0b, 0xee, 0x3d, 0x34, 0x71, 0x11, 0x07,
        0x67, 0x2d, 0x6a, 0x13, 0x57, 0x26, 0x7d, 0x5a, 0xcb, 0x3b, 0x98, 0x4c, 0xa5, 0xbf, 0xf4,
        0xbf, 0x33, 0x78, 0x32, 0x96,
    ];

    let pubkey_data = params.pubkey_hex.map(|h| hex::decode_hex(&h));
    let pk = match pubkey_data.as_ref() {
        Some(d) => d.as_slice(),
        None => bob_pub.as_ref(),
    };

    let local_state = LOCAL_STATE.lock().unwrap();
    let alice_priv = &local_state
        .ecdh_private_key
        .as_ref()
        .expect("ECDH private key not initialized");
    let key = ecdh::agree(alice_priv, pk);
    debug!("ECDH derived secret key: {}", hex::encode_hex_compact(&key));

    if let Some(msg_b64) = params.message_b64 {
        let mut msg = base64::decode(&msg_b64).expect("Failed to decode msg_b64");
        let iv = aead::generate_iv();
        aead::encrypt(&iv, &key, &mut msg);

        debug!("AES-GCM: {}", hex::encode_hex_compact(&msg));
        debug!("IV: {}", hex::encode_hex_compact(&iv));
    }
}

fn test(param: TestReq) -> Result<Value, Value> {
    if param.test_bridge == Some(true) {
        test_bridge();
    }
    if let Some(p) = param.test_ecdh {
        test_ecdh(p);
    }
    Ok(json!({}))
}

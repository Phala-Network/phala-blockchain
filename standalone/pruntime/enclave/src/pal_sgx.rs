use std::os::unix::prelude::OsStrExt as _;
use std::str;
use std::time::Duration;

use anyhow::anyhow;
use anyhow::Result;
use http_req::request::{Method, Request};
use log::{error, info, warn};
use rand::RngCore as _;

use sgx_tcrypto::*;
use sgx_tse::*;
use sgx_tstd::io::ErrorKind;
use sgx_tstd::os::unix::prelude::OsStrExt as _;
use sgx_tstd::sgxfs::{read as sgxfs_read, write as sgxfs_write};
use sgx_types::*;
use std::convert::TryFrom;

use phactory_pal::{Machine, Sealing, RA};

use async_executor::Executor;

pub const IAS_HOST: &str = env!("IAS_HOST");
pub const IAS_SIGRL_ENDPOINT: &str = env!("IAS_SIGRL_ENDPOINT");
pub const IAS_REPORT_ENDPOINT: &str = env!("IAS_REPORT_ENDPOINT");

#[derive(Debug, Clone, Copy)]
pub(crate) struct SgxPlatform;

fn to_tstd_path(path: &std::path::Path) -> &sgx_tstd::path::Path {
    let bytes = path.as_os_str().as_bytes();
    let os_str = sgx_tstd::ffi::OsStr::from_bytes(bytes);
    sgx_tstd::path::Path::new(os_str)
}


static EXECUTOR: Executor<'static> = Executor::new();

impl SgxPlatform {
    pub fn async_executor_run() {
        info!("Async executor started");
        async_io::block_on(EXECUTOR.run(futures::future::pending::<()>()));
    }

    pub fn async_reactor_run() {
        info!("Async reactor started");
        async_io::io_main_loop();
    }
}

impl Sealing for SgxPlatform {
    type SealError = anyhow::Error;
    type UnsealError = anyhow::Error;

    fn seal_data(
        &self,
        path: impl AsRef<std::path::Path>,
        data: &[u8],
    ) -> Result<(), Self::SealError> {
        let path = to_tstd_path(path.as_ref());
        Ok(sgxfs_write(path, data)
            .map_err(|err| anyhow!("Write sealed data failed: {:?} path={:?}", err, path))?)
    }

    fn unseal_data(
        &self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<Option<Vec<u8>>, Self::UnsealError> {
        let path = to_tstd_path(path.as_ref());
        match sgxfs_read(path) {
            Ok(data) => Ok(Some(data)),
            Err(err) => {
                if matches!(err.kind(), ErrorKind::NotFound) {
                    Ok(None)
                } else {
                    Err(anyhow!("Read seal file failed: {:?} path={:?}", err, path))
                }
            }
        }
    }
}

impl RA for SgxPlatform {
    type Error = anyhow::Error;

    fn create_attestation_report(
        &self,
        data: &[u8],
    ) -> Result<(String, String, String), Self::Error> {
        create_attestation_report(data, sgx_quote_sign_type_t::SGX_LINKABLE_SIGNATURE)
    }
}

impl Machine for SgxPlatform {
    fn machine_id(&self) -> Vec<u8> {
        generate_seal_key().to_vec()
    }

    fn cpu_core_num(&self) -> u32 {
        sgx_trts::enclave::rsgx_get_cpu_core_num()
    }

    fn cpu_feature_level(&self) -> u32 {
        let mut cpu_feature_level: u32 = 1;
        // Atom doesn't support AVX
        if sgx_trts::is_x86_feature_detected!("avx2") {
            info!("CPU Support AVX2");
            cpu_feature_level += 1;

            // Customer-level Core doesn't support AVX512
            if sgx_trts::is_x86_feature_detected!("avx512f") {
                info!("CPU Support AVX512");
                cpu_feature_level += 1;
            }
        }
        cpu_feature_level
    }
}

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

    // sgx_status_t SGX_CDECL ocall_eventfd(int* retval, unsigned int init, int flags, int* errno);
    pub fn ocall_eventfd(
        ret_val: *mut libc::c_int,
        errno: *mut libc::c_int,
        init: libc::c_uint,
        flags: libc::c_int,
    ) -> sgx_status_t;

    pub fn ocall_timerfd_create(
        ret_val: *mut libc::c_int,
        errno: *mut libc::c_int,
        clockid: libc::c_int,
        flags: libc::c_int,
    ) -> sgx_status_t;

    pub fn ocall_timerfd_settime(
        ret_val: *mut libc::c_int,
        errno: *mut libc::c_int,
        fd: libc::c_int,
        flags: libc::c_int,
        new_value: *const libc::itimerspec,
        old_value: *mut libc::itimerspec,
    ) -> sgx_status_t;

    pub fn ocall_timerfd_gettime(
        ret_val: *mut libc::c_int,
        errno: *mut libc::c_int,
        fd: libc::c_int,
        curr_value: *mut libc::itimerspec,
    ) -> sgx_status_t;
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

    decode_spid(&key_str[..key_len])
}

fn decode_spid(raw_hex: &str) -> sgx_spid_t {
    let mut spid = sgx_spid_t::default();
    let raw_hex = raw_hex.trim();

    if raw_hex.len() < 16 * 2 {
        log::warn!("Input spid file len ({}) is incorrect!", raw_hex.len());
        return spid;
    }

    let decoded_vec = hex::decode(raw_hex).expect("Failed to decode SPID hex");
    spid.id.copy_from_slice(&decoded_vec[..16]);
    spid
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
    let url = TryFrom::try_from(url.as_str())
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
        let res_body = res_body_buffer;
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
    let url = TryFrom::try_from(url.as_str())
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
    cert = percent_decode(cert);
    let v: Vec<&str> = cert.split("-----").collect();
    let sig_cert = v[2].to_string();

    // len_num == 0
    (attn_report, sig, sig_cert)
}

fn percent_decode(orig: String) -> String {
    let v: Vec<&str> = orig.split('%').collect();
    let mut ret = String::new();
    ret.push_str(v[0]);
    if v.len() > 1 {
        for s in v[1..].iter() {
            ret.push(u8::from_str_radix(&s[0..2], 16).unwrap() as char);
            ret.push_str(&s[2..]);
        }
    }
    ret
}

fn as_u32_le(array: &[u8; 4]) -> u32 {
    (array[0] as u32)
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
        error!("sgx_init_quote res = {:?}", res);
        return Err(anyhow::Error::msg(res).context("init quote"));
    }

    if rt != sgx_status_t::SGX_SUCCESS {
        error!("sgx_init_quote rt = {:?}", rt);
        return Err(anyhow::Error::msg(rt).context("init quote"));
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
    let (p_sigrl, sigrl_len) = if sigrl_vec.is_empty() {
        (core::ptr::null(), 0)
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
        error!("ocall_get_quote result={}", result);
        return Err(anyhow::Error::msg(result).context("get quote"));
    }

    if rt != sgx_status_t::SGX_SUCCESS {
        error!("ocall_get_quote rt={}", rt);
        return Err(anyhow::Error::msg(rt).context("get quote"));
    }

    // Added 09-28-2018
    // Perform a check on qe_report to verify if the qe_report is valid
    match rsgx_verify_report(&qe_report) {
        Ok(()) => info!("rsgx_verify_report passed!"),
        Err(x) => {
            error!("rsgx_verify_report failed with {:?}", x);
            return Err(anyhow::Error::msg(x).context("verify report"));
        }
    }

    // Check if the qe_report is produced on the same platform
    if ti.mr_enclave.m != qe_report.body.mr_enclave.m
        || ti.attributes.flags != qe_report.body.attributes.flags
        || ti.attributes.xfrm != qe_report.body.attributes.xfrm
    {
        error!("qe_report does not match current target_info!");
        return Err(anyhow::Error::msg("Quote report check failed"));
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

    info!("rhs hash = {}", hex::encode(rhs_hash));
    info!("report hs= {}", hex::encode(lhs_hash));

    if rhs_hash != lhs_hash {
        error!("Quote is tampered!");
        return Err(anyhow::Error::msg("Quote is tampered"));
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

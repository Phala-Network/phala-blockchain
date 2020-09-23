use super::*;
use ring_aead::aead::{generic_array::GenericArray, Aead, AeadInPlace, NewAead, Payload};
use ring_aead::Aes256Gcm;
use hex_literal::hex;

type Sid = String;
#[derive(Serialize, Deserialize)]
struct Plain {
    Plain: String
}

//SetPageView
#[derive(Serialize, Deserialize)]
struct PageViewCount {
    page_view_count: u32
}

#[derive(Serialize, Deserialize)]
struct SetPageView {
    SetPageView: PageViewCount
}

//GetOnlineUsers
#[derive(Serialize, Deserialize)]
struct OnlineUser {
    sid: Sid,
    cid_count: String,
    ip_count: String,
    timestamp: u32,
}

#[derive(Serialize, Deserialize)]
struct OnlineUsers {
    online_users: Vec<OnlineUser>,
    encrypted: bool
}

#[derive(Serialize, Deserialize)]
struct GetOnlineUsers {
    GetOnlineUsers: OnlineUsers
}

//GetHourlyStats
#[derive(Serialize, Deserialize)]
pub struct HourlyPageViewStat {
    sid: Sid,
    pv_count: String,
    cid_count: String,
    avg_duration: String,
    timestamp: u32,
}

#[derive(Serialize, Deserialize)]
pub struct WeeklySite {
    sid: Sid,
    path: String,
    count: String,
    timestamp: u32,
}

#[derive(Serialize, Deserialize)]
pub struct WeeklyDevice {
    sid: Sid,
    device: String,
    count: String,
    timestamp: u32,
}

#[derive(Serialize, Deserialize)]
pub struct WeeklyClient {
    sid: Sid,
    cids: Vec<String>,
    timestamp: u32,
}

#[derive(Serialize, Deserialize)]
pub struct SiteClient {
    sid: Sid,
    cids: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct HourlyStat {
    hourly_page_view_stats: Vec<HourlyPageViewStat>,
    site_clients: Vec<SiteClient>,
    weekly_clients: Vec<WeeklyClient>,
    weekly_sites: Vec<WeeklySite>,
    weekly_devices: Vec<WeeklyDevice>,
    total_stats: Vec<HourlyPageViewStat>
}

#[derive(Serialize, Deserialize)]
struct HourlyStats {
    hourly_stat: HourlyStat,
    encrypted: bool
}

#[derive(Serialize, Deserialize)]
struct GetHourlyStats {
    GetHourlyStats: HourlyStats
}

//ClearPageView
#[derive(Serialize, Deserialize)]
struct ClearPageView {
    ClearPageView: PageViewCount
}

#[test]
fn test_w3a_setpageview() {

    init_enclave_for_test();

    let input_string = r#"{
      "input": {
        "query_payload": "{\"Plain\":\"{\\\"contract_id\\\":4,\\\"nonce\\\":655605,\\\"request\\\":{\\\"SetPageView\\\":{\\\"page_views\\\":[{\\\"id\\\":\\\"2cfcb3eb-a38e-494f-b6c0-a38c9cd2c267\\\",\\\"sid\\\":\\\"1\\\",\\\"cid\\\":\\\"d540041d837820e4a5a868c9c45d40ad\\\",\\\"host\\\":\\\"http://localhost:9000\\\",\\\"path\\\":\\\"/index.html\\\",\\\"referrer\\\":\\\"/page1.html\\\",\\\"ip\\\":\\\"::1\\\",\\\"user_agent\\\":\\\"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/85.0.4183.102 Safari/537.36\\\",\\\"created_at\\\":1600675693},{\\\"id\\\":\\\"80aad35c-7d58-494d-ba95-be95c011eb2c\\\",\\\"sid\\\":\\\"1\\\",\\\"cid\\\":\\\"d540041d837820e4a5a868c9c45d40ad\\\",\\\"host\\\":\\\"http://localhost:9000\\\",\\\"path\\\":\\\"/index.html\\\",\\\"referrer\\\":\\\"/page1.html\\\",\\\"ip\\\":\\\"::1\\\",\\\"user_agent\\\":\\\"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/85.0.4183.102 Safari/537.36\\\",\\\"created_at\\\":1600730399}],\\\"encrypted\\\":false}}}\"}"
      },
      "nonce": {
        "id": 1
      }
    }"#;

    //let contract_input: ContractInput = serde_json::from_str(input_string).unwrap();
    //assert_eq!(true, contract_input.nonce.get("id").unwrap() == 1);

    let (result, output_value) = call_in_enclave(input_string);

    assert_eq!(result, sgx_status_t::SGX_SUCCESS);
    assert_eq!(output_value.get("status").unwrap().as_str().unwrap(), "ok");

    let plain: Plain = serde_json::from_str(output_value.get("payload").unwrap().as_str().unwrap()).unwrap();
    let spv: SetPageView = serde_json::from_str(&plain.Plain).unwrap();
    assert_eq!(spv.SetPageView.page_view_count, 2);

    destroy_enclave();
}

#[test]
fn test_w3a_getonlineusers() {

    init_enclave_for_test();

    let mut input_string = r#"{
      "input": {
        "query_payload": "{\"Plain\":\"{\\\"contract_id\\\":4,\\\"nonce\\\":655605,\\\"request\\\":{\\\"SetPageView\\\":{\\\"page_views\\\":[{\\\"id\\\":\\\"2cfcb3eb-a38e-494f-b6c0-a38c9cd2c267\\\",\\\"sid\\\":\\\"1\\\",\\\"cid\\\":\\\"d540041d837820e4a5a868c9c45d40ad\\\",\\\"host\\\":\\\"http://localhost:9000\\\",\\\"path\\\":\\\"/index.html\\\",\\\"referrer\\\":\\\"/page1.html\\\",\\\"ip\\\":\\\"::1\\\",\\\"user_agent\\\":\\\"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/85.0.4183.102 Safari/537.36\\\",\\\"created_at\\\":1600675693},{\\\"id\\\":\\\"80aad35c-7d58-494d-ba95-be95c011eb2c\\\",\\\"sid\\\":\\\"1\\\",\\\"cid\\\":\\\"d540041d837820e4a5a868c9c45d40ad\\\",\\\"host\\\":\\\"http://localhost:9000\\\",\\\"path\\\":\\\"/index.html\\\",\\\"referrer\\\":\\\"/page1.html\\\",\\\"ip\\\":\\\"::1\\\",\\\"user_agent\\\":\\\"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/85.0.4183.102 Safari/537.36\\\",\\\"created_at\\\":1600730399}],\\\"encrypted\\\":false}}}\"}"
      },
      "nonce": {
        "id": 1
      }
    }"#;

    let (result, output_value) = call_in_enclave(input_string);

    assert_eq!(result, sgx_status_t::SGX_SUCCESS);
    assert_eq!(output_value.get("status").unwrap().as_str().unwrap(), "ok");

    let mut plain: Plain = serde_json::from_str(output_value.get("payload").unwrap().as_str().unwrap()).unwrap();
    let spv: SetPageView = serde_json::from_str(&plain.Plain).unwrap();
    assert_eq!(spv.SetPageView.page_view_count, 2);

    input_string = r#"{
      "input": {
        "query_payload": "{\"Plain\":\"{\\\"contract_id\\\":4,\\\"nonce\\\":123165,\\\"request\\\":{\\\"GetOnlineUsers\\\":{\\\"start\\\":1600675680,\\\"end\\\":1600742280}}}\"}"
      },
      "nonce": {
        "id": 1
      }
    }"#;

    let (result, output_value) = call_in_enclave(input_string);
    assert_eq!(result, sgx_status_t::SGX_SUCCESS);
    assert_eq!(output_value.get("status").unwrap().as_str().unwrap(), "ok");

    plain = serde_json::from_str(output_value.get("payload").unwrap().as_str().unwrap()).unwrap();
    let gou: GetOnlineUsers = serde_json::from_str(&plain.Plain).unwrap();
    assert_eq!(gou.GetOnlineUsers.encrypted, false);

    assert_eq!(gou.GetOnlineUsers.online_users.len(), 2);
    assert_eq!(gou.GetOnlineUsers.online_users[0].cid_count, "1");
    assert_eq!(gou.GetOnlineUsers.online_users[0].ip_count, "1");
    assert_eq!(gou.GetOnlineUsers.online_users[1].cid_count, "1");
    assert_eq!(gou.GetOnlineUsers.online_users[1].ip_count, "1");

    destroy_enclave();
}

#[test]
fn test_w3a_gethourlystats() {

    init_enclave_for_test();

    let mut input_string = r#"{
      "input": {
        "query_payload": "{\"Plain\":\"{\\\"contract_id\\\":4,\\\"nonce\\\":752078,\\\"request\\\":{\\\"SetPageView\\\":{\\\"page_views\\\":[{\\\"id\\\":\\\"47ca3f6f-d296-447e-90e8-5ef10e4b713e\\\",\\\"sid\\\":\\\"1\\\",\\\"cid\\\":\\\"d540041d837820e4a5a868c9c45d40ad\\\",\\\"host\\\":\\\"http://localhost:9000\\\",\\\"path\\\":\\\"/page2.html\\\",\\\"referrer\\\":\\\"/index.html\\\",\\\"ip\\\":\\\"::1\\\",\\\"user_agent\\\":\\\"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/85.0.4183.102 Safari/537.36\\\",\\\"created_at\\\":1600822028},{\\\"id\\\":\\\"81bee3d3-3b3c-4366-ba08-1ee9884e29ee\\\",\\\"sid\\\":\\\"1\\\",\\\"cid\\\":\\\"d540041d837820e4a5a868c9c45d40ad\\\",\\\"host\\\":\\\"http://localhost:9000\\\",\\\"path\\\":\\\"/index.html\\\",\\\"referrer\\\":\\\"/page2.html\\\",\\\"ip\\\":\\\"::1\\\",\\\"user_agent\\\":\\\"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/85.0.4183.102 Safari/537.36\\\",\\\"created_at\\\":1600822045},{\\\"id\\\":\\\"5effeb8f-b62e-43ae-a7ab-69fc49efb8ab\\\",\\\"sid\\\":\\\"1\\\",\\\"cid\\\":\\\"d540041d837820e4a5a868c9c45d40ad\\\",\\\"host\\\":\\\"http://localhost:9000\\\",\\\"path\\\":\\\"/page1.html\\\",\\\"referrer\\\":\\\"/index.html\\\",\\\"ip\\\":\\\"::1\\\",\\\"user_agent\\\":\\\"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/85.0.4183.102 Safari/537.36\\\",\\\"created_at\\\":1600822081}],\\\"encrypted\\\":false}}}\"}"
      },
      "nonce": {
        "id": 1
      }
    }"#;

    let (result, output_value) = call_in_enclave(input_string);

    assert_eq!(result, sgx_status_t::SGX_SUCCESS);
    assert_eq!(output_value.get("status").unwrap().as_str().unwrap(), "ok");

    let mut plain: Plain = serde_json::from_str(output_value.get("payload").unwrap().as_str().unwrap()).unwrap();
    let spv: SetPageView = serde_json::from_str(&plain.Plain).unwrap();
    assert_eq!(spv.SetPageView.page_view_count, 3);

    input_string = r#"{
      "input": {
        "query_payload": "{\"Plain\":\"{\\\"contract_id\\\":4,\\\"nonce\\\":448170,\\\"request\\\":{\\\"GetHourlyStats\\\":{\\\"start\\\":1600822020,\\\"end\\\":1600822800,\\\"start_of_week\\\":1600646400}}}\"}"
      },
      "nonce": {
        "id": 1
      }
    }"#;

    let (result, output_value) = call_in_enclave(input_string);
    assert_eq!(result, sgx_status_t::SGX_SUCCESS);
    assert_eq!(output_value.get("status").unwrap().as_str().unwrap(), "ok");

    plain = serde_json::from_str(output_value.get("payload").unwrap().as_str().unwrap()).unwrap();
    let ghs: GetHourlyStats = serde_json::from_str(&plain.Plain).unwrap();

    assert_eq!(ghs.GetHourlyStats.encrypted, false);

    assert_eq!(ghs.GetHourlyStats.hourly_stat.hourly_page_view_stats.len(), 1);
    assert_eq!(ghs.GetHourlyStats.hourly_stat.hourly_page_view_stats[0].cid_count, "1");
    assert_eq!(ghs.GetHourlyStats.hourly_stat.hourly_page_view_stats[0].pv_count, "3");
    assert_eq!(ghs.GetHourlyStats.hourly_stat.hourly_page_view_stats[0].avg_duration, "26");

    assert_eq!(ghs.GetHourlyStats.hourly_stat.site_clients.len(), 1);
    assert_eq!(ghs.GetHourlyStats.hourly_stat.site_clients[0].cids.len(), 1);

    assert_eq!(ghs.GetHourlyStats.hourly_stat.weekly_devices.len(), 1);
    assert_eq!(ghs.GetHourlyStats.hourly_stat.weekly_devices[0].count, "3");
    assert_eq!(ghs.GetHourlyStats.hourly_stat.weekly_devices[0].device, "Chrome");

    assert_eq!(ghs.GetHourlyStats.hourly_stat.weekly_sites.len(), 3);
    assert_eq!(ghs.GetHourlyStats.hourly_stat.weekly_sites[0].count, "1");
    assert_eq!(ghs.GetHourlyStats.hourly_stat.weekly_sites[0].path, "/page2.html");
    assert_eq!(ghs.GetHourlyStats.hourly_stat.weekly_sites[1].count, "1");
    assert_eq!(ghs.GetHourlyStats.hourly_stat.weekly_sites[1].path, "/index.html");
    assert_eq!(ghs.GetHourlyStats.hourly_stat.weekly_sites[2].count, "1");
    assert_eq!(ghs.GetHourlyStats.hourly_stat.weekly_sites[2].path, "/page1.html");

    destroy_enclave();
}

#[test]
fn test_w3a_clearpageview() {

    init_enclave_for_test();

    let mut input_string = r#"{
      "input": {
        "query_payload": "{\"Plain\":\"{\\\"contract_id\\\":4,\\\"nonce\\\":752078,\\\"request\\\":{\\\"SetPageView\\\":{\\\"page_views\\\":[{\\\"id\\\":\\\"47ca3f6f-d296-447e-90e8-5ef10e4b713e\\\",\\\"sid\\\":\\\"1\\\",\\\"cid\\\":\\\"d540041d837820e4a5a868c9c45d40ad\\\",\\\"host\\\":\\\"http://localhost:9000\\\",\\\"path\\\":\\\"/page2.html\\\",\\\"referrer\\\":\\\"/index.html\\\",\\\"ip\\\":\\\"::1\\\",\\\"user_agent\\\":\\\"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/85.0.4183.102 Safari/537.36\\\",\\\"created_at\\\":1600822028},{\\\"id\\\":\\\"81bee3d3-3b3c-4366-ba08-1ee9884e29ee\\\",\\\"sid\\\":\\\"1\\\",\\\"cid\\\":\\\"d540041d837820e4a5a868c9c45d40ad\\\",\\\"host\\\":\\\"http://localhost:9000\\\",\\\"path\\\":\\\"/index.html\\\",\\\"referrer\\\":\\\"/page2.html\\\",\\\"ip\\\":\\\"::1\\\",\\\"user_agent\\\":\\\"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/85.0.4183.102 Safari/537.36\\\",\\\"created_at\\\":1600822045},{\\\"id\\\":\\\"5effeb8f-b62e-43ae-a7ab-69fc49efb8ab\\\",\\\"sid\\\":\\\"1\\\",\\\"cid\\\":\\\"d540041d837820e4a5a868c9c45d40ad\\\",\\\"host\\\":\\\"http://localhost:9000\\\",\\\"path\\\":\\\"/page1.html\\\",\\\"referrer\\\":\\\"/index.html\\\",\\\"ip\\\":\\\"::1\\\",\\\"user_agent\\\":\\\"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/85.0.4183.102 Safari/537.36\\\",\\\"created_at\\\":1600822081}],\\\"encrypted\\\":false}}}\"}"
      },
      "nonce": {
        "id": 1
      }
    }"#;

    let (result, output_value) = call_in_enclave(input_string);

    assert_eq!(result, sgx_status_t::SGX_SUCCESS);
    assert_eq!(output_value.get("status").unwrap().as_str().unwrap(), "ok");

    let mut plain: Plain = serde_json::from_str(output_value.get("payload").unwrap().as_str().unwrap()).unwrap();
    let spv: SetPageView = serde_json::from_str(&plain.Plain).unwrap();
    assert_eq!(spv.SetPageView.page_view_count, 3);

    input_string = r#"{
      "input": {
        "query_payload": "{\"Plain\":\"{\\\"contract_id\\\":4,\\\"nonce\\\":503566,\\\"request\\\":{\\\"ClearPageView\\\":{\\\"timestamp\\\":1600826400}}}\"}"
      },
      "nonce": {
        "id": 1
      }
    }"#;

    let (result, output_value) = call_in_enclave(input_string);
    assert_eq!(result, sgx_status_t::SGX_SUCCESS);
    assert_eq!(output_value.get("status").unwrap().as_str().unwrap(), "ok");

    plain = serde_json::from_str(output_value.get("payload").unwrap().as_str().unwrap()).unwrap();
    let cpv: ClearPageView = serde_json::from_str(&plain.Plain).unwrap();
    assert_eq!(cpv.ClearPageView.page_view_count, 0);

    destroy_enclave();
}

#[test]
fn test_w3a_encrypted_getonlineusers() {

    init_enclave_for_test();

    let mut input_string = r#"{
      "input": {
        "query_payload": "{\"Plain\":\"{\\\"contract_id\\\":4,\\\"nonce\\\":472339,\\\"request\\\":{\\\"SetPageView\\\":{\\\"page_views\\\":[{\\\"id\\\":\\\"2cfcb3eb-a38e-494f-b6c0-a38c9cd2c267\\\",\\\"sid\\\":\\\"1\\\",\\\"cid\\\":\\\"d540041d837820e4a5a868c9c45d40ad\\\",\\\"host\\\":\\\"Rl1ZDc82BIoTipjA|FtXjNvvJCwj6S7IimvP8BzmySwoFH9PIM64sj02FQxGJY+ob7g==\\\",\\\"path\\\":\\\"7/yzt7blRiXvgVvC|fZbun6AiTf3BwOjFLx47frnXeDiiJfTx/IBT\\\",\\\"referrer\\\":\\\"0lT0tnE/PMCcarWR|b1Y3k81MomkrHzfDR3d8/9iQdLNP/nHUKrTR\\\",\\\"ip\\\":\\\"w2K5n/ZRtof8p9P0|29vBKOrRVDRLIBnRK6F1isoB6A==\\\",\\\"user_agent\\\":\\\"iWjDOvbhkRciGetZ|KweE9m2FimyYRM86eJM2aLcs4OPOUy+UHtTN8jhZVDLjiJeV+SGFr/loy+6/e/xdYCK9BfYTB0UErcljABtva7OEgNtKTK2cygW0Sb/QXwB3EhqpmABoMMVLR4NPOkqTqFiwC3vdxxT52Is4xZfJA6GUTD44FZfbOg==\\\",\\\"created_at\\\":1600675693},{\\\"id\\\":\\\"80aad35c-7d58-494d-ba95-be95c011eb2c\\\",\\\"sid\\\":\\\"1\\\",\\\"cid\\\":\\\"d540041d837820e4a5a868c9c45d40ad\\\",\\\"host\\\":\\\"D0+sobfMLVjxknzm|WefGXy36Ovr2pukulGX5fieIBVWFzOelqx2EsuVBQ5OkgwAwCw==\\\",\\\"path\\\":\\\"I2188wkIE7brW4go|2o81adRytEimk27P5tnj96V2HTBOVCjTvMFf\\\",\\\"referrer\\\":\\\"JmtxXYxtW2ZH8cH/|A1DloMwS5F85vIM5h27ZGqLFM3R8hSPrzPIK\\\",\\\"ip\\\":\\\"q3Lybr/NRVC7DxCu|d8bvmbSmqSkxAkJ0Ed3boKFWRg==\\\",\\\"user_agent\\\":\\\"x+Br1zBK2glAPBW6|0+QwC2Rr+m3uWdD4u59VGxmGI4bBkFWJsTEH6NK3Mis8ejxASHkPu9pYAfvyPL3Er/5iagdHgoaJs7V6LpTCvBgs2vVb2f7Oa+xYtLOEgYuuzQgM03Fmlhd/CEAmOoBCoZdgjFzENNzT5Oh47Uc5OJFALRSgzRXXyw==\\\",\\\"created_at\\\":1600730399}],\\\"encrypted\\\":true}}}\"}"
      },
      "nonce": {
        "id": 1
      }
    }"#;

    let (result, output_value) = call_in_enclave(input_string);

    assert_eq!(result, sgx_status_t::SGX_SUCCESS);
    assert_eq!(output_value.get("status").unwrap().as_str().unwrap(), "ok");

    let mut plain: Plain = serde_json::from_str(output_value.get("payload").unwrap().as_str().unwrap()).unwrap();
    let spv: SetPageView = serde_json::from_str(&plain.Plain).unwrap();
    assert_eq!(spv.SetPageView.page_view_count, 2);

    input_string = r#"{
      "input": {
        "query_payload": "{\"Plain\":\"{\\\"contract_id\\\":4,\\\"nonce\\\":123165,\\\"request\\\":{\\\"GetOnlineUsers\\\":{\\\"start\\\":1600675680,\\\"end\\\":1600742280}}}\"}"
      },
      "nonce": {
        "id": 1
      }
    }"#;

    let (result, output_value) = call_in_enclave(input_string);
    assert_eq!(result, sgx_status_t::SGX_SUCCESS);
    assert_eq!(output_value.get("status").unwrap().as_str().unwrap(), "ok");

    plain = serde_json::from_str(output_value.get("payload").unwrap().as_str().unwrap()).unwrap();
    let gou: GetOnlineUsers = serde_json::from_str(&plain.Plain).unwrap();

    assert_eq!(gou.GetOnlineUsers.encrypted, true);

    assert_eq!(gou.GetOnlineUsers.online_users.len(), 2);
    assert_eq!(decrypt(gou.GetOnlineUsers.online_users[0].cid_count.clone()), "1");
    assert_eq!(decrypt(gou.GetOnlineUsers.online_users[0].ip_count.clone()), "1");
    assert_eq!(decrypt(gou.GetOnlineUsers.online_users[1].cid_count.clone()), "1");
    assert_eq!(decrypt(gou.GetOnlineUsers.online_users[1].ip_count.clone()), "1");

    destroy_enclave();
}

fn init_enclave_for_test() {
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
}

fn call_in_enclave(input_string: &str) -> (sgx_status_t, serde_json::value::Value) {
    let eid = get_eid();
    let mut retval = sgx_status_t::SGX_SUCCESS;

    let mut return_output_buf = vec![0; ENCLAVE_OUTPUT_BUF_MAX_LEN].into_boxed_slice();
    let mut output_len : usize = 0;
    let output_slice = &mut return_output_buf;
    let output_ptr = output_slice.as_mut_ptr();
    let output_len_ptr = &mut output_len as *mut usize;

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

    return (result, output_value);
}

fn decrypt(cipher: String) -> String {
    let vec: Vec<&str> = cipher.split("|").collect();
    let iv = base64::decode(vec[0]).unwrap();
    let cipher_data = base64::decode(vec[1]).unwrap();

    aead_decrypt(&iv, &cipher_data)
}

fn aead_decrypt(iv: &[u8], cipher_data: &[u8]) -> String {
    let key = hex!("290c3c5d812a4ba7ce33adf09598a462692a615beb6c80fdafb3f9e3bbef8bc6");
    let payload = Payload {
        msg: cipher_data,
        aad: b"",
    };
    let cipher = Aes256Gcm::new(GenericArray::from_slice(&key));
    let plaintext = cipher.decrypt(GenericArray::from_slice(iv), payload).unwrap();

    String::from_utf8(plaintext).unwrap()
}

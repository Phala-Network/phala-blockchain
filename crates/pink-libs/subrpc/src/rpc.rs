use crate::traits::common::Error;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use pink_extension::http_post;

pub fn call_rpc(rpc_node: &str, data: Vec<u8>) -> core::result::Result<Vec<u8>, Error> {
    let content_length = format!("{}", data.len());
    let headers: Vec<(String, String)> = vec![
        ("Content-Type".into(), "application/json".into()),
        ("Content-Length".into(), content_length),
    ];
    let response = http_post!(rpc_node, data, headers);

    if response.status_code != 200 {
        return Err(Error::SubRPCRequestFailed);
    }

    Ok(response.body)
}

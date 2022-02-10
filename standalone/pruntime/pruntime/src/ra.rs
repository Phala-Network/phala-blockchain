use anyhow::{anyhow, Context as _, Result};
use http_req::request::{Method, Request};
use log::{error, warn};
use std::{convert::TryFrom, fs, time::Duration};

pub const IAS_HOST: &str = env!("IAS_HOST");
pub const IAS_REPORT_ENDPOINT: &str = env!("IAS_REPORT_ENDPOINT");

fn get_report_from_intel(quote: &[u8], ias_key: &str) -> Result<(String, String, String)> {
    let encoded_quote = base64::encode(quote);
    let encoded_json = format!("{{\"isvEnclaveQuote\":\"{}\"}}\r\n", encoded_quote);

    let mut res_body_buffer = Vec::new(); //container for body of a response
    let timeout = Some(Duration::from_secs(8));

    let url = format!("https://{}{}", IAS_HOST, IAS_REPORT_ENDPOINT);
    let url = TryFrom::try_from(url.as_str()).context("Invalid IAS URI")?;
    let res = Request::new(&url)
        .header("Connection", "Close")
        .header("Content-Type", "application/json")
        .header("Content-Length", &encoded_json.len())
        .header("Ocp-Apim-Subscription-Key", ias_key)
        .method(Method::POST)
        .body(encoded_json.as_bytes())
        .timeout(timeout)
        .connect_timeout(timeout)
        .read_timeout(timeout)
        .send(&mut res_body_buffer)
        .context("Http request to IAS failed")?;

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
        return Err(anyhow!(format!("Bad http status: {}", status_code)));
    }

    let content_len = match res.content_len() {
        Some(len) => len,
        _ => {
            warn!("content_length not found");
            0
        }
    };

    if content_len == 0 {
        return Err(anyhow!("Empty HTTP response"));
    }

    let attn_report = String::from_utf8(res_body_buffer).context("UTF8 decode response")?;
    let sig = res
        .headers()
        .get("X-IASReport-Signature")
        .context("Get X-IASReport-Signature")?
        .to_string();
    let cert = res
        .headers()
        .get("X-IASReport-Signing-Certificate")
        .context("Get X-IASReport-Signing-Certificate")?
        .to_string();

    // Remove %0A from cert, and only obtain the signing cert
    let cert = cert.replace("%0A", "");
    let cert = urlencoding::decode(&cert).context("percent_decode cert")?;
    let v: Vec<&str> = cert.split("-----").collect();
    let sig_cert = v[2].to_string();

    // len_num == 0
    Ok((attn_report, sig, sig_cert))
}

pub fn create_quote_vec(data: &[u8]) -> Result<Vec<u8>> {
    fs::write("/dev/attestation/user_report_data", data)?;
    Ok(fs::read("/dev/attestation/quote")?)
}

pub fn create_attestation_report(data: &[u8], ias_key: &str) -> Result<(String, String, String)> {
    let quote_vec = create_quote_vec(data)?;
    let (attn_report, sig, cert) = get_report_from_intel(&quote_vec, ias_key)?;
    Ok((attn_report, sig, cert))
}

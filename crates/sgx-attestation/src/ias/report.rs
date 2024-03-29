use anyhow::{anyhow, Context as _, Result};
use std::time::Duration;
use tracing::{error, info, warn};

use crate::{gramine::create_quote_vec, AttestationReport};

use base64::{engine::general_purpose::STANDARD as base64, Engine};

use reqwest_env_proxy::EnvProxyBuilder as _;

const IAS_HOST: &str = env!("IAS_HOST");
const IAS_REPORT_ENDPOINT: &str = env!("IAS_REPORT_ENDPOINT");
const IAS_API_KEY_STR: &str = env!("IAS_API_KEY");

fn get_report_from_intel(quote: &[u8], timeout: Duration) -> Result<(String, String, String)> {
    let encoded_quote = base64.encode(quote);
    let encoded_json = format!("{{\"isvEnclaveQuote\":\"{encoded_quote}\"}}\r\n");

    let mut res_body_buffer = Vec::new(); //container for body of a response

    let url: reqwest::Url = format!("https://{IAS_HOST}{IAS_REPORT_ENDPOINT}").parse()?;
    info!(from=%url, "Getting RA report");
    let mut res = reqwest::blocking::Client::builder()
        .hickory_dns(true)
        .timeout(Some(timeout))
        .env_proxy(url.domain().unwrap_or_default())
        .build()
        .context("Failed to create http client, maybe invalid IAS URI")?
        .post(url)
        .header("Connection", "Close")
        .header("Content-Type", "application/json")
        .header("Ocp-Apim-Subscription-Key", IAS_API_KEY_STR)
        .body(encoded_json)
        .send()
        .context("Failed to send http request")?;

    let status_code = res.status().as_u16();
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

        error!(%msg);
        return Err(anyhow!(format!("Bad http status: {status_code}")));
    }

    let content_len = match res.content_length() {
        Some(len) => len,
        _ => {
            warn!("content_length not found");
            0
        }
    };

    if content_len == 0 {
        return Err(anyhow!("Empty HTTP response"));
    }

    res.copy_to(&mut res_body_buffer)
        .context("Failed to read response body from IAS")?;

    let attn_report =
        String::from_utf8(res_body_buffer).context("Failed to decode attestation report")?;
    let sig = res
        .headers()
        .get("X-IASReport-Signature")
        .context("No header X-IASReport-Signature")?
        .to_str()
        .context("Failed to decode X-IASReport-Signature")?;
    let cert = res
        .headers()
        .get("X-IASReport-Signing-Certificate")
        .context("No header X-IASReport-Signing-Certificate")?
        .to_str()
        .context("Failed to decode X-IASReport-Signing-Certificate")?;

    // Remove %0A from cert, and only obtain the signing cert
    let cert = cert.replace("%0A", "");
    let cert = urlencoding::decode(&cert).context("Failed to urldecode cert")?;
    let v: Vec<&str> = cert.split("-----").collect();
    let sig_cert = v[2].to_string();

    // len_num == 0
    Ok((attn_report, sig.into(), sig_cert))
}

pub fn create_attestation_report(data: &[u8], timeout: Duration) -> Result<AttestationReport> {
    let quote_vec = create_quote_vec(data)?;
    let (ra_report, sig, cert) = get_report_from_intel(&quote_vec, timeout)?;

    let signature = base64
        .decode(sig)
        .context("Failed to decode sig in base64 format")?;
    let raw_signing_cert = base64
        .decode(cert)
        .context("Failed to decode cert in base64 format")?;
    Ok(AttestationReport::SgxIas {
        ra_report: ra_report.into_bytes(),
        signature,
        raw_signing_cert,
    })
}

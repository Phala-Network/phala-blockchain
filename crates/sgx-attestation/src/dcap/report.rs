use core::time::Duration;

use alloc::string::{String, ToString};
use anyhow::{anyhow, Result};
use futures::executor::block_on;
use pink_types::sgx::Collateral;
use scale::Decode;

use super::{quote::Quote, SgxV30QuoteCollateral};
use crate::{gramine::create_quote_vec, AttestationReport};

fn get_header(resposne: &reqwest::Response, name: &str) -> Result<String> {
    let value = resposne
        .headers()
        .get(name)
        .ok_or(anyhow!("Missing {name}"))?
        .to_str()?;
    let value = urlencoding::decode(value)?;
    Ok(value.into_owned())
}

/// Get collateral given DCAP quote and base URL of PCCS server URL.
pub async fn get_collateral(
    pccs_url: &str,
    mut quote: &[u8],
    timeout: Duration,
) -> Result<SgxV30QuoteCollateral> {
    let quote = Quote::decode(&mut quote)?;
    let fmspc = hex::encode_upper(quote.fmspc().map_err(|_| anyhow!("get fmspc error"))?);
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(timeout)
        .build()?;
    let base_url = pccs_url.trim_end_matches('/');

    let pck_crl_issuer_chain;
    let pck_crl;
    {
        let response = client
            .get(format!("{base_url}/pckcrl?ca=processor"))
            .send()
            .await?;
        pck_crl_issuer_chain = get_header(&response, "SGX-PCK-CRL-Issuer-Chain")?;
        pck_crl = response.text().await?;
    };
    let root_ca_crl = client
        .get(format!("{base_url}/rootcacrl"))
        .send()
        .await?
        .text()
        .await?;
    let tcb_info_issuer_chain;
    let raw_tcb_info;
    {
        let resposne = client
            .get(format!("{base_url}/tcb?fmspc={fmspc}"))
            .send()
            .await?;
        tcb_info_issuer_chain = get_header(&resposne, "SGX-TCB-Info-Issuer-Chain")
            .or(get_header(&resposne, "TCB-Info-Issuer-Chain"))?;
        raw_tcb_info = resposne.text().await?;
    };
    let qe_identity_issuer_chain;
    let raw_qe_identity;
    {
        let response = client.get(format!("{base_url}/qe/identity")).send().await?;
        qe_identity_issuer_chain = get_header(&response, "SGX-Enclave-Identity-Issuer-Chain")?;
        raw_qe_identity = response.text().await?;
    };

    let tcb_info_json: serde_json::Value =
        serde_json::from_str(&raw_tcb_info).map_err(|_| anyhow!("TCB Info should a JSON"))?;
    let tcb_info = tcb_info_json["tcbInfo"].to_string();
    let tcb_info_signature = tcb_info_json
        .get("signature")
        .ok_or(anyhow!("TCB Info should has `signature` field"))?
        .as_str()
        .ok_or(anyhow!("TCB Info signature should a hex string"))?;
    let tcb_info_signature = hex::decode(tcb_info_signature)
        .map_err(|_| anyhow!("TCB Info signature should a hex string"))?;

    let qe_identity_json: serde_json::Value = serde_json::from_str(raw_qe_identity.as_str())
        .map_err(|_| anyhow!("QE Identity should a JSON"))?;
    let qe_identity = qe_identity_json
        .get("enclaveIdentity")
        .ok_or(anyhow!("QE Identity should has `enclaveIdentity` field"))?
        .to_string();
    let qe_identity_signature = qe_identity_json
        .get("signature")
        .ok_or(anyhow!("QE Identity should has `signature` field"))?
        .as_str()
        .ok_or(anyhow!("QE Identity signature should a hex string"))?;
    let qe_identity_signature = hex::decode(qe_identity_signature)
        .map_err(|_| anyhow!("QE Identity signature should a hex string"))?;

    Ok(SgxV30QuoteCollateral {
        pck_crl_issuer_chain,
        root_ca_crl,
        pck_crl,
        tcb_info_issuer_chain,
        tcb_info,
        tcb_info_signature,
        qe_identity_issuer_chain,
        qe_identity,
        qe_identity_signature,
    })
}

pub fn create_attestation_report(
    data: &[u8],
    pccs_url: &str,
    timeout: Duration,
) -> Result<AttestationReport> {
    let quote = create_quote_vec(data)?;
    let collateral = if pccs_url.is_empty() {
        None
    } else {
        let collateral = block_on(get_collateral(pccs_url, &quote, timeout))?;
        Some(Collateral::SgxV30(collateral))
    };
    Ok(AttestationReport::SgxDcap { quote, collateral })
}

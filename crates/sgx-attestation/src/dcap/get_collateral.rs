use core::time::Duration;

use alloc::string::String;
use anyhow::{anyhow, Result};
use scale::Decode;

use super::QuoteCollateral;

mod parse_quote;

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
) -> Result<QuoteCollateral> {
    let quote = parse_quote::Quote::decode(&mut quote)?;
    let fmspc = hex::encode_upper(quote.fmspc()?);
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(timeout)
        .build()?;
    let base_url = pccs_url.trim_end_matches('/');
    let pck_crl_issuer_chain;
    let pck_crl;
    {
        let resposne = client
            .get(format!("{base_url}/pckcrl?ca=processor"))
            .send()
            .await?;
        pck_crl_issuer_chain = get_header(&resposne, "SGX-PCK-CRL-Issuer-Chain")?;
        pck_crl = resposne.text().await?;
    };
    let root_ca_crl = client
        .get(format!("{base_url}/rootcacrl"))
        .send()
        .await?
        .text()
        .await?;
    let tcb_info_issuer_chain;
    let tcb_info;
    {
        let resposne = client
            .get(format!("{base_url}/tcb?fmspc={fmspc}"))
            .send()
            .await?;
        tcb_info_issuer_chain = get_header(&resposne, "SGX-TCB-Info-Issuer-Chain")
            .or(get_header(&resposne, "TCB-Info-Issuer-Chain"))?;
        tcb_info = resposne.text().await?;
    };
    let qe_identity_issuer_chain;
    let qe_identity;
    {
        let resposne = client.get(format!("{base_url}/qe/identity")).send().await?;
        qe_identity_issuer_chain = get_header(&resposne, "SGX-Enclave-Identity-Issuer-Chain")?;
        qe_identity = resposne.text().await?;
    };
    Ok(QuoteCollateral {
        pck_crl_issuer_chain: pck_crl_issuer_chain.into_bytes(),
        root_ca_crl: root_ca_crl.into_bytes(),
        pck_crl: pck_crl.into_bytes(),
        tcb_info_issuer_chain: tcb_info_issuer_chain.into_bytes(),
        tcb_info: tcb_info.into_bytes(),
        qe_identity_issuer_chain: qe_identity_issuer_chain.into_bytes(),
        qe_identity: qe_identity.into_bytes(),
    })
}

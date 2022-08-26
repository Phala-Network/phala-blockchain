use crate::{
    chain_client,
    types::{ParachainApi, PrClient, SrSigner},
    Args,
};
use anyhow::{anyhow, Result};
use codec::Decode;
use log::{error, info};

async fn update_worker_endpoint(
    para_api: &ParachainApi,
    encoded_endpoint_payload: Vec<u8>,
    signature: Vec<u8>,
    signer: &mut SrSigner,
    args: &Args,
) -> Result<bool> {
    chain_client::update_signer_nonce(para_api, signer).await?;
    let signed_endpoint = Decode::decode(&mut &encoded_endpoint_payload[..])
        .map_err(|_| anyhow!("Decode signed endpoint failed"))?;
    let params = crate::mk_params(para_api, args.longevity, args.tip).await?;
    let tx = phaxt::parachain::tx()
        .phala_registry()
        .update_worker_endpoint(signed_endpoint, signature);
    let ret = para_api
        .tx()
        .sign_and_submit_then_watch(&tx, signer, params)
        .await;
    if ret.is_err() {
        error!("FailedToCallBindWorkerEndpoint: {:?}", ret);
        return Err(anyhow!("failed to call update_worker_endpoint"));
    }
    signer.increment_nonce();
    Ok(true)
}

pub async fn try_update_worker_endpoint(
    pr: &PrClient,
    para_api: &ParachainApi,
    signer: &mut SrSigner,
    args: &Args,
) -> Result<bool> {
    let info = pr.get_endpoint_info(()).await?;
    let encoded_endpoint_payload = match info.encoded_endpoint_payload {
        None => return Ok(false), // Early return if no endpoint payload is available
        Some(payload) => payload,
    };
    let signature = info.signature.ok_or(anyhow!("No endpoint signature"))?;
    info!("Binding worker's endpoint...");
    update_worker_endpoint(&para_api, encoded_endpoint_payload, signature, signer, args).await
}

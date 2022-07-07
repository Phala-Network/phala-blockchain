use anyhow::{anyhow, Result};
use log::{error, info};
use crate::{chain_client, Args, types::{ParachainApi, SrSigner, PrClient}};
use codec::Decode;

async fn update_worker_endpoint(
    para_api: &ParachainApi,
    encoded_endpoint_payload: Vec<u8>,
    signature: Vec<u8>,
    signer: &mut SrSigner,
    args: &Args,
) -> Result<()> {
    chain_client::update_signer_nonce(para_api, signer).await?;
    let signed_endpoint = Decode::decode(&mut &encoded_endpoint_payload[..])
        .map_err(|_| anyhow!("Decode signed endpoint failed"))?;
    let params = crate::mk_params(para_api, args.longevity, args.tip).await?;
    let ret = para_api
        .tx()
        .phala_registry()
        .update_worker_endpoint(signed_endpoint, signature)?
        .sign_and_submit_then_watch(signer, params)
        .await;
    if ret.is_err() {
        error!("FailedToCallBindWorkerEndpoint: {:?}", ret);
        return Err(anyhow!("failed to call update_worker_endpoint"));
    }
    signer.increment_nonce();
    Ok(())
}

pub async fn try_update_worker_endpoint(
    pr: &PrClient,
    para_api: &ParachainApi,
    signer: &mut SrSigner,
    args: &Args,
) -> Result<()> {
    let info = pr.get_endpoint_info(()).await?;
    if let signature = info.signature.ok_or(anyhow!("No endpoint signature"))? {
        info!("Binding worker's endpoint...");
        update_worker_endpoint(
            &para_api,
            info.encoded_endpoint_payload,
            signature,
            signer,
            args,
        )
            .await?;
        return Ok(());
    };

    Err(anyhow!("No endpoint signature"))
}
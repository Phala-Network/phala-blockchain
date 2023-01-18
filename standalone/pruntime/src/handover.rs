use crate::pal_gramine::GraminePlatform;
use anyhow::{Context, Result};
use phactory::RpcService;
use phactory_api::{
    ecall_args::InitArgs, prpc::phactory_api_server::PhactoryApi,
    pruntime_client::new_pruntime_client,
};

pub(crate) async fn handover_from(url: &str, args: InitArgs) -> Result<()> {
    let mut this = RpcService::new(GraminePlatform);
    this.lock_phactory().init(args);

    let from_pruntime = new_pruntime_client(url.into());
    let challenge = from_pruntime
        .handover_create_challenge(())
        .await
        .context("Failed to create challenge")?;
    let response = this
        .handover_accept_challenge(challenge)
        .await
        .context("Failed to accept challenge")?;
    let encrypted_key = from_pruntime
        .handover_start(response)
        .await
        .context("Failed to start handover")?;
    this.handover_receive(encrypted_key)
        .await
        .context("Failed to receive handover result")?;
    Ok(())
}

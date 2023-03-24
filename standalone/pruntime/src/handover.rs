use crate::pal_gramine::GraminePlatform;
use anyhow::{Context, Result};
use phactory::RpcService;
use phactory_api::{
    ecall_args::InitArgs, prpc::phactory_api_server::PhactoryApi,
    pruntime_client::new_pruntime_client,
};
use tracing::info;

pub(crate) async fn handover_from(url: &str, args: InitArgs) -> Result<()> {
    let mut this = RpcService::new(GraminePlatform);
    this.lock_phactory().init(args);

    let from_pruntime = new_pruntime_client(url.into());
    info!("Requesting for challenge");
    let challenge = from_pruntime
        .handover_create_challenge(())
        .await
        .context("Failed to create challenge")?;
    info!("Challenge received");
    let response = this
        .handover_accept_challenge(challenge)
        .await
        .context("Failed to accept challenge")?;
    info!("Requesting for key");
    let encrypted_key = from_pruntime
        .handover_start(response)
        .await
        .context("Failed to start handover")?;
    info!("Key received");
    this.handover_receive(encrypted_key)
        .await
        .context("Failed to receive handover result")?;
    Ok(())
}

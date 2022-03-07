use anyhow::Result;
use phactory_api::prpc as pb;
use phactory_api::pruntime_client::new_pruntime_client;

#[tokio::main]
async fn main() -> Result<()> {
    let pruntime_endpoint =
        std::env::var("POD_PRUNTIME_ENDPOINT").expect("POD_PRUNTIME_ENDPOINT must be set");
    let id = std::env::var("POD_ID").expect("POD_ID must be set");
    let prpc_url = format!("{}/prpc", pruntime_endpoint);
    let client = new_pruntime_client(prpc_url);
    let pubkey = todo!();
    let quote = todo!();
    let response = client.register_pod(pb::RegisterPodInfo {
        id,
        quote,
        pubkey,
    }).await?;
    Ok(())
}

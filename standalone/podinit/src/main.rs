use anyhow::{Result, Context};
use phactory_api::pruntime_client;
use phala_podauth::{AuthRequest, AuthResponse};
use sp_core::crypto::Pair;

#[tokio::main]
fn main() -> Result<()> {
    let key = pruntime_client::gen_key();
    let pubkey = key.public();
    let quote = create_quote_vec(&pubkey).context("Unable to crate quote")?;
    let request = AuthRequest { quote };
    let response: AuthResponse =
        pruntime_client::query(url, id, data, private_key, remote_pubkey).await?;
    Ok(())
}

pub fn create_quote_vec(data: &[u8]) -> Result<Vec<u8>> {
    std::fs::write("/dev/attestation/user_report_data", data)?;
    Ok(std::fs::read("/dev/attestation/quote")?)
}

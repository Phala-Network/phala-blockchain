use anyhow::{Context, Result};
use clap::Parser;
use phactory_api::pruntime_client;
use phala_podauth as auth;
use sp_core::crypto::Pair;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The RPC endpoint of the pruntime
    #[clap(long, default_value = "http://localhost:8000")]
    pruntime_url: String,

    /// The the contract id to do the key distribution
    #[clap(long)]
    contract_id: String,

    /// The path to store the private key
    #[clap(long)]
    save_key_to: String,

    /// The path to store the certificate
    #[clap(long)]
    save_cert_to: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let private_key = pruntime_client::gen_key();
    let pubkey = private_key.public();
    let quote = create_quote_vec(&pubkey).context("Unable to crate quote")?;
    let request = auth::Request::Auth { quote };

    let contract_id: [u8; 32] = TryFrom::try_from(&try_decode_hex(&args.contract_id)?[..])?;

    let response: Result<auth::Response, auth::QueryError> = pruntime_client::query(
        args.pruntime_url,
        contract_id.into(),
        request,
        &private_key,
    )
    .await?;
    let response = response.map_err(|e| anyhow::anyhow!("Auth failed: {:?}", e))?;
    match response {
        auth::Response::Auth { cert, key } => {
            std::fs::write(&args.save_cert_to, &cert).context("Unable to write the cert file")?;
            std::fs::write(&args.save_key_to, &key).context("Unable to write the key file")?;
        }
        auth::Response::RootCert { .. } => anyhow::bail!("Invalid contract response"),
    }
    Ok(())
}

pub fn create_quote_vec(data: &[u8]) -> Result<Vec<u8>> {
    std::fs::write("/dev/attestation/user_report_data", data)?;
    Ok(std::fs::read("/dev/attestation/quote")?)
}

fn try_decode_hex(hex_str: &str) -> Result<Vec<u8>, hex::FromHexError> {
    hex::decode(hex_str.strip_prefix("0x").unwrap_or(hex_str))
}

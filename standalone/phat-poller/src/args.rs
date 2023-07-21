use clap::{Parser, Subcommand};
use phala_clap_parsers::parse_duration;
use sp_core::{crypto::SecretStringError, Pair, H256};
use std::time::Duration;

#[derive(Parser)]
#[clap(about = "Cache server for relaychain headers", version, author)]
pub struct AppArgs {
    #[clap(subcommand)]
    pub action: Action,
}

#[derive(Subcommand)]
pub enum Action {
    /// Run
    Run(RunArgs),
}

#[derive(Parser)]
pub struct RunArgs {
    /// The URI of a phala-node
    #[arg(long, default_value = "ws://localhost:9944")]
    pub node_uri: String,

    /// The worker URL used to development
    #[arg(long)]
    pub dev_worker_uri: Option<String>,

    /// Cluster ID
    #[arg(long, value_parser = parse_hash, default_value = "0x0000000000000000000000000000000000000000000000000000000000000001")]
    pub cluster_id: H256,

    /// Max number of history tasks to keep in
    #[arg(long, default_value = "200")]
    pub max_history: usize,

    /// The interval to update and probe the worker list
    #[arg(long, value_parser = parse_duration, default_value = "5m")]
    pub probe_interval: Duration,

    /// The timeout to probe a worker
    #[arg(long, value_parser = parse_duration, default_value = "5s")]
    pub probe_timeout: Duration,

    /// Contract poll interval
    #[arg(long, value_parser = parse_duration, default_value = "10s")]
    pub poll_interval: Duration,

    /// Contract poll timeout
    #[arg(long, value_parser = parse_duration, default_value = "10s")]
    pub poll_timeout: Duration,

    /// Contract poll timeout over all contracts
    #[arg(long, value_parser = parse_duration, default_value = "20s")]
    pub poll_timeout_overall: Duration,

    /// Top n workers to be used to poll
    #[arg(long, default_value_t = 5)]
    pub use_top_workers: usize,

    /// Contract ID for the BricksProfileFactory to poll
    #[arg(long, value_parser = parse_hash)]
    pub factory_contract: H256,

    /// Contract caller SR25519 private key mnemonic, private key seed, or derive path
    #[arg(long, value_parser = parse_sr25519, default_value = "//Alice")]
    pub caller: sp_core::sr25519::Pair,
}

fn parse_sr25519(s: &str) -> Result<sp_core::sr25519::Pair, SecretStringError> {
    <sp_core::sr25519::Pair as Pair>::from_string(s, None)
}

pub fn parse_hash(s: &str) -> Result<H256, hex::FromHexError> {
    let s = s.trim_start_matches("0x");
    if s.len() != 64 {
        return Err(hex::FromHexError::InvalidStringLength);
    }
    let mut buf = [0u8; 32];
    hex::decode_to_slice(s, &mut buf)?;
    Ok(H256::from(buf))
}

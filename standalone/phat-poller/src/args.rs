use clap::{Parser, Subcommand};
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

#[derive(Debug)]
pub struct InvalidDuration;

impl std::fmt::Display for InvalidDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid duration")
    }
}

impl std::error::Error for InvalidDuration {}

fn parse_sr25519(s: &str) -> Result<sp_core::sr25519::Pair, SecretStringError> {
    <sp_core::sr25519::Pair as Pair>::from_string(s, None)
}

fn parse_duration(s: &str) -> Result<Duration, InvalidDuration> {
    let mut num_str = s;
    let mut unit = "s";

    if let Some(idx) = s.find(|c: char| !c.is_numeric()) {
        num_str = &s[..idx];
        unit = &s[idx..];
    }

    let num = num_str.parse::<u64>().or(Err(InvalidDuration))?;

    let num = match unit {
        "s" | "" => num,
        "m" => num * 60,
        "h" => num * 60 * 60,
        "d" => num * 60 * 60 * 24,
        _ => return Err(InvalidDuration),
    };

    Ok(Duration::from_secs(num))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("10").unwrap(), Duration::from_secs(10));
        assert_eq!(parse_duration("10s").unwrap(), Duration::from_secs(10));
        assert_eq!(parse_duration("10m").unwrap(), Duration::from_secs(600));
        assert_eq!(parse_duration("10h").unwrap(), Duration::from_secs(36000));
        assert_eq!(parse_duration("10d").unwrap(), Duration::from_secs(864000));
        assert_eq!(parse_duration("1").unwrap(), Duration::from_secs(1));
        assert_eq!(parse_duration("100").unwrap(), Duration::from_secs(100));
    }

    #[test]
    fn test_parse_duration_invalid() {
        assert!(parse_duration("10x").is_err());
        assert!(parse_duration("ms").is_err());
    }

    #[test]
    fn test_parse_duration_empty() {
        assert!(parse_duration("").is_err());
    }
}

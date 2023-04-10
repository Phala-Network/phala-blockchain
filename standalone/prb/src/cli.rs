use crate::configurator;
use crate::wm::wm;
use clap::{Parser, Subcommand};
use log::debug;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Clone)]
#[command(name="prb", version, about="Phala Runtime Bridge Worker Manager", long_about = None)]
pub struct WorkerManagerCliArgs {
    /// Path to the local database
    #[arg(short = 'd', long, env, default_value = "/var/data/prb-wm")]
    pub db_path: String,

    #[arg(short = 's', long, env, default_value = "/var/data/prb-wm/ds.yml")]
    pub data_source_config_path: String,

    /// Listen address of management interface
    #[arg(short = 'm', long, env, default_values_t = vec!["0.0.0.0:3001".to_string(), "[::]:3001".to_string()])]
    pub mgmt_listen_addresses: Vec<String>,

    /// Enable mDNS broadcast of management interface information
    #[arg(long, env)]
    pub mgmt_disable_mdns: bool,

    /// Disable fast-sync feature
    #[arg(long, env)]
    pub disable_fast_sync: bool,

    /// Size of in-memory cache, default to 1 GiB
    #[arg(short = 'c', long, env, default_value_t = 1073741824)]
    pub cache_size: usize,

    /// URL of webhook endpoint
    #[arg(short = 'w', long, env)]
    pub webhook_url: Option<String>,
}

pub async fn start_wm() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp_micros()
        .parse_default_env()
        .init();
    wm(WorkerManagerCliArgs::parse()).await
}

#[derive(Parser, Debug)]
#[command(name="prb", version, about="Phala Runtime Bridge Worker Manager", long_about = None)]
pub struct ConfigCliArgs {
    /// Path to the local database
    #[arg(short = 'd', long, env, default_value = "/var/data/prb-wm")]
    pub db_path: String,

    #[command(subcommand)]
    pub(crate) command: ConfigCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ConfigCommands {
    /// Add a pool
    AddPool {
        /// Name of the pool
        #[arg(short, long)]
        name: String,

        /// Pool pid
        #[arg(short, long)]
        pid: u64,

        /// Whether workers belongs to the pool are disabled
        #[arg(short, long, default_value_t = false)]
        disabled: bool,

        /// Whether workers belongs to the pool should be in sync-only mode
        #[arg(short, long, default_value_t = false)]
        sync_only: bool,
    },

    /// Remove a pool,
    RemovePool {
        /// Pool pid
        #[arg(short, long)]
        pid: u64,
    },

    /// Update a pool,
    UpdatePool {
        /// Name of the pool
        #[arg(short, long)]
        name: String,

        /// Pool pid
        #[arg(short, long)]
        pid: u64,

        /// Whether workers belongs to the pool are disabled
        #[arg(short, long, default_value_t = false)]
        disabled: bool,

        /// Whether workers belongs to the pool should be in sync-only mode
        #[arg(short, long, default_value_t = false)]
        sync_only: bool,
    },

    /// Get a pool,
    GetPool {
        /// Pool pid
        #[arg(short, long)]
        pid: u64,
    },

    /// Get a pool with all workers belonged to
    GetPoolWithWorkers {
        /// Pool pid
        #[arg(short, long)]
        pid: u64,
    },

    /// Get all pools,
    GetAllPools,

    /// Get all pools with workers,
    GetAllPoolsWithWorkers,

    /// Add a worker
    AddWorker {
        /// Name of the worker
        #[arg(short, long)]
        name: String,

        /// HTTP endpoint to the worker
        #[arg(short, long)]
        endpoint: String,

        /// Stake amount in BN String
        #[arg(short = 't', long)]
        stake: String,

        /// Pool pid
        #[arg(short, long)]
        pid: u64,

        /// Whether the worker is disabled
        #[arg(short, long, default_value_t = false)]
        disabled: bool,

        /// Whether the worker should be in sync-only mode
        #[arg(short, long, default_value_t = false)]
        sync_only: bool,

        /// Whether the worker should be a gatekeeper
        #[arg(short, long, default_value_t = false)]
        gatekeeper: bool,
    },

    /// Update a worker
    UpdateWorker {
        /// Current name of the worker
        #[arg(short, long)]
        name: String,

        /// New name of the worker
        #[arg(long)]
        new_name: Option<String>,

        /// HTTP endpoint to the worker
        #[arg(short, long)]
        endpoint: String,

        /// Stake amount in BN String
        #[arg(short = 't', long)]
        stake: String,

        /// Pool pid
        #[arg(short, long)]
        pid: u64,

        /// Whether the worker is disabled
        #[arg(short, long, default_value_t = false)]
        disabled: bool,

        /// Whether the worker should be in sync-only mode
        #[arg(short, long, default_value_t = false)]
        sync_only: bool,

        /// Whether the worker should be a gatekeeper
        #[arg(short, long, default_value_t = false)]
        gatekeeper: bool,
    },

    /// Remove a worker
    RemoveWorker {
        /// UUID of the worker
        #[arg(short, long)]
        name: String,
    },

    /// Get all pool operators
    GetAllPoolOperators,

    /// Get a pool operators by pid
    GetPoolOperator {
        /// PID of the pool
        #[arg(short, long)]
        pid: u64,
    },

    /// Set a pool operators for pool
    SetPoolOperator {
        /// PID of the pool
        #[arg(short, long)]
        pid: u64,

        /// Account string of the operator, can be either a mnemonic or a seed, learn more: https://docs.rs/sp-core/latest/sp_core/crypto/trait.Pair.html#method.from_string_with_seed
        #[arg(short, long)]
        account: String,

        /// Proxied pool owner account in SS58 format
        #[arg(short = 'x', long)]
        proxied_account_id: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
struct CliErrorMessage {
    message: String,
    backtrace: String,
}

pub async fn start_config() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp_micros()
        .parse_default_env()
        .init();
    match configurator::cli_main(ConfigCliArgs::parse()).await {
        Ok(_) => {}
        Err(e) => {
            debug!("{}\n{}", &e, e.backtrace());
            let ce = CliErrorMessage {
                message: format!("{}", &e),
                backtrace: format!("{}", e.backtrace()),
            };
            println!("{}", serde_json::to_string_pretty(&ce).unwrap())
        }
    }
}

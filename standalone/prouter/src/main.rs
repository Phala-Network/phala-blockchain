#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

mod types;
mod config;
mod utils;

use anyhow::{Result, Error};
#[allow(unused_imports)]
use log::{debug, error, info, warn};
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use structopt::StructOpt;
use tokio::signal;
use tokio::time::{sleep, Duration};
use tokio::select;

use chrono::{DateTime, Utc};

use types::I2PD;
use config::*;
use utils::*;

#[derive(Debug, StructOpt)]
#[structopt(name = "prouter")]
struct Args {
    #[structopt(
        default_value = "http://localhost:8000",
        long,
        help = "pRuntime http endpoint"
    )]
    pruntime_endpoint: String,

    #[structopt(long, default_value = "600")]
    graceful_shutdown_interval: u64,

    #[structopt(long, default_value = "./pdata")]
    prouter_datadir: String,
}

fn preprocess_path(path_str: &String) -> Result<PathBuf> {
    // Check path exists
    let path = Path::new(&path_str);
    if !path.exists() {
        fs::create_dir(&path)?;
    }
    // Convert to absolute path
    let absolute_path = path
        .canonicalize()
        .expect("Any path should always be able to converted into a absolute path");

    Ok(absolute_path)
}

async fn display_prouter_info(i2pd: &I2PD) -> Result<()> {
    let client_tunnels_info = i2pd.get_client_tunnels_info()?;
    let server_tunnels_info = i2pd.get_server_tunnels_info()?;
    info!("Client Tunnels Count: {}, Server Tunnels Count: {}", client_tunnels_info.len(), server_tunnels_info.len());
    info!("Client Tunnels:");
    for (i, client_tun) in client_tunnels_info.iter().enumerate() {
        info!("\t\u{1F3AF}{} => {}", client_tun.0, client_tun.1);
    }
    info!("Server Tunnels:");
    for (i, server_tun) in server_tunnels_info.iter().enumerate() {
        info!("\t\u{1F91D} {} <= {}", server_tun.0, server_tun.1);
    }

    loop {
        sleep(Duration::from_secs(10)).await;
    }
    // Will never return
}

pub async fn prouter_main() -> Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    let args = Args::from_args();

    // init path
    let datadir = preprocess_path(&args.prouter_datadir)?;

    // init conf
    init_prouter_conf(&datadir)?;
    init_tunnels_conf(&datadir)?;

    // init I2PD
    let mut i2pd = I2PD::new("PRouter".parse()?);
    i2pd.add_config("datadir".parse()?, pathbuf_to_string(datadir.clone())?);
    i2pd.add_config(
        "conf".parse()?,
        get_relative_filepath_str(&datadir, "i2pd.conf")?,
    );

    let pnetwork_identity = i2pd.load_private_keys_from_file(true)?;

    info!("PRouter is initializing...");
    i2pd.init();
    info!("PRouter is starting...");
    i2pd.start();
    info!("PRouter is successfully started, logging into {}", get_relative_filepath_str(&datadir, "prouter.log")?);
    info!("Press CTRL-C to gracefully shutdown");
    info!(" ");

    select! {
        _ = signal::ctrl_c() => {},
        _ = display_prouter_info(&i2pd) => {},
    };

    info!("PRouter is gracefully shutting down...");
    i2pd.close_accepts_tunnels();
    info!("Accepts tunnels are closed");

    let now: DateTime<Utc> = Utc::now();
    let sleep = sleep(Duration::from_secs(args.graceful_shutdown_interval));
    info!("");
    info!("\u{23F3} Shutting down after {} seconds [from {}]", &args.graceful_shutdown_interval, now.to_rfc2822());
    info!("");

    select! {
        _ = sleep => {
            info!("Shutting down...");
        }
        _ = signal::ctrl_c() => {
            info!("Force shutting down...");
        },
    };

    i2pd.stop();
    info!("PRouter is successfully stopped");
    Ok(())
}

#[tokio::main]
async fn main() {
    match prouter_main().await {
        Ok(()) => {}
        Err(e) => panic!("Fetal error: {:?}", e),
    };
}

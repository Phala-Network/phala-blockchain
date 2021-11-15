#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

mod config;
mod types;
mod utils;

use anyhow::{Error, Result};
#[allow(unused_imports)]
use log::{debug, error, info, warn};
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use structopt::StructOpt;
use tokio::select;
use tokio::signal;
use tokio::time::{sleep, Duration};

use chrono::{DateTime, Utc};

use config::*;
use types::I2PD;
use utils::*;

#[derive(Debug, StructOpt)]
#[structopt(name = "prouter")]
struct Args {
    #[structopt(
        default_value = "127.0.0.1",
        long
    )]
    pruntime_host: String,

    #[structopt(
        default_value = "8000",
        long
    )]
    pruntime_port: String,

    #[structopt(long, default_value = "600")]
    shutdown_interval: u64,

    #[structopt(long, default_value = "./pdata")]
    datadir: String,

    #[structopt(long)]
    ignore_pnetwork: bool,

    #[structopt(
        long,
        help = "Your custom tunnels file that contains other tunnels. PRouter will merge your tunnels with the Phala Network tunnel."
    )]
    existed_tunconf: Option<String>,
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
    let http_proxy_info = i2pd.get_http_proxy_info()?;
    let socks_proxy_info = i2pd.get_socks_proxy_info()?;
    let server_tunnels_info = i2pd.get_server_tunnels_info()?;
    info!(
        "📦 Client Tunnels Count: {}, Server Tunnels Count: {}",
        client_tunnels_info.len(),
        server_tunnels_info.len()
    );
    info!("🚇 Client Tunnels:");
    if let Ok(http_proxy_tun) = i2pd.get_http_proxy_info() {
        info!("\t✅ {} => {}", http_proxy_tun.0, http_proxy_tun.1);
    }
    if let Ok(socks_proxy_tun) = i2pd.get_socks_proxy_info() {
        info!("\t✅ {} => {}", socks_proxy_tun.0, socks_proxy_tun.1);
    }
    for (i, client_tun) in client_tunnels_info.iter().enumerate() {
        info!("\t✅ {} => {}", client_tun.0, client_tun.1);
    }
    info!("🚇 Server Tunnels:");
    for (i, server_tun) in server_tunnels_info.iter().enumerate() {
        info!("\t✅ {} <= {}", server_tun.0, server_tun.1);
    }

    loop {
        let network_status = i2pd.get_network_status()?;
        info!("💡 Network Status: {}", network_status);
        let tunnel_creation_success_rate = i2pd.get_tunnel_creation_success_rate()?;
        info!(
            "🛠 Tunnel Creation Success Rate: {}%",
            tunnel_creation_success_rate
        );
        let received_byte = i2pd.get_received_byte()?;
        let in_bandwidth = i2pd.get_in_bandwidth()?;
        info!(
            "📥 Received Bytes: {} ({})",
            format_traffic(received_byte)?,
            format_bandwidth(in_bandwidth)?
        );
        let sent_byte = i2pd.get_sent_byte()?;
        let out_bandwidth = i2pd.get_out_bandwidth()?;
        info!(
            "📤 Sent Bytes: {} ({})",
            format_traffic(sent_byte)?,
            format_bandwidth(out_bandwidth)?
        );
        let transit_byte = i2pd.get_transit_byte()?;
        let transit_bandwidth = i2pd.get_transit_bandwidth()?;
        info!(
            "👺 Transit Bytes: {} ({})",
            format_traffic(transit_byte)?,
            format_bandwidth(transit_bandwidth)?
        );
        let httpproxy_enabled = i2pd.is_httpproxy_enabled()?;
        info!(
            "🤝 HTTP Proxy: {}",
            if httpproxy_enabled {
                "Enabled 🟢"
            } else {
                "Disabled 🔴"
            }
        );
        let socksproxy_enabled = i2pd.is_socksproxy_enabled()?;
        info!(
            "🤝 SOCKS Proxy: {}",
            if socksproxy_enabled {
                "Enabled 🟢"
            } else {
                "Disabled 🔴"
            }
        );
        let bob_enabled = i2pd.is_bob_enabled()?;
        info!("🤝 BOB: {}", if bob_enabled { "Enabled 🟢" } else { "Disabled 🔴" });
        let sam_enabled = i2pd.is_sam_enabled()?;
        info!("🤝 SAM: {}", if sam_enabled { "Enabled 🟢" } else { "Disabled 🔴" });
        let i2cp_enabled = i2pd.is_i2cp_enabled()?;
        info!(
            "🤝 I2CP: {}",
            if i2cp_enabled { "Enabled 🟢" } else { "Disabled 🔴" }
        );
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
    let datadir = preprocess_path(&args.datadir)?;

    // init conf
    init_prouter_conf(&datadir)?;
    init_tunnels_conf(&datadir, &args.existed_tunconf, &args.pruntime_host, &args.pruntime_port, &args.ignore_pnetwork)?;

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
    info!(
        "PRouter is successfully started, logging into {}",
        get_relative_filepath_str(&datadir, "prouter.log")?
    );
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
    let sleep = sleep(Duration::from_secs(args.shutdown_interval));
    info!("");
    info!(
        "⏳ Shutting down after {} seconds [from {}]",
        &args.shutdown_interval,
        now.to_rfc2822()
    );
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

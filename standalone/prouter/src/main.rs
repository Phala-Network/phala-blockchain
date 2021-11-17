#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

mod config;
mod i2pd;
mod utils;

use anyhow::{anyhow, Error, Result};
#[allow(unused_imports)]
use log::{debug, error, info, warn};
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use structopt::StructOpt;
use tokio::select;
use tokio::signal;
use tokio::time::{sleep, Duration};
extern crate rand;

use chrono::{DateTime, Utc};

use config::*;
use i2pd::I2PD;
use utils::*;

use crate::prpc::phactory_api_client::PhactoryApiClient;
use phactory_api::prpc::{self};
use phactory_api::pruntime_client;

#[derive(Debug, StructOpt)]
#[structopt(name = "prouter")]
pub struct Args {
    ///
    /// pRuntime Setting
    ///
    #[structopt(long, help = "Set and pRouter will work without pRuntime")]
    no_pruntime: bool,

    #[structopt(
        default_value = "http://localhost:8000",
        long,
        help = "Endpoint of pRuntime that pRouter could communicate with"
    )]
    pruntime_endpoint: String,

    ///
    /// Phala Node Setting
    ///
    #[structopt(
        long,
        help = "Set and pRouter will work without Phala node rpc websocket endpoint"
    )]
    no_pnode: bool,

    #[structopt(
        default_value = "http://localhost:8000",
        long,
        help = "Phala node rpc websocket endpoint"
    )]
    pnode_ws_endpoint: String,

    ///
    /// Phala Network Setting
    ///
    #[structopt(long, help = "Join Phala Network?")]
    join_pnetwork: bool,

    #[structopt(
        default_value = "127.0.0.1",
        long,
        help = "Host for the public API, without `http://` prefix. Required to support http protocol"
    )]
    pnetwork_host: String,

    #[structopt(
        default_value = "8000",
        long,
        help = "Port for the public API. Required to support http protocol"
    )]
    pnetwork_port: String,

    ///
    /// Router Setting
    ///
    #[structopt(
        long,
        default_value = "600",
        help = "Seconds of pRouter gracefully shutting down"
    )]
    shutdown_interval: u64,

    #[structopt(long, default_value = "./pdata", help = "Path to store pRouter data")]
    datadir: String,

    #[structopt(long, help = "Override default i2pd config file to provided")]
    override_i2pd: Option<String>,

    #[structopt(
        long,
        help = "Override default tunnels config file to provided. You need to prepare your own key file, path is relative to datadir"
    )]
    override_tun: Option<String>,

    #[structopt(long, help = "Auto restart self after an error occurred")]
    auto_restart: bool,

    #[structopt(
        default_value = "10",
        long,
        help = "Max auto restart retries if it continuously failing. Only used with --auto-restart"
    )]
    max_restart_retries: u32,
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
        info!("💡 Network Status: {}", &network_status);
        if network_status.to_lowercase().contains("error") {
            warn!("Error happened: {}", &network_status);
            return Err(anyhow!("{}", network_status));
        }
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
        info!(
            "🤝 BOB: {}",
            if bob_enabled {
                "Enabled 🟢"
            } else {
                "Disabled 🔴"
            }
        );
        let sam_enabled = i2pd.is_sam_enabled()?;
        info!(
            "🤝 SAM: {}",
            if sam_enabled {
                "Enabled 🟢"
            } else {
                "Disabled 🔴"
            }
        );
        let i2cp_enabled = i2pd.is_i2cp_enabled()?;
        info!(
            "🤝 I2CP: {}",
            if i2cp_enabled {
                "Enabled 🟢"
            } else {
                "Disabled 🔴"
            }
        );
        sleep(Duration::from_secs(10)).await;
    }
    // Will never return
}

pub async fn daemon_run(mut i2pd: I2PD, args: &Args) -> Result<()> {
    info!("PRouter is initializing...");
    i2pd.init();
    info!("PRouter is starting...");
    i2pd.start();
    info!("PRouter is successfully started");
    info!("Press CTRL-C to gracefully shutdown");
    info!(" ");

    select! {
        _ = signal::ctrl_c() => {},
        ret = display_prouter_info(&i2pd) => {
            return ret;
        },
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

pub async fn prouter_main(args: &Args) -> Result<()> {
    let mut pr: Option<PhactoryApiClient<pruntime_client::RpcRequest>> = None;
    let pnetwork_ident_sk: Vec<u8>;

    if !args.no_pruntime {
        pr = Some(pruntime_client::new_pruntime_client(
            args.pruntime_endpoint.clone(),
        ));
    }

    // generating conf
    let abs_datadir_path = preprocess_path(&args.datadir)?;
    let tunconf_path = init_tunnels_conf(&abs_datadir_path, args)?;
    let conf_path = init_prouter_conf(&abs_datadir_path, tunconf_path, args)?;

    // initializing I2PD
    let mut i2pd = I2PD::new("PRouter".parse()?);
    i2pd.add_config("conf".parse()?, conf_path);

    if args.join_pnetwork {
        if !args.no_pruntime {
            // Wait until pruntime is initialized
            loop {
                let info = pr
                    .as_ref()
                    .expect("guaranteed to be initialized")
                    .get_info(())
                    .await?;
                if !info.initialized {
                    warn!("pRuntime is not initialized. Waiting...");
                } else {
                    info!("pRuntime already initialized.");
                    break;
                }
                sleep(Duration::from_secs(5)).await;
            }
            // Get derived identity for pnetwork
            pnetwork_ident_sk = pr
                .expect("guaranteed to be initialized")
                .derive_ident_sk(prpc::DeriveIdentSkRequest {
                    info: b"pNetwork".to_vec(),
                })
                .await?
                .sk;
        } else {
            // Random generate
            pnetwork_ident_sk = (0..64).map(|_| rand::random::<u8>()).collect();
        }
        i2pd::generate_ident_to_file(
            &abs_datadir_path,
            "pnetwork.key".to_string(),
            pnetwork_ident_sk,
        )?;
    }

    let mut restart_failure_count: u32 = 0;
    loop {
        if let Err(err) = daemon_run(i2pd.clone(), args).await {
            info!("daemon_run() exited with error: {:?}", err);
            if !args.auto_restart || restart_failure_count > args.max_restart_retries {
                std::process::exit(1);
            }
            restart_failure_count += 1;
            sleep(Duration::from_secs(2)).await;
            info!("Restarting...");
        } else {
            break;
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    let args = Args::from_args();

    match prouter_main(&args).await {
        Ok(()) => {}
        Err(e) => panic!("Fetal error: {:?}", e),
    };
}

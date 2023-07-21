mod config;
mod i2pd;
mod server;
mod translator;
mod utils;

use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Result};
use log::{error, info, warn};
use std::fs;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use tokio::time::{sleep, Duration};
use tokio::{select, signal};

use phaxt::ParachainApi;

use chrono::{DateTime, Utc};

use config::*;
use i2pd::I2pd;
use utils::*;

use crate::prpc::phactory_api_client::PhactoryApiClient;
use phactory_api::prpc;
use phactory_api::pruntime_client;
use phaxt::rpc::ExtraRpcExt;

use phactory_api::endpoints::EndpointType;

pub type SharedParachainApi = Arc<Mutex<Option<ParachainApi>>>;

#[derive(Debug, StructOpt)]
#[structopt(name = "prouter")]
pub struct Args {
    // pRuntime Setting
    #[structopt(long, help = "Set and pRouter will work without pRuntime")]
    no_pruntime: bool,

    #[structopt(
        default_value = "http://localhost:8000",
        long,
        help = "Endpoint of pRuntime that pRouter could communicate with"
    )]
    pruntime_endpoint: String,

    // Phala Node Setting
    #[structopt(
        long,
        help = "Set and pRouter will work without Phala node rpc websocket endpoint"
    )]
    no_pnode: bool,

    #[structopt(
        default_value = "ws://localhost:9944",
        long,
        help = "Parachain rpc websocket endpoint"
    )]
    substrate_ws_endpoint: String,

    #[structopt(long, help = "Don't wait the substrate nodes to sync blocks")]
    no_wait: bool,

    // Provide custom public endpoint. Will disable i2p router if this field is provided
    #[structopt(long, help = "Specify the type of the endpoint", default_value = "i2p")]
    endpoint_type: EndpointType,

    #[structopt(long, help = "Provide custom public endpoint. E.g. http://xxx:3333")]
    endpoint: Option<String>,

    // PRouter Settings
    #[structopt(long, help = "Auto restart self after an error occurred")]
    auto_restart: bool,

    #[structopt(
        default_value = "10",
        long,
        help = "Max auto restart retries if it continuously failing. Only used with --auto-restart"
    )]
    max_restart_retries: u32,

    // I2P Router Setting, Only effective with I2P endpoint
    #[structopt(
        long,
        default_value = "600",
        help = "Seconds of pRouter gracefully shutting down"
    )]
    shutdown_interval: u64,

    #[structopt(
        long,
        default_value = "./prouter_data",
        help = "Path to store pRouter data"
    )]
    datadir: String,

    #[structopt(long, help = "Override default i2pd config file provided")]
    override_i2pd: Option<String>,

    #[structopt(
        long,
        help = "Override default tunnels config file to provided. You need to prepare your own key file, path is relative to datadir"
    )]
    override_tun: Option<String>,

    #[structopt(
        default_value = "127.0.0.1",
        long,
        help = "Host for the public API, without `http://` prefix. Required to support http protocol"
    )]
    exposed_address: String,

    #[structopt(
        default_value = "8001",
        long,
        help = "Port for the public API. Required to support http protocol"
    )]
    exposed_port: u16,

    #[structopt(
        default_value = "127.0.0.1",
        long,
        help = "Host for the pRouter server, without `http://` prefix. Required to support http protocol"
    )]
    server_address: String,

    #[structopt(
        default_value = "8100",
        long,
        help = "Port for the pRouter server. Required to support http protocol"
    )]
    server_port: u16,
}

fn preprocess_path(path_str: &String) -> Result<PathBuf> {
    // Check path exists
    let path = Path::new(&path_str);
    if !path.exists() {
        fs::create_dir(&path)?;
    }
    // Convert to absolute path for i2pd config
    let absolute_path = path
        .canonicalize()
        .expect("Any path should always be able to converted into a absolute path");

    Ok(absolute_path)
}

async fn display_prouter_info(i2pd: &I2pd) -> Result<()> {
    let client_tunnels_info = i2pd.get_client_tunnels_info()?;
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
        info!("\t✅ {} {} => {}", i, client_tun.0, client_tun.1);
    }
    info!("🚇 Server Tunnels:");
    for (i, server_tun) in server_tunnels_info.iter().enumerate() {
        info!("\t✅ {} {} <= {}", i, server_tun.0, server_tun.1);
    }

    let mut network_retry = 0;

    loop {
        let network_status = i2pd.get_network_status()?;
        info!("💡 Network Status: {}", &network_status);
        if network_status.to_lowercase().contains("error") {
            warn!("Error happened: {}", &network_status);
            network_retry += 1;
            if network_retry > 3 {
                return Err(anyhow!("{}", network_status));
            }
            i2pd.run_peer_test();
        } else if !network_status.to_lowercase().contains("testing") {
            network_retry = 0;
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

        fn log_feature_enabled(feature: &str, enabled: bool) {
            info!(
                "{}: {}",
                feature,
                if enabled {
                    "Enabled 🟢"
                } else {
                    "Disabled 🔴"
                }
            );
        }

        let httpproxy_enabled = i2pd.is_httpproxy_enabled()?;
        log_feature_enabled("🤝 HTTP Proxy", httpproxy_enabled);

        let socksproxy_enabled = i2pd.is_socksproxy_enabled()?;
        log_feature_enabled("🤝 SOCKS Proxy", socksproxy_enabled);

        let bob_enabled = i2pd.is_bob_enabled()?;
        log_feature_enabled("🤝 Bob", bob_enabled);

        let sam_enabled = i2pd.is_sam_enabled()?;
        log_feature_enabled("🤝 Sam", sam_enabled);

        let i2cp_enabled = i2pd.is_i2cp_enabled()?;
        log_feature_enabled("🤝 I2CP", i2cp_enabled);

        let inbound_tunnels_count = i2pd.get_inbound_tunnels_count()?;
        info!("✨ {} Inbound tunnels", &inbound_tunnels_count);
        for index in 0..inbound_tunnels_count {
            let raw_info = i2pd.get_inbound_tunnel_formatted_info(index)?;
            info!("\t⛓ {}", raw_info.replace("&#8658;", "->"));
        }

        let outbound_tunnels_count = i2pd.get_outbound_tunnels_count()?;
        info!("✨ {} Outbound tunnels", &outbound_tunnels_count);
        for index in 0..outbound_tunnels_count {
            let raw_info = i2pd.get_outbound_tunnel_formatted_info(index)?;
            info!("\t⛓ {}", raw_info.replace("&#8658;", "->"));
        }

        sleep(Duration::from_secs(10)).await;
    }
    // Will never return
}

pub async fn daemon_run(mut i2pd: I2pd, args: &Args) -> Result<()> {
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
            if ret.is_err() {
                i2pd.stop();
                return ret;
            }
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

async fn wait_until_synced(client: &phaxt::RpcClient) -> Result<()> {
    loop {
        let state = client.extra_rpc().system_sync_state().await?;
        info!(
            "Checking synced: current={} highest={:?}",
            state.current_block, state.highest_block
        );
        if let Some(highest) = state.highest_block {
            if highest - state.current_block <= 2 {
                return Ok(());
            }
        }
        sleep(Duration::from_secs(5)).await;
    }
}

pub async fn prouter_daemon(args: &Args, i2pd: &I2pd) -> Result<()> {
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

pub async fn prouter_main(args: &Args) -> Result<(String, I2pd)> {
    let mut i2pd = I2pd::new("PRouter".parse()?);
    let mut local_proxy: String = Default::default();
    {
        let mut pr: Option<PhactoryApiClient<pruntime_client::RpcRequest>> = None;
        let endpoint: String;
        let mut no_bind: bool = args.no_pnode;

        // connect to pruntime (need pruntime to be initialized first)
        if !args.no_pruntime {
            pr = Some(pruntime_client::new_pruntime_client(
                args.pruntime_endpoint.clone(),
            ));
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
        }

        // Start i2pd
        // 1. generate config files
        let abs_datadir_path = preprocess_path(&args.datadir)?;
        let tunconf_path = init_tunnels_conf(&abs_datadir_path, args)?;
        let conf_path;
        (conf_path, local_proxy) = init_prouter_conf(&abs_datadir_path, tunconf_path, args)?;
        // 2. get endpoint
        let phala_i2p_key: Vec<u8>;
        match args.endpoint_type {
            EndpointType::I2p => {
                if args.no_pruntime {
                    // Random generate, it will not go on-chain
                    no_bind = true;
                    info!("No pRuntime, using random pubkey instead");
                    phala_i2p_key = (0..64).map(|_| rand::random::<u8>()).collect();
                } else {
                    if !args.no_pruntime {
                        phala_i2p_key = pr
                            .as_ref()
                            .expect("guaranteed to be initialized")
                            .derive_phala_i2p_key(())
                            .await?
                            .phala_i2p_key;
                    } else {
                        error!("No pRuntime but i2p endpoint is used!");
                        return Err(anyhow!("No pRuntime but i2p endpoint is used"));
                    }
                }
                let i2p_endpoint = i2pd::generate_ident_to_file(
                    &abs_datadir_path,
                    "phala.key".to_string(),
                    phala_i2p_key,
                )?;
                endpoint = format!("{}:{}", i2p_endpoint, args.exposed_port);
            }
            EndpointType::Http => {
                endpoint = args.endpoint.clone().expect("--endpoint is required");
            }
        }
        // 3. initializing i2pd
        i2pd.add_config(
            "datadir".parse()?,
            pathbuf_to_string(abs_datadir_path.clone())?,
        );
        i2pd.add_config("conf".parse()?, conf_path.clone());
        // 4. register endpoint to pRuntime
        if !no_bind {
            info!("Binding Endpoint: {}", endpoint);
            let add_endpoint_request =
                prpc::AddEndpointRequest::new(args.endpoint_type.clone(), endpoint);
            pr.as_ref()
                .expect("guaranteed to be initialized")
                .add_endpoint(add_endpoint_request)
                .await?;
        }
    }

    Ok((local_proxy.clone(), i2pd.clone()))
}

fn check_args(args: &Args) -> Result<()> {
    if matches!(args.endpoint_type, EndpointType::Http) {
        if args.endpoint.is_none() {
            return Err(anyhow!("Custom endpoint is required for http endpoint"));
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
    check_args(&args).expect("Args should be valid");

    let para_api: SharedParachainApi = if args.no_pnode {
        Arc::new(Mutex::new(None))
    } else {
        let para_uri: &str = &args.substrate_ws_endpoint;
        let connected_para_api: ParachainApi = phaxt::connect(para_uri)
            .await
            .expect("should connect to parachain")
            .into();
        info!("Connected to parachain node at: {}", para_uri);
        if !args.no_wait {
            // Don't start the router until the substrate node is synced
            info!("Waiting for substrate to sync blocks...");
            wait_until_synced(&connected_para_api)
                .await
                .expect("should wait for sync");
            info!("Substrate sync blocks done");
        }

        Arc::new(Mutex::new(Some(connected_para_api)))
    };

    let (local_proxy, i2pd) = prouter_main(&args)
        .await
        .expect("prouter_main should be ok");
    let server_address = args.server_address.clone();
    let server_port = args.server_port.clone();

    select! {
        _ = server::spawn_socks_server(local_proxy, para_api.clone(), server_address, server_port) => {},
        ret = prouter_daemon(&args, &i2pd) => {
            match ret {
                Ok(_) => {
                    std::process::exit(0);
                },
                Err(e) => panic!("Fetal error: {:?}", e),
            }
        },
    };
}

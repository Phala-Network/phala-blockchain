#![feature(decl_macro)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
extern crate rocket_cors;
#[macro_use]
extern crate lazy_static;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

mod config;
mod i2pd;
mod reseeder;
mod server;
mod translator;
mod types;
mod utils;

use std::sync::{Arc, Mutex};
use std::thread;

use anyhow::{anyhow, Context, Error, Result};
use codec::Decode;
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

use phaxt::{subxt, ParachainApi, RelaychainApi};
use sp_core::{crypto::Pair, sr25519, storage::StorageKey};

use chrono::{DateTime, Utc};

use config::*;
use i2pd::I2PD;
use phaxt::subxt::Signer;
use reseeder::SU3File;
use utils::*;

use crate::prpc::phactory_api_client::PhactoryApiClient;
use phactory_api::prpc::{self};
use phactory_api::pruntime_client;
use phaxt::rpc::ExtraRpcExt;

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
        default_value = "ws://localhost:9944",
        long,
        help = "Substrate rpc websocket endpoint"
    )]
    substrate_ws_endpoint: String,

    #[structopt(
        default_value = "ws://localhost:9977",
        long,
        help = "Parachain collator rpc websocket endpoint"
    )]
    collator_ws_endpoint: String,

    #[structopt(long = "parachain", help = "Parachain mode")]
    parachain: bool,

    #[structopt(long, help = "Don't wait the substrate nodes to sync blocks")]
    no_wait: bool,

    #[structopt(
        required = true,
        default_value = "//Alice",
        short = "m",
        long = "mnemonic",
        help = "Controller SR25519 private key mnemonic, private key seed, or derive path"
    )]
    mnemonic: String,

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

lazy_static! {
    static ref API: Arc<Mutex<Option<RelaychainApi>>> = Arc::new(Mutex::new(None));
    static ref PARA_API: Arc<Mutex<Option<ParachainApi>>> = Arc::new(Mutex::new(None));
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

async fn wait_until_synced<T: subxt::Config>(client: &subxt::Client<T>) -> Result<()> {
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

pub async fn subxt_connect<T: subxt::Config>(uri: &str) -> Result<subxt::Client<T>> {
    subxt::ClientBuilder::new()
        .set_url(uri)
        .build()
        .await
        .context("Connect to substrate")
}

/// Updates the nonce from the mempool
pub async fn update_signer_nonce(
    api: &ParachainApi,
    signer: &mut subxt::PairSigner<phaxt::KhalaConfig, sr25519::Pair>,
) -> Result<()> {
    let account_id = signer.account_id().clone();
    let nonce = api.client.extra_rpc().account_nonce(&account_id).await?;
    signer.set_nonce(nonce);
    log::info!("Fetch account {} nonce={}", account_id, nonce);
    Ok(())
}

async fn bind_worker_pnetwork_ident(
    para_api: &ParachainApi,
    pubkey: &phaxt::khala::runtime_types::sp_core::sr25519::Public,
    signer: &mut subxt::PairSigner<phaxt::KhalaConfig, sr25519::Pair>,
    pnetwork_ident: &String,
) -> Result<()> {
    let data = phaxt::khala::runtime_types::phala_types::PhalaNetworkIdentBindingInfo {
        pubkey: phaxt::khala::runtime_types::sp_core::sr25519::Public(pubkey.0),
        pnetwork_ident: pnetwork_ident.clone(),
        version: 0,
    };
    update_signer_nonce(para_api, signer).await?;
    let ret = para_api
        .tx()
        .phala_registry()
        .bind_worker_pnetwork_ident(data)
        .sign_and_submit_then_watch(signer)
        .await;
    if ret.is_err() {
        error!("FailedToCallBindWorkerPNetworkIdent: {:?}", ret);
        return Err(anyhow!("failed to call bind_worker_pnetwork_ident"));
    }
    signer.increment_nonce();
    Ok(())
}

pub async fn prouter_main(args: &Args) -> Result<()> {
    let mut i2pd = I2PD::new("PRouter".parse()?);
    {
        // let mut f = SU3File::new("sooptq@gmail.com")?;
        // f.reseed("./pdata/netDb")?;
        // let path = Path::new("./test.su3");
        // f.write(path.to_path_buf())?;
        // return Ok(());
        let mut pr: Option<PhactoryApiClient<pruntime_client::RpcRequest>> = None;
        // let mut api: Option<RelaychainApi> = None;
        // let mut para_api: Option<ParachainApi> = None;
        let mut api = API.lock().unwrap();
        let mut para_api = PARA_API.lock().unwrap();
        let pair = <sr25519::Pair as Pair>::from_string(&args.mnemonic, None)
            .expect("Bad privkey derive path");
        let mut signer: subxt::PairSigner<phaxt::KhalaConfig, sr25519::Pair> =
            subxt::PairSigner::new(pair); // Only usable when registering worker

        let pnetwork_ident_sk: Vec<u8>;
        let mut pnetwork_ident: String = Default::default();

        // Connect to substrate
        if !args.no_pnode {
            // api= Some(subxt_connect(&args.substrate_ws_endpoint).await?.into());
            *api = Some(subxt_connect(&args.substrate_ws_endpoint).await?.into());
            info!("Connected to relaychain at: {}", args.substrate_ws_endpoint);

            let para_uri: &str = if args.parachain {
                &args.collator_ws_endpoint
            } else {
                &args.substrate_ws_endpoint
            };
            // para_api = Some(subxt_connect(para_uri).await?.into());
            *para_api = Some(subxt_connect(para_uri).await?.into());
            info!(
                "Connected to parachain node at: {}",
                args.collator_ws_endpoint
            );

            if !args.no_wait {
                // Don't start our worker until the substrate node is synced
                info!("Waiting for substrate to sync blocks...");
                wait_until_synced(&api.as_ref().expect("Api should be initialized here").client)
                    .await?;
                wait_until_synced(
                    &para_api
                        .as_ref()
                        .expect("ParaApi should be initialized here")
                        .client,
                )
                .await?;
                info!("Substrate sync blocks done");
            }
        }

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
        i2pd.add_config(
            "datadir".parse()?,
            pathbuf_to_string(abs_datadir_path.clone())?,
        );
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
                    .as_ref()
                    .expect("guaranteed to be initialized")
                    .derive_ident_sk(prpc::DeriveIdentSkRequest {
                        info: b"pNetwork".to_vec(),
                    })
                    .await?
                    .sk;
            } else {
                // Random generate
                info!("No pRuntime, using random pubkey instead");
                pnetwork_ident_sk = (0..64).map(|_| rand::random::<u8>()).collect();
            }
            pnetwork_ident = i2pd::generate_ident_to_file(
                &abs_datadir_path,
                "pnetwork.key".to_string(),
                pnetwork_ident_sk,
            )?;
        }

        if args.join_pnetwork && !args.no_pnode && !args.no_pruntime {
            // bind our ident
            info!("Binding Phala Network identity: {}", &pnetwork_ident);
            let info = &pr
                .as_ref()
                .expect("guaranteed to be initialized")
                .get_runtime_info(())
                .await?;
            let pubkey: phaxt::khala::runtime_types::sp_core::sr25519::Public =
                Decode::decode(&mut &info.encoded_public_key[..])
                    .map_err(|_| anyhow!("Decode pruntime info failed"))?;
            bind_worker_pnetwork_ident(
                &para_api.as_ref().expect("guaranteed to be initialized"),
                &pubkey,
                &mut signer,
                &pnetwork_ident,
            )
            .await?;
        }
    } // unlock api and para_api

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
    let api = Arc::clone(&*API);
    let para_api = Arc::clone(&*PARA_API);
    let rocket = thread::Builder::new()
        .name("rocket".into())
        .spawn(move || {
            server::rocket(api, para_api).launch();
        })
        .expect("Failed to launch Rocket");
    match prouter_main(&args).await {
        Ok(()) => {
            std::process::exit(0);
        }
        Err(e) => panic!("Fetal error: {:?}", e),
    };
    let _ = rocket.join();
}

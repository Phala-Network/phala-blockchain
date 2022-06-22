use anyhow::{anyhow, Context};
use fast_socks5::{
    client::{self, Socks5Stream},
    server::{Config, SimpleUserPassword, Socks5Server, Socks5Socket},
    util::target_addr::TargetAddr,
    Result, SocksError,
};
use log::{debug, error, info};

use std::convert::TryInto;
use std::io::ErrorKind;
use std::net::ToSocketAddrs;
use structopt::StructOpt;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::task;
use tokio_stream::StreamExt;

use phala_types::EndpointType;

use binascii::b32decode;

use crate::translator;

pub async fn spawn_socks_server(
    local_proxy: String,
    para_api: super::SharedParachainApi,
    server_address: String,
    server_port: u16,
) -> Result<()> {
    let mut config = Config::default();
    config.set_dns_resolve(false);
    config.set_transfer_data(false);

    let mut listener = Socks5Server::bind(format!("{}:{}", &server_address, &server_port)).await?;
    listener.set_config(config);

    let mut incoming = listener.incoming();

    info!(
        "Listen for socks connections @ {}:{}, using proxy @ {}",
        &server_address, &server_port, &local_proxy
    );

    while let Some(socket_res) = incoming.next().await {
        match socket_res {
            Ok(socket) => {
                let proxy_addr = local_proxy.clone();
                let para_api = para_api.clone();
                task::spawn(async move {
                    if let Err(err) = handle_socket(socket, proxy_addr, para_api).await {
                        error!("socket handle error = {:#}", err);
                    }
                });
            }
            Err(err) => {
                error!("accept error = {:#}", err);
            }
        }
    }

    Ok(())
}

async fn handle_socket<T>(
    socket: Socks5Socket<T>,
    proxy_addr: String,
    para_api: super::SharedParachainApi,
) -> Result<()>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    // upgrade socket to SOCKS5 proxy
    let mut socks5_socket = socket
        .upgrade_to_socks5()
        .await
        .context("upgrade incoming socket to socks5")?;

    let unresolved_target_addr = socks5_socket
        .target_addr()
        .context("find unresolved target address for incoming socket")?;

    debug!(
        "incoming request for target address: {}",
        unresolved_target_addr
    );

    let (target_addr, target_port) =
        resolve_domain(unresolved_target_addr.clone(), para_api.clone()).await?;

    debug!(
        "incoming request resolved target address to: {}:{}",
        target_addr, target_port
    );

    // connect to downstream proxy
    let mut stream = Socks5Stream::connect(
        proxy_addr,
        target_addr,
        target_port,
        client::Config::default(),
    )
    .await
    .context("connect to downstream proxy for incoming socket")?;

    // copy data between our incoming client and the used downstream proxy
    match tokio::io::copy_bidirectional(&mut stream, &mut socks5_socket).await {
        Ok(res) => {
            info!("socket transfer closed ({}, {})", res.0, res.1);
            Ok(())
        }
        Err(err) => match err.kind() {
            ErrorKind::NotConnected => {
                info!("socket transfer closed by client");
                Ok(())
            }
            ErrorKind::ConnectionReset => {
                info!("socket transfer closed by downstream proxy");
                Ok(())
            }
            _ => Err(SocksError::Other(anyhow!(
                "socket transfer error: {:#}",
                err
            ))),
        },
    }
}

async fn resolve_domain(
    target_addr: TargetAddr,
    para_api: super::SharedParachainApi,
) -> Result<(String, u16)> {
    match target_addr {
        TargetAddr::Ip(ip) => Ok((ip.ip().to_string(), ip.port())),
        TargetAddr::Domain(domain, port) => {
            debug!("Attempt to resolve the domain {}", &domain);
            // if it ends with `.i2p`, it is a I2P domain
            if domain.ends_with(".i2p") {
                // if it ends with `phala.i2p`, it is a phala domain
                if domain.ends_with("phala.i2p") {
                    let b32_pubkey = domain.split(".").collect::<Vec<&str>>()[0];
                    debug!("Attempt to resolve encoded phala domain {}", &domain);
                    let mut output_buffer = [0u8; 64];
                    let decoded_pubkey = b32decode(&b32_pubkey.as_bytes(), &mut output_buffer)
                        .map_err(|e| anyhow!("Failed to decode the pubkey"))
                        .context("Decode phala domain")?;
                    let mut endpoint_str = String::new();
                    {
                        let para_api = para_api.lock().unwrap();
                        let endpoint = translator::block_get_endpoint_info_by_pubkey(
                            &mut para_api.as_ref().expect("guaranteed to be initialized"),
                            decoded_pubkey
                                .try_into()
                                .map_err(|e| anyhow!("Failed to convert pubkey to endpoint: {}", e))
                                .expect("guaranteed to be a valid pubkey"),
                            EndpointType::I2P,
                        )
                        .ok_or(anyhow!("Failed to fetch on-chain storage"))
                        .context("Fetch on-chain storage")?;
                        endpoint_str = String::from_utf8_lossy(&endpoint).into_owned();
                    }
                    let endpoint_url = endpoint_str.split(":").collect::<Vec<&str>>()[0];
                    let endpoint_port = endpoint_str.split(":").collect::<Vec<&str>>()[1];
                    debug!("Resolved phala domain {}:{}", &endpoint_url, &endpoint_port);

                    return Ok((
                        endpoint_url.to_string(),
                        endpoint_port
                            .parse::<u16>()
                            .ok()
                            .expect("guaranteed to be a valid port"),
                    ));
                };
                return Ok((domain, port));
            };

            Ok((domain, port))
        }
    }
}

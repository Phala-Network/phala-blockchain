mod helper;
mod replay_gk;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(about = "The Phala TEE worker app.", version, author)]
pub struct Args {
    #[arg(
        default_value = "ws://localhost:9944",
        long,
        help = "Substrate rpc websocket endpoint."
    )]
    node_uri: String,

    #[arg(
        long,
        help = "Headers cache endpoint."
    )]
    cache_uri: Option<String>,

    #[arg(
        default_value = "413895",
        long,
        help = "The block number to start to replay at."
    )]
    start_at: u32,

    #[arg(
        long,
        help = "The block number to stop at."
    )]
    stop_at: Option<u32>,

    #[arg(
        default_value = "127.0.0.1:8080",
        long,
        help = "Bind address for local HTTP server."
    )]
    bind_addr: String,

    #[arg(
        default_value = "",
        long,
        help = "The PostgresQL database to store the events."
    )]
    persist_events_to: String,

    #[arg(
        default_value = "0",
        long,
        help = "Assume the give number of block finalized."
    )]
    assume_finalized: u32,

    #[arg(
        default_value = "100000",
        long,
        help = "The number of blocks between two checkpoints. 0 for disabled"
    )]
    checkpoint_interval: u32,

    #[arg(
        long,
        help = "The checkpoint file to restore from. Default is to use the latest checkpoint."
    )]
    restore_from: Option<String>,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let args = Args::parse();
    replay_gk::replay(args).await.expect("Failed to run replay");
}

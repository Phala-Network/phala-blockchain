mod replay_gk;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "replay")]
pub struct Args {
    #[structopt(
        default_value = "ws://localhost:9944",
        long,
        help = "Substrate rpc websocket endpoint."
    )]
    node_uri: String,

    #[structopt(
        default_value = "413895",
        long,
        help = "The block number to start to replay at."
    )]
    start_at: u32,

    #[structopt(
        default_value = "127.0.0.1:8080",
        long,
        help = "Bind address for local HTTP server."
    )]
    bind_addr: String,

    #[structopt(
        default_value = "",
        long,
        help = "The PostgresQL database to store the events."
    )]
    persist_events_to: String,

    #[structopt(
        default_value = "0",
        long,
        help = "Assume the give number of block finalized."
    )]
    assume_finalized: u32,

    #[structopt(
        default_value = "100000",
        long,
        help = "The number of blocks between two checkpoints. 0 for disabled"
    )]
    checkpoint_interval: u32,

    #[structopt(
        long,
        help = "The checkpoint file to restore from. Default is to use the latest checkpoint."
    )]
    restore_from: Option<String>,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let args = Args::from_args();
    replay_gk::replay(args).await.expect("Failed to run replay");
}

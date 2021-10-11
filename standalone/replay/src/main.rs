mod replay_gk;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "replay")]
struct Args {
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
        help = "Assmue the give number of block finalized."
    )]
    assume_finalized: u32,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let args = Args::from_args();
    replay_gk::replay(
        args.node_uri,
        args.start_at,
        args.persist_events_to,
        args.bind_addr,
        args.assume_finalized,
    )
    .await
    .expect("Failed to run replay");
}

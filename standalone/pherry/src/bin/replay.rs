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

    #[structopt(long, help = "The PostgresQL database to store the events.")]
    db_uri: String,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let args = Args::from_args();
    pherry::replay_gk::replay(args.node_uri, args.start_at, args.db_uri)
        .await
        .expect("Failed to run replay");
}

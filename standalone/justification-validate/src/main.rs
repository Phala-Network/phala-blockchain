use clap::Parser;

mod feed_pruntime;
mod validator;

#[derive(Parser)]
#[clap(about = "Validate justifications", version, author)]
pub struct Args {
    /// The genesis.bin grabbed with headers-cache.
    #[arg(long)]
    genesis: String,
    /// The headers.bin grabbed with headers-cache.
    #[arg(long)]
    headers: String,
    /// The block number that need to validate from.
    #[arg(long, default_value_t = 0)]
    from: u32,
    /// The block number that need to validate to.
    #[arg(long, default_value_t = u32::MAX)]
    to: u32,
    /// Number of threads used to validate headers in background.
    #[arg(long, default_value_t = default_n_threads())]
    threads: usize,
    /// Feed headers into given pruntime rather than validate in place.
    #[arg(long)]
    pruntime: Option<String>,
}

fn default_n_threads() -> usize {
    std::thread::available_parallelism()
        .map(|x| x.get())
        .unwrap_or(1)
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Args::parse();
    if let Some(url) = args.pruntime.clone() {
        feed_pruntime::feed_pruntime(url, args).await;
    } else {
        validator::Validator::new(args).run();
    }
}

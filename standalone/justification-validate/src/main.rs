use clap::Parser;

mod feed_pruntime;
mod fetcher;
mod validator;

#[derive(Parser, Clone)]
#[clap(about = "Validate justifications", version, author)]
pub struct Args {
    /// Feed headers into given pruntime rather than validate in place.
    #[arg(long)]
    pruntime: Option<String>,
    /// The headers-cache URI.
    cache_uri: String,
}

#[tokio::main]
async fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info,phactory::light_validation=warn");
    }
    env_logger::init();
    let args = Args::parse();
    if let Some(url) = args.pruntime.clone() {
        feed_pruntime::feed_pruntime(url, args).await;
    } else {
        validator::Validator::new(args).await.run().await;
    }
}

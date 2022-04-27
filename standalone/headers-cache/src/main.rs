#![feature(decl_macro)] // for rocket

use scale::Encode;
use std::fs::File;
use std::io::Write;

use clap::{AppSettings, Parser, Subcommand};
use pherry::headers_cache as cache;

mod db;
mod web_api;

#[derive(Parser)]
#[clap(about = "Cache server for relaychain headers", version, author)]
#[clap(global_setting(AppSettings::DeriveDisplayOrder))]
pub struct Args {
    /// The relaychain RPC endpoint
    #[clap(long, default_value = "ws://localhost:9945")]
    node_uri: String,
    /// The block number to be treated as genesis
    #[clap(long, default_value = "cache.db")]
    db: String,
    #[clap(subcommand)]
    action: Action,
}

#[derive(Subcommand)]
enum Action {
    /// Grap headers from the chain and dump them to a file
    Grab {
        /// The block number to start at
        #[clap(long, default_value_t = 1)]
        from_block: u32,
        /// Number of headers to grab
        #[clap(long, default_value_t = u32::MAX)]
        count: u32,
        /// Minimum number of blocks between justification
        #[clap(default_value_t = 1000)]
        justification_interval: u32,
        /// The file to write the headers to
        #[clap(default_value = "headers.bin")]
        output: String,
    },
    /// Import headers from a file into the cache database
    Import {
        /// The file to read from
        #[clap(default_value = "headers.bin")]
        input: String,
    },
    /// Grab genesis info from the chain and dump it to a file
    GrabGenesis {
        /// The block number to be treated as genesis
        #[clap(long, default_value_t = 1)]
        block: u32,
        /// The file to write the result to
        #[clap(default_value = "genesis.bin")]
        output: String,
    },
    /// Import genesis info from a file into the cache database
    ImportGenesis {
        /// The genesis file to read from
        #[clap(default_value = "genesis.bin")]
        input: String,
    },
    /// Run the cache server
    Serve,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args = Args::parse();
    match args.action {
        Action::Grab {
            from_block,
            count,
            justification_interval,
            output,
        } => {
            let output = File::create(output)?;
            let api = pherry::subxt_connect(&args.node_uri).await?.into();
            let count = cache::grap_headers_to_file(
                &api,
                from_block,
                count,
                justification_interval,
                output,
            )
            .await?;
            println!("{} headers written", count);
        }
        Action::GrabGenesis { block, output } => {
            let mut output = File::create(output)?;
            let api = pherry::subxt_connect(&args.node_uri).await?.into();
            let info = cache::fetch_genesis_info(&api, block).await?;
            output.write_all(info.encode().as_ref())?;
        }
        Action::Import { input } => {
            let input = File::open(input)?;
            let mut cache = db::CacheDB::open(&args.db)?;
            let count = cache::import_headers(input, &mut cache).await?;
            println!("{} headers imported", count);
        }
        Action::ImportGenesis { input } => {
            let mut cache = db::CacheDB::open(&args.db)?;
            cache.put_genesis(&std::fs::read(input)?)?;
            println!("done");
        }
        Action::Serve => {
            web_api::serve(&args.db)?;
        }
    }
    Ok(())
}

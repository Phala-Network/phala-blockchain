#![feature(decl_macro)] // for rocket

use std::fs::File;
use std::io::Write;

use anyhow::Context;
use scale::{Decode, Encode};

use clap::{AppSettings, Parser, Subcommand};
use pherry::headers_cache as cache;

mod db;
mod web_api;

type BlockNumber = u32;

#[derive(Parser)]
#[clap(about = "Cache server for relaychain headers", version, author)]
#[clap(global_setting(AppSettings::DeriveDisplayOrder))]
pub struct Args {
    #[clap(subcommand)]
    action: Action,
}

#[derive(Subcommand)]
enum Import {
    /// Import headers from given file to database.
    Headers {
        /// The grabbed headers file to read from
        #[clap(default_value = "headers.bin")]
        input: String,
    },
    /// Import genesis from given file to database.
    Genesis {
        /// The grabbed genesis file to read from
        #[clap(default_value = "genesis.bin")]
        input: String,
    },
}

#[derive(Subcommand)]
enum Grab {
    /// Grap headers from the chain and dump them to a file
    Headers {
        /// The relaychain RPC endpoint
        #[clap(long, default_value = "ws://localhost:9945")]
        node_uri: String,
        /// The parachain RPC endpoint
        #[clap(long, default_value = "ws://localhost:9944")]
        para_node_uri: String,
        /// The block number to start at
        #[clap(long, default_value_t = 1)]
        from_block: BlockNumber,
        /// Number of headers to grab
        #[clap(long, default_value_t = BlockNumber::MAX)]
        count: BlockNumber,
        /// Prefered minimum number of blocks between justification
        #[clap(long, default_value_t = 1000)]
        justification_interval: BlockNumber,
        /// The file to write the headers to
        #[clap(default_value = "headers.bin")]
        output: String,
    },
    Genesis {
        /// The relaychain RPC endpoint
        #[clap(long, default_value = "ws://localhost:9945")]
        node_uri: String,
        /// The block number to be treated as genesis
        #[clap(long, default_value_t = 0)]
        block: BlockNumber,
        /// The file to write the result to
        #[clap(default_value = "genesis.bin")]
        output: String,
    },
}

#[derive(Subcommand)]
enum Action {
    /// Grab genesis or headers from the blockchain and dump it to a file
    Grab {
        /// What to grab
        #[clap(subcommand)]
        what: Grab,
    },
    /// Import genesis or headers from a file into the cache database
    Import {
        /// The database file to use
        #[clap(long, default_value = "cache.db")]
        db: String,
        /// What type of data to import
        #[clap(subcommand)]
        what: Import,
    },
    /// Run the cache server
    Serve {
        /// The database file to use
        #[clap(long, default_value = "cache.db")]
        db: String,
    },
    /// For debug. Show the authority set id for a given block
    ShowSetId {
        #[clap(long)]
        uri: String,
        block: BlockNumber,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args = Args::parse();
    match args.action {
        Action::Grab { what } => match what {
            Grab::Headers {
                node_uri,
                para_node_uri,
                from_block,
                count,
                justification_interval,
                output,
            } => {
                let api = pherry::subxt_connect(&node_uri).await?.into();
                let para_api = pherry::subxt_connect(&para_node_uri).await?.into();
                let output = File::create(output)?;
                let count = cache::grap_headers_to_file(
                    &api,
                    &para_api,
                    from_block,
                    count,
                    justification_interval,
                    output,
                )
                .await?;
                println!("{} headers written", count);
            }
            Grab::Genesis {
                node_uri,
                block,
                output,
            } => {
                let api = pherry::subxt_connect(&node_uri).await?.into();
                let mut output = File::create(output)?;
                let info = cache::fetch_genesis_info(&api, block).await?;
                output.write_all(info.encode().as_ref())?;
            }
        },
        Action::Import { db, what } => {
            let mut cache = db::CacheDB::open(&db)?;
            match what {
                Import::Headers { input } => {
                    let input = File::open(&input)?;
                    let count = cache::import_headers(input, &mut cache).await?;
                    println!("{} headers imported", count);
                }
                Import::Genesis { input } => {
                    let data = std::fs::read(input)?;
                    let info = cache::GenesisBlockInfo::decode(&mut &data[..])
                        .context("Failed to decode the genesis data")?;
                    cache.put_genesis(&data, info.block_header.number)?;
                    println!("genesis at {} put", info.block_header.number);
                }
            }
        }
        Action::Serve { db } => {
            web_api::serve(&db)?;
        }
        Action::ShowSetId { uri, block } => {
            let api = pherry::subxt_connect(&uri).await?.into();
            let id = cache::get_set_id(&api, block).await?;
            println!("{:?}", id);
        }
    }
    Ok(())
}

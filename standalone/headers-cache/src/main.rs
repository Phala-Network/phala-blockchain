use std::fs::File;
use std::io::Write;

use anyhow::Context;
use log::{error, info};
use scale::{Decode, Encode};

use clap::{AppSettings, Parser, Subcommand};
use pherry::headers_cache as cache;

mod db;
mod grab;
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
        /// The grabbed headers files to read from
        #[clap(default_value = "headers.bin")]
        input_files: Vec<String>,
    },
    /// Import parachain headers from given file to database.
    ParaHeaders {
        /// The grabbed headers files to read from
        #[clap(default_value = "parachain-headers.bin")]
        input_files: Vec<String>,
    },
    /// Import storage changes from given file to database.
    StorageChanges {
        /// The grabbed files to read from
        #[clap(default_value = "storage-changes.bin")]
        input_files: Vec<String>,
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
    /// Grap parachain headers from the chain and dump them to a file
    ParaHeaders {
        /// The parachain RPC endpoint
        #[clap(long, default_value = "ws://localhost:9944")]
        para_node_uri: String,
        /// The block number to start at
        #[clap(long, default_value_t = 0)]
        from_block: BlockNumber,
        /// Number of headers to grab
        #[clap(long, default_value_t = BlockNumber::MAX)]
        count: BlockNumber,
        /// The file to write the headers to
        #[clap(default_value = "parachain-headers.bin")]
        output: String,
    },
    /// Grap storage changes from the chain and dump them to a file
    StorageChanges {
        /// The parachain RPC endpoint
        #[clap(long, default_value = "ws://localhost:9944")]
        para_node_uri: String,
        /// The block number to start at
        #[clap(long, default_value_t = 0)]
        from_block: BlockNumber,
        /// Number of headers to grab
        #[clap(long, default_value_t = BlockNumber::MAX)]
        count: BlockNumber,
        /// Number of blocks requested in a single RPC.
        #[clap(long, default_value_t = 10)]
        batch_size: BlockNumber,
        /// The file to write the headers to
        #[clap(default_value = "storage-changes.bin")]
        output: String,
    },
    Genesis {
        /// The relaychain RPC endpoint
        #[clap(long, default_value = "ws://localhost:9945")]
        node_uri: String,
        /// The block number to be treated as genesis
        #[clap(long, default_value_t = 0)]
        from_block: BlockNumber,
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
        /// Auto grab new headers from the node
        #[clap(long)]
        grab: bool,
        /// The relaychain RPC endpoint
        #[clap(long, default_value = "ws://localhost:9945")]
        node_uri: String,
        /// The parachain RPC endpoint
        #[clap(long, default_value = "ws://localhost:9944")]
        para_node_uri: String,
        /// Interval that start a batch of grab
        #[clap(long, default_value_t = 600)]
        interval: u64,
    },
    /// Split given grabbed headers file into chunks
    Split {
        /// Size in MB of each chunk
        #[clap(long, default_value_t = 200)]
        size: usize,

        /// The headers file to split
        file: String,
    },
    /// Show block number info for given bin file
    Inspect {
        /// The grabbed headers file to read from
        files: Vec<String>,
    },
    /// Show imported block number info in the cache database
    InspectDb {
        /// The database file to use
        #[clap(long, default_value = "cache.db")]
        db: String,
    },
    /// Merge given chunks into a single file
    Merge {
        /// Appending to existing file
        #[clap(long, short = 'a')]
        append: bool,
        /// The destination file to write to
        dest_file: String,
        /// The header chunk files to merge.
        files: Vec<String>,
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
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

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
                let api = pherry::subxt_connect(&node_uri).await?;
                let para_api = pherry::subxt_connect(&para_node_uri).await?;
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
                println!("{count} headers written");
            }
            Grab::ParaHeaders {
                para_node_uri,
                from_block,
                count,
                output,
            } => {
                let para_api = pherry::subxt_connect(&para_node_uri).await?;
                let output = File::create(output)?;
                let count =
                    cache::grap_para_headers_to_file(&para_api, from_block, count, output).await?;
                println!("{count} headers written");
            }
            Grab::StorageChanges {
                para_node_uri,
                from_block,
                count,
                batch_size,
                output,
            } => {
                let para_api = pherry::subxt_connect(&para_node_uri).await?;
                let output = File::create(output)?;
                let count = cache::grap_storage_changes_to_file(
                    &para_api, from_block, count, batch_size, output,
                )
                .await?;
                println!("{count} blocks written");
            }
            Grab::Genesis {
                node_uri,
                from_block,
                output,
            } => {
                let api = pherry::subxt_connect(&node_uri).await?;
                let mut output = File::create(output)?;
                let info = cache::fetch_genesis_info(&api, from_block).await?;
                output.write_all(info.encode().as_ref())?;
            }
        },
        Action::Import { db, what } => {
            let cache = db::CacheDB::open(&db)?;
            match what {
                Import::Headers { input_files } => {
                    for filename in input_files {
                        println!("Importing headers from {filename}");
                        let mut metadata = cache.get_metadata()?.unwrap_or_default();
                        let input = File::open(&filename)?;
                        let count = cache::read_items(input, |record| {
                            let header = record.header()?;
                            cache.put_header(header.number, record.payload())?;
                            if header.number % 1000 == 0 {
                                info!("Imported to {}", header.number);
                            }
                            metadata.update_header(header.number);
                            Ok(false)
                        })?;
                        cache.put_metadata(&metadata)?;
                        println!("{count} headers imported");
                    }
                }
                Import::ParaHeaders { input_files } => {
                    for filename in input_files {
                        println!("Importing parachain headers from {filename}");
                        let input = File::open(&filename)?;
                        let mut metadata = cache.get_metadata()?.unwrap_or_default();
                        let count = cache::read_items(input, |record| {
                            let header = record.header()?;
                            cache.put_para_header(header.number, record.payload())?;
                            if header.number % 1000 == 0 {
                                info!("Imported to {}", header.number);
                            }
                            metadata.update_para_header(header.number);
                            Ok(false)
                        })?;
                        cache.put_metadata(&metadata)?;
                        println!("{count} headers imported");
                    }
                }
                Import::StorageChanges { input_files } => {
                    for filename in input_files {
                        println!("Importing storage changes from {filename}");
                        let input = File::open(&filename)?;
                        let mut metadata = cache.get_metadata()?.unwrap_or_default();
                        let count = cache::read_items(input, |record| {
                            let header = record.header()?;
                            cache.put_storage_changes(header.number, record.payload())?;
                            if header.number % 1000 == 0 {
                                info!("Imported to {}", header.number);
                            }
                            metadata.update_storage_changes(header.number);
                            Ok(false)
                        })?;
                        cache.put_metadata(&metadata)?;
                        println!("{count} blocks imported");
                    }
                }
                Import::Genesis { input } => {
                    let data = std::fs::read(input)?;
                    let info = cache::GenesisBlockInfo::decode(&mut &data[..])
                        .context("Failed to decode the genesis data")?;
                    cache.put_genesis(info.block_header.number, &data)?;
                    let mut metadata = cache.get_metadata()?.unwrap_or_default();
                    metadata.put_genesis(info.block_header.number);
                    cache.put_metadata(&metadata)?;
                    println!("genesis at {} put", info.block_header.number);
                }
            }
            cache.flush()?;
        }
        Action::Serve {
            db,
            grab,
            node_uri,
            para_node_uri,
            interval,
        } => {
            let db = db::CacheDB::open(&db)?;
            if grab {
                let db = db.clone();
                tokio::spawn(async move {
                    let result = grab::run(db, &node_uri, &para_node_uri, interval).await;
                    if let Err(err) = result {
                        error!("The grabbing task exited with error: {}", err);
                    }
                });
            }
            web_api::serve(db).await?;
        }
        Action::ShowSetId { uri, block } => {
            let api = pherry::subxt_connect(&uri).await?;
            let id = cache::get_set_id(&api, block).await?;
            println!("{id:?}");
        }
        Action::Split { size, file } => {
            let mut input = File::open(&file)?;
            let tmpfile = format!("{file}.output.tmp");
            let limit = size * 1024 * 1024;
            loop {
                let mut outfile = File::create(&tmpfile)?;
                let mut file_size = 0;
                let mut first = u32::MAX;
                let mut last = 0;
                let count = cache::read_items(&mut input, |record| {
                    let hdr = record.header()?;
                    let len = record.write(&mut outfile)?;
                    if first == u32::MAX {
                        first = hdr.number;
                    }
                    last = hdr.number;
                    file_size += len;
                    Ok(file_size >= limit)
                })?;
                drop(outfile);
                if count == 0 {
                    std::fs::remove_file(&tmpfile)?;
                    break;
                } else {
                    let filename = format!("{file}_{first:0>9}-{last:0>9}");
                    std::fs::rename(&tmpfile, &filename)?;
                    println!("Saved {filename}");
                }
            }
        }
        Action::Merge {
            append,
            dest_file,
            files,
        } => {
            let mut output = if append {
                std::fs::OpenOptions::new().append(true).open(dest_file)?
            } else {
                File::create(dest_file)?
            };
            for filename in files {
                println!("Merging {filename}");
                std::io::copy(&mut File::open(filename)?, &mut output)?;
            }
        }
        Action::Inspect { files } => {
            for file in &files {
                let mut input = File::open(file)?;
                let mut first = None;
                let mut last = None;
                let count = cache::read_items(&mut input, |record| {
                    let number = record.header()?.number;
                    if first.is_none() {
                        first = Some(number);
                    }
                    if let Some(last) = &last {
                        if last + 1 != number {
                            println!("WARN: non-contiguous block numbers: {last} -> {number}");
                        }
                    }
                    last = Some(number);
                    Ok(false)
                });
                match (first, last, count) {
                    (None, None, Ok(0)) => {
                        println!("{file}: no blocks");
                    }
                    (Some(first), Some(last), Ok(count)) => {
                        println!("{file}: {count} blocks, from {first} to {last}");
                    }
                    _ => {
                        println!("{file}: broken file");
                    }
                }
            }
        }
        Action::InspectDb { db } => {
            let cache = db::CacheDB::open(&db)?;
            let metadata = cache.get_metadata()?.unwrap_or_default();
            serde_json::to_writer_pretty(std::io::stdout(), &metadata)?;
        }
    }
    Ok(())
}

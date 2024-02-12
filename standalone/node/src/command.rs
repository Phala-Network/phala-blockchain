// This file is part of Substrate.

// Copyright (C) 2017-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use super::benchmarking::{inherent_benchmark_data, RemarkBuilder, TransferKeepAliveBuilder};
use crate::{
    chain_spec, service,
    service::{ChainOpsComponents, FullClient},
    Cli, Subcommand,
};
use frame_benchmarking_cli::*;
use node_runtime::{Block, ExistentialDeposit, RuntimeApi};
use sc_cli::SubstrateCli;
use sp_keyring::Sr25519Keyring;

use std::sync::Arc;

#[cfg(feature = "try-runtime")]
use {
    node_runtime::constants::time::SLOT_DURATION,
    try_runtime_cli::block_building_info::substrate_info,
};

impl SubstrateCli for Cli {
    fn impl_name() -> String {
        "Phala Substrate Node".into()
    }

    fn impl_version() -> String {
        env!("SUBSTRATE_CLI_IMPL_VERSION").into()
    }

    fn description() -> String {
        env!("CARGO_PKG_DESCRIPTION").into()
    }

    fn author() -> String {
        env!("CARGO_PKG_AUTHORS").into()
    }

    fn support_url() -> String {
        "https://github.com/Phala-Network/phala-blockchain/issues/new".into()
    }

    fn copyright_start_year() -> i32 {
        2019
    }

    fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
        let spec = match id {
            "" => {
                return Err(
                    "Please specify which chain you want to run, e.g. --dev or --chain=local"
                        .into(),
                )
            }
            "dev" => match self.block_millisecs {
                Some(block_millisecs) => Box::new(
                    chain_spec::development_config_custom_block_duration(block_millisecs),
                ),
                None => Box::new(chain_spec::development_config()),
            },
            "local" => Box::new(chain_spec::local_config()),
            "phala" | "phala_testnet" => Box::new(chain_spec::testnet_config()?),
            "phala-local" | "phala_testnet-local" => Box::new(chain_spec::testnet_local_config()),
            path => Box::new(chain_spec::ChainSpec::from_json_file(
                std::path::PathBuf::from(path),
            )?),
        };
        Ok(spec)
    }
}

/// Parse command line arguments into service configuration.
pub fn run() -> sc_cli::Result<()> {
    let cli = Cli::from_args();

    match &cli.subcommand {
        None => {
            let runner = cli.create_runner(&cli.run)?;
            runner.run_node_until_exit(|config| async move {
                service::new_full(
                    config,
                    &cli.eth,
                    cli.no_hardware_benchmarks,
                    cli.gossip_duration_millisecs,
                )
                .await
                .map_err(sc_cli::Error::Service)
            })
        }
        Some(Subcommand::Inspect(cmd)) => {
            let runner = cli.create_runner(cmd)?;

            runner.sync_run(|config| cmd.run::<Block, RuntimeApi>(config))
        }
        Some(Subcommand::Benchmark(cmd)) => {
            let runner = cli.create_runner(cmd)?;

            runner.sync_run(|mut config| {
                // This switch needs to be in the client, since the client decides
                // which sub-commands it wants to support.
                match cmd {
                    BenchmarkCmd::Pallet(cmd) => {
                        if !cfg!(feature = "runtime-benchmarks") {
                            return Err(
                                "Runtime benchmarking wasn't enabled when building the node. \
                            You can enable it with `--features runtime-benchmarks`."
                                    .into(),
                            );
                        }

                        cmd.run::<Block, ()>(config)
                    }
                    BenchmarkCmd::Block(cmd) => {
                        // ensure that we keep the task manager alive
                        let partial = service::new_chain_ops(&mut config, &cli.eth)?;
                        cmd.run(partial.client)
                    }
                    #[cfg(not(feature = "runtime-benchmarks"))]
                    BenchmarkCmd::Storage(_) => Err(
                        "Storage benchmarking can be enabled with `--features runtime-benchmarks`."
                            .into(),
                    ),
                    #[cfg(feature = "runtime-benchmarks")]
                    BenchmarkCmd::Storage(cmd) => {
                        // ensure that we keep the task manager alive
                        let partial = service::new_chain_ops(&mut config, &cli.eth)?;
                        let db = partial.backend.expose_db();
                        let storage = partial.backend.expose_storage();

                        cmd.run(config, partial.client, db, storage)
                    }
                    BenchmarkCmd::Overhead(cmd) => {
                        // ensure that we keep the task manager alive
                        let partial = service::new_chain_ops(&mut config, &cli.eth)?;
                        let ext_builder = RemarkBuilder::new(partial.client.clone());

                        cmd.run(
                            config,
                            partial.client,
                            inherent_benchmark_data()?,
                            Vec::new(),
                            &ext_builder,
                        )
                    }
                    BenchmarkCmd::Extrinsic(cmd) => {
                        // ensure that we keep the task manager alive
                        let partial = service::new_chain_ops(&mut config, &cli.eth)?;
                        // Register the *Remark* and *TKA* builders.
                        let ext_factory = ExtrinsicFactory(vec![
                            Box::new(RemarkBuilder::new(partial.client.clone())),
                            Box::new(TransferKeepAliveBuilder::new(
                                partial.client.clone(),
                                Sr25519Keyring::Alice.to_account_id(),
                                ExistentialDeposit::get(),
                            )),
                        ]);

                        cmd.run(
                            partial.client,
                            inherent_benchmark_data()?,
                            Vec::new(),
                            &ext_factory,
                        )
                    }
                    BenchmarkCmd::Machine(cmd) => {
                        cmd.run(&config, SUBSTRATE_REFERENCE_HARDWARE.clone())
                    }
                }
            })
        }
        Some(Subcommand::Key(cmd)) => cmd.run(&cli),
        Some(Subcommand::Sign(cmd)) => cmd.run(),
        Some(Subcommand::Verify(cmd)) => cmd.run(),
        Some(Subcommand::Vanity(cmd)) => cmd.run(),
        Some(Subcommand::BuildSpec(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
        }
        Some(Subcommand::CheckBlock(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|mut config| {
                let ChainOpsComponents {
                    client,
                    task_manager,
                    import_queue,
                    ..
                } = service::new_chain_ops(&mut config, &cli.eth)?;
                Ok((cmd.run(client, import_queue), task_manager))
            })
        }
        Some(Subcommand::ExportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|mut config| {
                let ChainOpsComponents {
                    client,
                    task_manager,
                    ..
                } = service::new_chain_ops(&mut config, &cli.eth)?;
                Ok((cmd.run(client, config.database), task_manager))
            })
        }
        Some(Subcommand::ExportState(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|mut config| {
                let ChainOpsComponents {
                    client,
                    task_manager,
                    ..
                } = service::new_chain_ops(&mut config, &cli.eth)?;
                Ok((cmd.run(client, config.chain_spec), task_manager))
            })
        }
        Some(Subcommand::ImportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|mut config| {
                let ChainOpsComponents {
                    client,
                    task_manager,
                    import_queue,
                    ..
                } = service::new_chain_ops(&mut config, &cli.eth)?;
                Ok((cmd.run(client, import_queue), task_manager))
            })
        }
        Some(Subcommand::PurgeChain(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.database))
        }
        Some(Subcommand::Revert(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|mut config| {
                let ChainOpsComponents {
                    client,
                    task_manager,
                    backend,
                    ..
                } = service::new_chain_ops(&mut config, &cli.eth)?;
                let aux_revert = Box::new(|client: Arc<FullClient>, backend, blocks| {
                    sc_consensus_babe::revert(client.clone(), backend, blocks)?;
                    sc_consensus_grandpa::revert(client, blocks)?;
                    Ok(())
                });
                Ok((cmd.run(client, backend, Some(aux_revert)), task_manager))
            })
        }
        Some(Subcommand::ChainInfo(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run::<Block>(&config))
        }
        Some(Subcommand::FrontierDb(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|mut config| {
                let ChainOpsComponents {
                    client,
                    frontier_backend,
                    ..
                } = service::new_chain_ops(&mut config, &cli.eth)?;
                let frontier_backend = match frontier_backend {
                    fc_db::Backend::KeyValue(kv) => std::sync::Arc::new(kv),
                    _ => panic!("Only fc_db::Backend::KeyValue supported"),
                };
                cmd.run(client, frontier_backend)
            })
        }
    }
}

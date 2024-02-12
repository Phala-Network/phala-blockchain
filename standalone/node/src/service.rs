// This file is part of Substrate.

// Copyright (C) 2018-2022 Parity Technologies (UK) Ltd.
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

#![warn(unused_extern_crates)]

//! Service implementation. Specialized wrapper over substrate service.

use codec::Encode;
use frame_system_rpc_runtime_api::AccountNonceApi;
use futures::prelude::*;
use node_primitives::Block;
use node_runtime::RuntimeApi;
use sc_client_api::{Backend, BlockBackend};
use sc_consensus::BasicQueue;
use sc_consensus_babe::{self, BabeLink, BabeWorkerHandle, SlotProportion};
use sc_consensus_grandpa as grandpa;
use sc_network::{event::Event, NetworkEventStream, NetworkService};
use sc_network_sync::{warp::WarpSyncParams, SyncingService};
use sc_service::{config::Configuration, error::Error as ServiceError, RpcHandlers, TaskManager};
use sc_telemetry::{Telemetry, TelemetryHandle, TelemetryWorker};
use sc_transaction_pool_api::OffchainTransactionPoolFactory;
use sp_api::ProvideRuntimeApi;
use sp_core::{crypto::Pair, U256};
use sp_runtime::{generic, traits::Block as BlockT, SaturatedConversion};
use std::sync::Arc;

use crate::eth::{self, spawn_frontier_tasks, EthConfiguration, FrontierBackend};
use crate::rpc;

/// Host functions required for kitchensink runtime and Substrate node.
#[cfg(not(feature = "runtime-benchmarks"))]
pub type HostFunctions = (sp_io::SubstrateHostFunctions,);

/// Host functions required for kitchensink runtime and Substrate node.
#[cfg(feature = "runtime-benchmarks")]
pub type HostFunctions = (
    sp_io::SubstrateHostFunctions,
    frame_benchmarking::benchmarking::HostFunctions,
);

/// A specialized `WasmExecutor` intended to use across substrate node. It provides all required
/// HostFunctions.
pub type RuntimeExecutor = sc_executor::WasmExecutor<HostFunctions>;

/// The full client type definition.
pub type FullClient = sc_service::TFullClient<Block, RuntimeApi, RuntimeExecutor>;
pub type FullBackend = sc_service::TFullBackend<Block>;
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;

type BasicImportQueue = sc_consensus::DefaultImportQueue<Block>;
type GrandpaBlockImport<Client> =
    grandpa::GrandpaBlockImport<FullBackend, Block, Client, FullSelectChain>;
type BoxBlockImport = sc_consensus::BoxBlockImport<Block>;

/// The minimum period of blocks on which justifications will be
/// imported and generated.
const GRANDPA_JUSTIFICATION_PERIOD: u32 = 512;

/// The transaction pool type defintion.
pub type TransactionPool = sc_transaction_pool::FullPool<Block, FullClient>;

/// Fetch the nonce of the given `account` from the chain state.
///
/// Note: Should only be used for tests.
pub fn fetch_nonce(client: &FullClient, account: sp_core::sr25519::Pair) -> u32 {
    let best_hash = client.chain_info().best_hash;
    client
        .runtime_api()
        .account_nonce(best_hash, account.public().into())
        .expect("Fetching account nonce works; qed")
}

/// Create a transaction using the given `call`.
///
/// The transaction will be signed by `sender`. If `nonce` is `None` it will be fetched from the
/// state of the best block.
///
/// Note: Should only be used for tests.
pub fn create_extrinsic(
    client: &FullClient,
    sender: sp_core::sr25519::Pair,
    function: impl Into<node_runtime::RuntimeCall>,
    nonce: Option<u32>,
) -> node_runtime::UncheckedExtrinsic {
    let function = function.into();
    let genesis_hash = client
        .block_hash(0)
        .ok()
        .flatten()
        .expect("Genesis block exists; qed");
    let best_hash = client.chain_info().best_hash;
    let best_block = client.chain_info().best_number;
    let nonce = nonce.unwrap_or_else(|| fetch_nonce(client, sender.clone()));

    let period = node_runtime::BlockHashCount::get()
        .checked_next_power_of_two()
        .map(|c| c / 2)
        .unwrap_or(2) as u64;
    let tip = 0;
    let extra: node_runtime::SignedExtra = (
        frame_system::CheckNonZeroSender::<node_runtime::Runtime>::new(),
        frame_system::CheckSpecVersion::<node_runtime::Runtime>::new(),
        frame_system::CheckTxVersion::<node_runtime::Runtime>::new(),
        frame_system::CheckGenesis::<node_runtime::Runtime>::new(),
        frame_system::CheckEra::<node_runtime::Runtime>::from(generic::Era::mortal(
            period,
            best_block.saturated_into(),
        )),
        frame_system::CheckNonce::<node_runtime::Runtime>::from(nonce),
        frame_system::CheckWeight::<node_runtime::Runtime>::new(),
        phala_pallets::pallet_mq::CheckMqSequence::<node_runtime::Runtime>::new(),
        pallet_transaction_payment::ChargeTransactionPayment::<node_runtime::Runtime>::from(tip),
    );

    let raw_payload = node_runtime::SignedPayload::from_raw(
        function.clone(),
        extra.clone(),
        (
            (),
            node_runtime::VERSION.spec_version,
            node_runtime::VERSION.transaction_version,
            genesis_hash,
            best_hash,
            (),
            (),
            (),
            (),
        ),
    );
    let signature = raw_payload.using_encoded(|e| sender.sign(e));

    node_runtime::UncheckedExtrinsic::new_signed(
        function,
        sp_runtime::AccountId32::from(sender.public()).into(),
        node_runtime::Signature::Sr25519(signature),
        extra,
    )
}

struct OtherComponents {
    block_import: BoxBlockImport,
    babe_link: sc_consensus_babe::BabeLink<Block>,
    babe_worker_handle: BabeWorkerHandle<Block>,
    grandpa_link: grandpa::LinkHalf<Block, FullClient, FullSelectChain>,
    voter_state: grandpa::SharedVoterState,
    telemetry: Option<Telemetry>,
    frontier_backend: FrontierBackend,
    overrides: Arc<fc_storage::OverrideHandle<Block>>,
}

/// Creates a new partial node.
#[allow(clippy::type_complexity, clippy::redundant_clone)]
fn new_partial<BIQ>(
    config: &Configuration,
    eth_config: &EthConfiguration,
    build_import_queue: BIQ,
) -> Result<
    sc_service::PartialComponents<
        FullClient,
        FullBackend,
        FullSelectChain,
        sc_consensus::DefaultImportQueue<Block>,
        sc_transaction_pool::FullPool<Block, FullClient>,
        OtherComponents,
    >,
    ServiceError,
>
where
    BIQ: FnOnce(
        Arc<FullClient>,
        Arc<FullBackend>,
        &Configuration,
        &EthConfiguration,
        &TaskManager,
        Option<TelemetryHandle>,
        GrandpaBlockImport<FullClient>,
        OffchainTransactionPoolFactory<Block>,
    ) -> Result<
        (
            BasicImportQueue,
            BabeWorkerHandle<Block>,
            BoxBlockImport,
            BabeLink<Block>,
        ),
        ServiceError,
    >,
{
    let telemetry = config
        .telemetry_endpoints
        .clone()
        .filter(|x| !x.is_empty())
        .map(|endpoints| -> Result<_, sc_telemetry::Error> {
            let worker = TelemetryWorker::new(16)?;
            let telemetry = worker.handle().new_telemetry(endpoints);
            Ok((worker, telemetry))
        })
        .transpose()?;

    let executor = sc_service::new_wasm_executor(config);

    let (client, backend, keystore_container, task_manager) =
        sc_service::new_full_parts::<Block, RuntimeApi, _>(
            config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
            executor,
        )?;
    let client = Arc::new(client);

    let telemetry = telemetry.map(|(worker, telemetry)| {
        task_manager
            .spawn_handle()
            .spawn("telemetry", None, worker.run());
        telemetry
    });

    let select_chain = sc_consensus::LongestChain::new(backend.clone());

    let transaction_pool = sc_transaction_pool::BasicPool::new_full(
        config.transaction_pool.clone(),
        config.role.is_authority().into(),
        config.prometheus_registry(),
        task_manager.spawn_essential_handle(),
        client.clone(),
    );

    let (grandpa_block_import, grandpa_link) = grandpa::block_import(
        client.clone(),
        GRANDPA_JUSTIFICATION_PERIOD,
        &(client.clone() as Arc<_>),
        select_chain.clone(),
        telemetry.as_ref().map(|x| x.handle()),
    )?;
    let (voter_state, import_queue, block_import, babe_link, babe_worker_handle) = {
        let shared_voter_state = sc_consensus_grandpa::SharedVoterState::empty();
        let shared_voter_state2 = shared_voter_state.clone();

        let client = client.clone();
        let (import_queue, babe_worker_handle, block_import, babe_link) = build_import_queue(
            client.clone(),
            backend.clone(),
            config,
            eth_config,
            &task_manager,
            telemetry.as_ref().map(|x| x.handle()),
            grandpa_block_import,
            OffchainTransactionPoolFactory::new(transaction_pool.clone()),
        )?;
        (
            shared_voter_state2,
            import_queue,
            block_import,
            babe_link,
            babe_worker_handle,
        )
    };
    let overrides = fc_storage::overrides_handle(client.clone());
    let frontier_backend = match eth_config.frontier_backend_type {
        eth::BackendType::KeyValue => FrontierBackend::KeyValue(fc_db::kv::Backend::open(
            Arc::clone(&client),
            &config.database,
            &eth::db_config_dir(config),
        )?),
        eth::BackendType::Sql => {
            let db_path = eth::db_config_dir(config).join("sql");
            std::fs::create_dir_all(&db_path).expect("failed creating sql db directory");
            let backend = futures::executor::block_on(fc_db::sql::Backend::new(
                fc_db::sql::BackendConfig::Sqlite(fc_db::sql::SqliteBackendConfig {
                    path: std::path::Path::new("sqlite:///")
                        .join(db_path)
                        .join("frontier.db3")
                        .to_str()
                        .unwrap(),
                    create_if_missing: true,
                    thread_count: eth_config.frontier_sql_backend_thread_count,
                    cache_size: eth_config.frontier_sql_backend_cache_size,
                }),
                eth_config.frontier_sql_backend_pool_size,
                std::num::NonZeroU32::new(eth_config.frontier_sql_backend_num_ops_timeout),
                overrides.clone(),
            ))
            .unwrap_or_else(|err| panic!("failed creating sql backend: {:?}", err));
            FrontierBackend::Sql(backend)
        }
    };

    Ok(sc_service::PartialComponents {
        client,
        backend,
        task_manager,
        keystore_container,
        select_chain,
        import_queue,
        transaction_pool,
        other: OtherComponents {
            block_import,
            babe_link,
            babe_worker_handle,
            grandpa_link,
            voter_state,
            telemetry,
            frontier_backend,
            overrides,
        },
    })
}

/// Result of [`new_full_base`].
pub struct NewFullBase {
    /// The task manager of the node.
    pub task_manager: TaskManager,
    /// The client instance of the node.
    pub client: Arc<FullClient>,
    /// The networking service of the node.
    pub network: Arc<NetworkService<Block, <Block as BlockT>::Hash>>,
    /// The syncing service of the node.
    pub sync: Arc<SyncingService<Block>>,
    /// The transaction pool of the node.
    pub transaction_pool: Arc<TransactionPool>,
    /// The rpc handlers of the node.
    pub rpc_handlers: RpcHandlers,
}

/// Creates a full service from the configuration.
pub async fn new_full_base(
    config: Configuration,
    eth_config: &EthConfiguration,
    disable_hardware_benchmarks: bool,
    gossip_duration_millis: Option<u64>,
) -> Result<NewFullBase, ServiceError> {
    let hwbench = if !disable_hardware_benchmarks {
        config.database.path().map(|database_path| {
            let _ = std::fs::create_dir_all(database_path);
            sc_sysinfo::gather_hwbench(Some(database_path))
        })
    } else {
        None
    };

    // Sinks for pubsub notifications.
    // Everytime a new subscription is created, a new mpsc channel is added to the sink pool.
    // The MappingSyncWorker sends through the channel on block import and the subscription emits a notification to the subscriber on receiving a message through this channel.
    // This way we avoid race conditions when using native substrate block import notification stream.
    let pubsub_notification_sinks: fc_mapping_sync::EthereumBlockNotificationSinks<
        fc_mapping_sync::EthereumBlockNotification<Block>,
    > = Default::default();
    let pubsub_notification_sinks = Arc::new(pubsub_notification_sinks);

    let sc_service::PartialComponents {
        client,
        backend,
        mut task_manager,
        import_queue,
        keystore_container,
        select_chain,
        transaction_pool,
        other,
    } = new_partial(&config, eth_config, build_babe_grandpa_import_queue)?;
    let OtherComponents {
        block_import,
        babe_link,
        babe_worker_handle,
        grandpa_link,
        voter_state: shared_voter_state,
        mut telemetry,
        frontier_backend,
        overrides,
        ..
    } = other;
    let eth::FrontierPartialComponents {
        filter_pool,
        fee_history_cache,
        fee_history_cache_limit,
    } = eth::new_frontier_partial(eth_config)?;

    let auth_disc_publish_non_global_ips = config.network.allow_non_globals_in_dht;
    let mut net_config = sc_network::config::FullNetworkConfiguration::new(&config.network);
    let grandpa_protocol_name = grandpa::protocol_standard_name(
        &client
            .block_hash(0)
            .ok()
            .flatten()
            .expect("Genesis block exists; qed"),
        &config.chain_spec,
    );
    let (grandpa_protocol_config, grandpa_notification_service) =
        grandpa::grandpa_peers_set_config(grandpa_protocol_name.clone());
    net_config.add_notification_protocol(grandpa_protocol_config);

    let warp_sync = Arc::new(grandpa::warp_proof::NetworkProvider::new(
        backend.clone(),
        grandpa_link.shared_authority_set().clone(),
        Vec::default(),
    ));

    let (network, system_rpc_tx, tx_handler_controller, network_starter, sync_service) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &config,
            net_config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue,
            block_announce_validator_builder: None,
            warp_sync_params: Some(WarpSyncParams::WithProvider(warp_sync)),
            block_relay: None,
        })?;

    let role = config.role.clone();
    let force_authoring = config.force_authoring;
    let backoff_authoring_blocks =
        Some(sc_consensus_slots::BackoffAuthoringOnFinalizedHeadLagging::default());
    let name = config.network.node_name.clone();
    let enable_grandpa = !config.disable_grandpa;
    let prometheus_registry = config.prometheus_registry().cloned();
    let enable_offchain_worker = config.offchain_worker.enabled;

    let rpc_builder = {
        let client = client.clone();
        let pool = transaction_pool.clone();
        let network = network.clone();
        let sync_service = sync_service.clone();
        let keystore = keystore_container.keystore();

        let is_authority = role.is_authority();
        let enable_dev_signer = eth_config.enable_dev_signer;
        let max_past_logs = eth_config.max_past_logs;
        let execute_gas_limit_multiplier = eth_config.execute_gas_limit_multiplier;
        let filter_pool = filter_pool.clone();
        let frontier_backend = frontier_backend.clone();
        let pubsub_notification_sinks = pubsub_notification_sinks.clone();
        let overrides = overrides.clone();
        let fee_history_cache = fee_history_cache.clone();
        let block_data_cache = Arc::new(fc_rpc::EthBlockDataCacheTask::new(
            task_manager.spawn_handle(),
            overrides.clone(),
            eth_config.eth_log_block_cache,
            eth_config.eth_statuses_cache,
            prometheus_registry.clone(),
        ));

        let slot_duration = babe_link.config().slot_duration();
        let target_gas_price = eth_config.target_gas_price;
        let pending_create_inherent_data_providers = move |_, ()| async move {
            let current = sp_timestamp::InherentDataProvider::from_system_time();
            let next_slot = current.timestamp().as_millis() + slot_duration.as_millis();
            let timestamp = sp_timestamp::InherentDataProvider::new(next_slot.into());
            let slot = sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
				*timestamp,
				slot_duration,
			);
            let dynamic_fee = fp_dynamic_fee::InherentDataProvider(U256::from(target_gas_price));
            Ok((slot, timestamp, dynamic_fee))
        };

        let rpc_backend = backend.clone();
        let rpc_select_chain = select_chain.clone();
        Box::new(move |deny_unsafe, subscription_task_executor| {
            let eth_deps = crate::rpc::eth::EthDeps {
                client: client.clone(),
                pool: pool.clone(),
                graph: pool.pool().clone(),
                converter: Some(node_runtime::TransactionConverter),
                is_authority,
                enable_dev_signer,
                network: network.clone(),
                sync: sync_service.clone(),
                frontier_backend: match frontier_backend.clone() {
                    fc_db::Backend::KeyValue(b) => Arc::new(b),
                    fc_db::Backend::Sql(b) => Arc::new(b),
                },
                overrides: overrides.clone(),
                block_data_cache: block_data_cache.clone(),
                filter_pool: filter_pool.clone(),
                max_past_logs,
                fee_history_cache: fee_history_cache.clone(),
                fee_history_cache_limit,
                execute_gas_limit_multiplier,
                forced_parent_hashes: None,
                pending_create_inherent_data_providers,
            };
            let deps = rpc::FullDeps {
                client: client.clone(),
                pool: pool.clone(),
                select_chain: rpc_select_chain.clone(),
                deny_unsafe,
                babe: rpc::BabeDeps {
                    keystore: keystore.clone(),
                    babe_worker_handle: babe_worker_handle.clone(),
                },
                backend: rpc_backend.clone(),
                eth: eth_deps,
            };
            let mut io = crate::rpc::create_full(
                deps,
                subscription_task_executor,
                pubsub_notification_sinks.clone(),
            )?;
            phala_node_rpc_ext::extend_rpc(
                &mut io,
                client.clone(),
                rpc_backend.clone(),
                pool.clone(),
            );
            Ok(io)
        })
    };

    let rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        config,
        backend: backend.clone(),
        client: client.clone(),
        keystore: keystore_container.keystore(),
        network: network.clone(),
        rpc_builder: Box::new(rpc_builder),
        transaction_pool: transaction_pool.clone(),
        task_manager: &mut task_manager,
        system_rpc_tx,
        tx_handler_controller,
        sync_service: sync_service.clone(),
        telemetry: telemetry.as_mut(),
    })?;

    if let Some(hwbench) = hwbench {
        sc_sysinfo::print_hwbench(&hwbench);

        if let Some(ref mut telemetry) = telemetry {
            let telemetry_handle = telemetry.handle();
            task_manager.spawn_handle().spawn(
                "telemetry_hwbench",
                None,
                sc_sysinfo::initialize_hwbench_telemetry(telemetry_handle, hwbench),
            );
        }
    }

    if let sc_service::config::Role::Authority { .. } = &role {
        let proposer = sc_basic_authorship::ProposerFactory::new(
            task_manager.spawn_handle(),
            client.clone(),
            transaction_pool.clone(),
            prometheus_registry.as_ref(),
            telemetry.as_ref().map(|x| x.handle()),
        );

        let client_clone = client.clone();
        let slot_duration = babe_link.config().slot_duration();
        let babe_config = sc_consensus_babe::BabeParams {
            keystore: keystore_container.keystore(),
            client: client.clone(),
            select_chain,
            env: proposer,
            block_import,
            sync_oracle: sync_service.clone(),
            justification_sync_link: sync_service.clone(),
            create_inherent_data_providers: move |parent, ()| {
                let client_clone = client_clone.clone();
                async move {
                    let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

                    let slot =
                        sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                            *timestamp,
                            slot_duration,
                        );

                    let storage_proof =
                        sp_transaction_storage_proof::registration::new_data_provider(
                            &*client_clone,
                            &parent,
                        )?;

                    Ok((slot, timestamp, storage_proof))
                }
            },
            force_authoring,
            backoff_authoring_blocks,
            babe_link,
            block_proposal_slot_portion: SlotProportion::new(0.5),
            max_block_proposal_slot_portion: None,
            telemetry: telemetry.as_ref().map(|x| x.handle()),
        };

        let babe = sc_consensus_babe::start_babe(babe_config)?;
        task_manager.spawn_essential_handle().spawn_blocking(
            "babe-proposer",
            Some("block-authoring"),
            babe,
        );
    }

    spawn_frontier_tasks(
        &task_manager,
        client.clone(),
        backend.clone(),
        frontier_backend,
        filter_pool,
        overrides,
        fee_history_cache,
        fee_history_cache_limit,
        sync_service.clone(),
        pubsub_notification_sinks,
    )
    .await;

    // Spawn authority discovery module.
    if role.is_authority() {
        let authority_discovery_role =
            sc_authority_discovery::Role::PublishAndDiscover(keystore_container.keystore());
        let dht_event_stream =
            network
                .event_stream("authority-discovery")
                .filter_map(|e| async move {
                    match e {
                        Event::Dht(e) => Some(e),
                        _ => None,
                    }
                });
        let (authority_discovery_worker, _service) =
            sc_authority_discovery::new_worker_and_service_with_config(
                sc_authority_discovery::WorkerConfig {
                    publish_non_global_ips: auth_disc_publish_non_global_ips,
                    ..Default::default()
                },
                client.clone(),
                network.clone(),
                Box::pin(dht_event_stream),
                authority_discovery_role,
                prometheus_registry.clone(),
            );

        task_manager.spawn_handle().spawn(
            "authority-discovery-worker",
            Some("networking"),
            authority_discovery_worker.run(),
        );
    }

    // if the node isn't actively participating in consensus then it doesn't
    // need a keystore, regardless of which protocol we use below.
    let keystore = if role.is_authority() {
        Some(keystore_container.keystore())
    } else {
        None
    };

    let grandpa_config = grandpa::Config {
        // FIXME #1578 make this available through chainspec
        gossip_duration: std::time::Duration::from_millis(gossip_duration_millis.unwrap_or(333)),
        justification_generation_period: 1, // https://github.com/paritytech/substrate/pull/14423#issuecomment-1633837906
        name: Some(name),
        observer_enabled: false,
        keystore,
        local_role: role.clone(),
        telemetry: telemetry.as_ref().map(|x| x.handle()),
        protocol_name: grandpa_protocol_name,
    };

    if enable_grandpa {
        // start the full GRANDPA voter
        // NOTE: non-authorities could run the GRANDPA observer protocol, but at
        // this point the full voter should provide better guarantees of block
        // and vote data availability than the observer. The observer has not
        // been tested extensively yet and having most nodes in a network run it
        // could lead to finality stalls.
        let grandpa_config = grandpa::GrandpaParams {
            config: grandpa_config,
            link: grandpa_link,
            network: network.clone(),
            sync: Arc::new(sync_service.clone()),
            notification_service: grandpa_notification_service,
            telemetry: telemetry.as_ref().map(|x| x.handle()),
            voting_rule: grandpa::VotingRulesBuilder::default().build(),
            prometheus_registry: prometheus_registry.clone(),
            shared_voter_state,
            offchain_tx_pool_factory: OffchainTransactionPoolFactory::new(transaction_pool.clone()),
        };

        // the GRANDPA voter task is considered infallible, i.e.
        // if it fails we take down the service with it.
        task_manager.spawn_essential_handle().spawn_blocking(
            "grandpa-voter",
            None,
            grandpa::run_grandpa_voter(grandpa_config)?,
        );
    }

    if enable_offchain_worker {
        task_manager.spawn_handle().spawn(
            "offchain-workers-runner",
            "offchain-work",
            sc_offchain::OffchainWorkers::new(sc_offchain::OffchainWorkerOptions {
                runtime_api_provider: client.clone(),
                keystore: Some(keystore_container.keystore()),
                offchain_db: backend.offchain_storage(),
                transaction_pool: Some(OffchainTransactionPoolFactory::new(
                    transaction_pool.clone(),
                )),
                network_provider: network.clone(),
                is_validator: role.is_authority(),
                enable_http_requests: true,
                custom_extensions: move |_| vec![],
            })
            .run(client.clone(), task_manager.spawn_handle())
            .boxed(),
        );
    }

    network_starter.start_network();
    Ok(NewFullBase {
        task_manager,
        client,
        network,
        sync: sync_service,
        transaction_pool,
        rpc_handlers,
    })
}

/// Builds a new service for a full client.
pub async fn new_full(
    config: Configuration,
    eth_config: &EthConfiguration,
    disable_hardware_benchmarks: bool,
    gossip_duration_millis: Option<u64>,
) -> Result<TaskManager, ServiceError> {
    new_full_base(
        config,
        eth_config,
        disable_hardware_benchmarks,
        gossip_duration_millis,
    )
    .await
    .map(|NewFullBase { task_manager, .. }| task_manager)
}

/// Build the import queue for the runtime (babe + grandpa).
#[allow(clippy::too_many_arguments)]
pub fn build_babe_grandpa_import_queue(
    client: Arc<FullClient>,
    backend: Arc<FullBackend>,
    config: &Configuration,
    eth_config: &EthConfiguration,
    task_manager: &TaskManager,
    telemetry: Option<TelemetryHandle>,
    grandpa_block_import: GrandpaBlockImport<FullClient>,
    offchain_tx_pool_factory: OffchainTransactionPoolFactory<Block>,
) -> Result<
    (
        BasicImportQueue,
        BabeWorkerHandle<Block>,
        BoxBlockImport,
        BabeLink<Block>,
    ),
    ServiceError,
> {
    let (babe_block_import, babe_link) = sc_consensus_babe::block_import(
        sc_consensus_babe::configuration(&*client)?,
        grandpa_block_import.clone(),
        client.clone(),
    )?;
    // The original implemenation is:
    //   let frontier_block_import = FrontierBlockImport::new(babe_block_import.clone(), client.clone());
    // However, because our runtime didn't contains frontier stuff at the beginning, FrontierBlockImport would reject
    // the early blocks until the runtime upgrade to support frontier. So, we use only babe_block_import here to make
    // it compatible with the old blocks.
    let frontier_block_import = babe_block_import.clone();
    let slot_duration = babe_link.config().slot_duration();
    let target_gas_price = eth_config.target_gas_price;
    let create_inherent_data_providers = move |_, ()| async move {
        let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
        let slot =
            sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                *timestamp,
                slot_duration,
            );
        let dynamic_fee = fp_dynamic_fee::InherentDataProvider(U256::from(target_gas_price));
        Ok((slot, timestamp, dynamic_fee))
    };

    let select_chain = sc_consensus::LongestChain::new(backend.clone());
    let (import_queue, babe_worker_handle) =
        sc_consensus_babe::import_queue(sc_consensus_babe::ImportQueueParams {
            block_import: frontier_block_import.clone(),
            justification_import: Some(Box::new(grandpa_block_import)),
            client,
            create_inherent_data_providers,
            spawner: &task_manager.spawn_essential_handle(),
            registry: config.prometheus_registry(),
            telemetry,
            link: babe_link.clone(),
            select_chain,
            offchain_tx_pool_factory,
        })
        .map_err::<ServiceError, _>(Into::into)?;

    Ok((
        import_queue,
        babe_worker_handle,
        Box::new(frontier_block_import),
        babe_link,
    ))
}

pub struct ChainOpsComponents {
    pub client: Arc<FullClient>,
    pub backend: Arc<FullBackend>,
    pub import_queue: BasicQueue<Block>,
    pub task_manager: TaskManager,
    pub frontier_backend: FrontierBackend,
}

pub fn new_chain_ops(
    config: &mut Configuration,
    eth_config: &EthConfiguration,
) -> Result<ChainOpsComponents, ServiceError> {
    config.keystore = sc_service::config::KeystoreConfig::InMemory;
    let sc_service::PartialComponents {
        client,
        backend,
        import_queue,
        task_manager,
        other,
        ..
    } = new_partial(config, eth_config, build_babe_grandpa_import_queue)?;
    Ok(ChainOpsComponents {
        client,
        backend,
        import_queue,
        task_manager,
        frontier_backend: other.frontier_backend,
    })
}

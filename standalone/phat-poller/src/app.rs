use crate::{
    args::{parse_hash, RunArgs},
    contracts::{SELECTOR_GET_USER_PROFILES, SELECTOR_POLL, SELECTOR_WORKFLOW_COUNT},
    instant::Instant,
    query::pink_query,
};

use std::{
    collections::{BTreeMap, VecDeque},
    future::Future,
    sync::{
        atomic::{AtomicI32, Ordering},
        Arc, Mutex, Weak,
    },
    time::Duration,
};

use anyhow::{anyhow, Context, Result};
use phaxt::AccountId;
use sp_core::{sr25519::Pair as KeyPair, H256};
use tokio::task::JoinHandle;
use tracing::{debug, error, info, instrument, warn, Instrument};

use serde::Serialize;

use phala_types::WorkerPublicKey;

pub type Config = RunArgs;
pub type TaskId = u64;
pub type ArcApp = Arc<App>;
type ContractId = H256;

#[derive(Clone, Serialize)]
struct Worker {
    pubkey: WorkerPublicKey,
    uri: String,
    probe_info: ProbeInfo,
}

impl Worker {
    fn latency(&self) -> Option<Duration> {
        let end = self.probe_info.probe_result.as_ref().ok()?;
        Some(end.duration_since(self.probe_info.probe_start))
    }
}

#[derive(Clone, Serialize)]
struct ProbeInfo {
    probe_start: Instant,
    probe_result: Result<Instant, String>,
}

#[derive(Default, Serialize)]
pub struct State {
    next_task_id: TaskId,
    workers: Vec<Worker>,
    running_tasks: BTreeMap<TaskId, Task>,
    history: VecDeque<Task>,
    last_probe_time: Option<Instant>,
    last_poll_time: Option<Instant>,
    last_poll_result: BTreeMap<ContractId, PollResult>,
}

#[derive(Default, Debug)]
struct Stats {
    contract_polled: AtomicI32,
    workflow_polled: AtomicI32,
    workflow_finished: AtomicI32,
    workflow_succeeded: AtomicI32,
    workflow_failed: AtomicI32,
}

impl Stats {
    fn inc_contract_polled(&self) {
        self.contract_polled.fetch_add(1, Ordering::Relaxed);
    }

    fn inc_polled(&self) {
        self.workflow_polled.fetch_add(1, Ordering::Relaxed);
    }
    fn inc_finished(&self) {
        self.workflow_finished.fetch_add(1, Ordering::Relaxed);
    }
    fn inc_succeeded(&self) {
        self.workflow_succeeded.fetch_add(1, Ordering::Relaxed);
    }
    fn inc_failed(&self) {
        self.workflow_failed.fetch_add(1, Ordering::Relaxed);
    }
}

pub struct App {
    weak_self: Weak<Self>,
    pub config: Config,
    pub state: Mutex<State>,
}

#[derive(Serialize)]
struct PollResult {
    worker: WorkerPublicKey,
    start_time: Instant,
    end_time: Option<Instant>,
    result: Result<(), String>,
}

#[derive(Serialize)]
struct Task {
    id: TaskId,
    name: String,
    description: String,
    start_time: Instant,
    end_time: Option<Instant>,
    duration: Option<Duration>,
    result: Option<Result<(), String>>,
}

pub fn create_app(config: Config) -> ArcApp {
    Arc::new_cyclic(|weak_app| App::new(config, weak_app.clone()))
}

impl App {
    fn new(config: Config, weak_self: Weak<Self>) -> Self {
        Self {
            weak_self,
            config,
            state: Default::default(),
        }
    }

    fn next_task_id(&self) -> u64 {
        let mut state = self.state.lock().unwrap();
        let id = state.next_task_id;
        state.next_task_id += 1;
        id
    }

    fn report_task_result<T>(&self, id: u64, result: &Result<T>) {
        let mut state = self.state.lock().unwrap();
        let Some(mut task) = state.running_tasks.remove(&id) else {
            warn!(id, "task gone");
            return;
        };
        task.end_time = Some(Instant::now());
        task.duration = Some(task.end_time.unwrap().duration_since(task.start_time));
        task.result = Some(match result {
            Ok(_) => Ok(()),
            Err(err) => Err(err.to_string()),
        });
        state.history.push_back(task);
        if state.history.len() > self.config.max_history {
            state.history.pop_front();
        }
    }

    fn spawn<T: Send + 'static>(
        &self,
        name: &str,
        description: &str,
        timeout: Duration,
        task: impl Future<Output = Result<T>> + Send + 'static,
    ) -> JoinHandle<Result<T>> {
        let id = self.next_task_id();
        let mut state = self.state.lock().unwrap();
        state.running_tasks.insert(
            id,
            Task {
                id,
                name: name.to_string(),
                description: description.to_string(),
                start_time: Instant::now(),
                end_time: None,
                duration: None,
                result: None,
            },
        );
        let app = self.weak_self.clone();
        tokio::spawn(
            track_task_result(app, id, async move {
                tokio::time::timeout(timeout, task)
                    .await
                    .or(Err(anyhow::anyhow!("timeout")))?
            })
            .instrument(tracing::info_span!("task", id, name)),
        )
    }

    #[instrument(skip(self), name = "probe")]
    async fn bg_update_worker_list(&self) {
        loop {
            info!("updating worker list");
            let start = Instant::now();
            self.state.lock().unwrap().last_probe_time = Some(start);
            if let Err(err) = self.update_worker_list().await {
                error!("failed to update worker list: {err}");
            }
            let rest_time = self.config.probe_interval.saturating_sub(start.elapsed());
            info!("{}ms elapsed", start.elapsed().as_millis());
            info!("next worker list update in {}s", rest_time.as_secs());
            tokio::time::sleep(rest_time).await;
        }
    }

    async fn update_worker_list(&self) -> Result<()> {
        info!("connecting to node {uri}", uri = self.config.node_uri);
        let chain_rpc = tokio::time::timeout(
            Duration::from_secs(5),
            phaxt::connect(&self.config.node_uri),
        )
        .await??;
        info!("connected to node {uri}", uri = self.config.node_uri);
        let workers = chain_rpc.get_workers(self.config.cluster_id).await?;
        info!("{} workers found", workers.len());
        if workers.is_empty() {
            warn!("no workers found");
            return Ok(());
        }
        let mut probe_tasks = Vec::with_capacity(workers.len());
        let mut probe_workers = Vec::with_capacity(workers.len());
        let mut workers_without_endpoints = Vec::new();

        let probe_start = Instant::now();
        let timeout = self.config.poll_timeout;
        for worker in workers {
            let endpoints = chain_rpc.get_endpoints(&worker).await?;
            debug!("worker {worker:?} endpoints: {endpoints:?}");
            let Some(uri) = endpoints
                .into_iter()
                .find(|url| url.starts_with("http://") || url.starts_with("https://"))
            else {
                workers_without_endpoints.push(worker);
                continue;
            };
            let handle = self.spawn(
                "probe",
                &format!(r#""uri":"{uri}"}}"#),
                timeout,
                probe_worker(Some(worker), uri.clone()),
            );
            probe_tasks.push(handle);
            probe_workers.push((worker, uri));
        }
        info!("probing {} workers", probe_tasks.len());
        let probe_results = futures::future::join_all(probe_tasks).await;
        let workers: Vec<_> = probe_workers
            .into_iter()
            .zip(probe_results)
            .map(|((pubkey, uri), result)| {
                let probe_result = match result {
                    Ok(Ok((_, instant))) => Ok(instant),
                    Ok(Err(err)) => Err(format!("{err}")),
                    Err(err) => Err(format!("{err}")),
                };
                Worker {
                    pubkey,
                    uri,
                    probe_info: ProbeInfo {
                        probe_start,
                        probe_result,
                    },
                }
            })
            .collect();
        info!("{} workers updated", workers.len());
        self.state.lock().unwrap().workers = workers;
        Ok(())
    }

    async fn probe_dev_worker(&self) -> Result<()> {
        let Some(uri) = self.config.dev_worker_uri.as_ref() else {
            warn!("no dev worker uri specified");
            return Ok(());
        };
        let handle = self.spawn(
            "probe",
            &format!("uri:{uri}"),
            self.config.probe_timeout,
            probe_worker(None, uri.clone()),
        );
        let t0 = Instant::now();
        let (pubkey, t1) = handle.await??;
        self.state.lock().unwrap().workers.push(Worker {
            pubkey,
            uri: uri.clone(),
            probe_info: ProbeInfo {
                probe_start: t0,
                probe_result: Ok(t1),
            },
        });
        Ok(())
    }
}

async fn track_task_result<T>(
    app: Weak<App>,
    task_id: TaskId,
    task: impl Future<Output = Result<T>>,
) -> Result<T> {
    let result = task.await;
    if let Some(app) = app.upgrade() {
        app.report_task_result(task_id, &result);
    }
    result
}

async fn probe_worker(
    pubkey: Option<WorkerPublicKey>,
    uri: String,
) -> Result<(WorkerPublicKey, Instant)> {
    debug!("probing worker {pubkey:?} at {uri}");
    let prpc = phactory_api::pruntime_client::new_pruntime_client_no_log(uri);
    let info = prpc.get_info(()).await?;
    let Some(public_key) = &info.public_key else {
        return Err(anyhow::anyhow!("worker is uninitialized"));
    };
    let public_key = WorkerPublicKey(parse_hash(public_key)?.0);
    if let Some(pubkey) = pubkey {
        if pubkey != public_key {
            return Err(anyhow::anyhow!("public key mismatch"));
        }
    }
    Ok((public_key, Instant::now()))
}

impl App {
    #[instrument(skip(self), name = "poll")]
    async fn bg_poll_contracts(&self) {
        loop {
            info!("poll contracts begin");
            let start = Instant::now();
            let stats = Arc::new(Stats::default());
            self.state.lock().unwrap().last_poll_time = Some(start);
            let result = tokio::time::timeout(
                self.config.poll_timeout_overall,
                self.poll_contracts(stats.clone()),
            )
            .await;
            match result {
                Err(_) => {
                    warn!("poll contracts timeout");
                }
                Ok(Err(err)) => {
                    warn!("failed to poll contracts: {err}");
                }
                Ok(Ok(())) => {}
            }
            let rest_time = self.config.poll_interval.saturating_sub(start.elapsed());
            info!("{}ms elapsed", start.elapsed().as_millis());
            info!("stats: {stats:?}");
            info!("next poll in {}s", rest_time.as_secs());
            tokio::time::sleep(rest_time).await;
        }
    }

    async fn poll_contracts(&self, stats: Arc<Stats>) -> Result<()> {
        let mut live_workers = self
            .state
            .lock()
            .unwrap()
            .workers
            .iter()
            .filter(|worker| worker.latency().is_some())
            .cloned()
            .collect::<Vec<_>>();

        if live_workers.is_empty() {
            warn!("no live workers available");
            return Ok(());
        }
        live_workers.sort_by_key(|worker| worker.latency().unwrap_or(Duration::MAX));
        let top_workers = live_workers
            .into_iter()
            .take(self.config.use_top_workers)
            .collect::<Vec<_>>();
        info!("top workers:");
        for worker in &top_workers {
            info!(latency=?worker.latency().unwrap_or(Duration::MAX), "  {}", worker.pubkey);
        }
        let worker = &top_workers[0];
        let contracts: Vec<(AccountId, ContractId)> = tokio::time::timeout(
            self.config.poll_timeout,
            pink_query::<_, crate::contracts::UserProfilesResponse>(
                &worker.pubkey,
                &worker.uri,
                self.config.factory_contract,
                SELECTOR_GET_USER_PROFILES,
                (),
                &self.config.caller,
            ),
        )
        .await
        .context("Get profiles timeout")??
        .or(Err(anyhow::anyhow!("query failed")))?;

        let mut poll_handles = Vec::with_capacity(contracts.len());

        let mut worker_index = 0;
        for (user, profile) in contracts {
            // call contract poll with workers in robin round
            let worker = &top_workers[worker_index];
            debug!(
                "adding poll, user={user}, profile={profile:?}, worker={}",
                worker.pubkey
            );
            stats.inc_contract_polled();
            let handle = self.spawn(
                "poll",
                &format!(
                    r#"{{"worker":"{}","contract":"{profile:?}"}}"#,
                    worker.pubkey
                ),
                self.config.poll_timeout.saturating_mul(2),
                poll_contract(
                    worker.clone(),
                    profile,
                    self.weak_self.clone(),
                    self.config.caller.clone(),
                    stats.clone(),
                ),
            );
            poll_handles.push(handle);
            worker_index = (worker_index + 1) % top_workers.len();
        }
        info!(count = poll_handles.len(), "polling contracts");
        let poll_results = futures::future::join_all(poll_handles).await;
        info!(count = poll_results.len(), "contracts polled");
        Ok(())
    }
}

async fn poll_contract(
    worker: Worker,
    profile: ContractId,
    weak_app: Weak<App>,
    caller: KeyPair,
    stats: Arc<Stats>,
) -> Result<()> {
    let pubkey = worker.pubkey;
    let start_time = Instant::now();
    let result = poll_contract_inner(worker, profile, weak_app.clone(), caller, stats).await;
    if let Some(app) = weak_app.upgrade() {
        let result = match &result {
            Ok(_) => Ok(()),
            Err(err) => Err(format!("{err}")),
        };
        app.state.lock().unwrap().last_poll_result.insert(
            profile,
            PollResult {
                worker: pubkey,
                start_time,
                end_time: Some(Instant::now()),
                result,
            },
        );
    }
    result
}

async fn poll_contract_inner(
    worker: Worker,
    profile: ContractId,
    weak_app: Weak<App>,
    caller: KeyPair,
    stats: Arc<Stats>,
) -> Result<()> {
    let count = pink_query::<(), u64>(
        &worker.pubkey,
        &worker.uri,
        profile,
        SELECTOR_WORKFLOW_COUNT,
        (),
        &caller,
    )
    .await?;

    let app = weak_app
        .upgrade()
        .ok_or_else(|| anyhow::anyhow!("app gone"))?;
    let mut handles = vec![];
    for i in 0..count {
        stats.inc_polled();
        let handle = app.spawn(
            "workflow",
            &format!(
                r#"{{"worker":"{}","contract":"{profile:?}"}}"#,
                worker.pubkey
            ),
            app.config.poll_timeout,
            poll_workflow(worker.clone(), profile, i, caller.clone(), stats.clone()),
        );
        handles.push(handle);
    }
    info!(count = handles.len(), ?profile, "polling workflows");
    let results = futures::future::join_all(handles).await;
    let errors = results
        .into_iter()
        .enumerate()
        .filter(|(_, r)| !matches!(r, Ok(Ok(_))))
        .collect::<Vec<_>>();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Failed flows: {:?}", errors))
    }
}

#[instrument(skip(worker, profile, caller, stats), name = "flow")]
async fn poll_workflow(
    worker: Worker,
    profile: ContractId,
    id: u64,
    caller: KeyPair,
    stats: Arc<Stats>,
) -> Result<()> {
    debug!("polling workflow");
    let result = pink_query::<_, crate::contracts::PollResponse>(
        &worker.pubkey,
        &worker.uri,
        profile,
        SELECTOR_POLL,
        (id,),
        &caller,
    )
    .await;
    debug!("result: {result:?}");
    if result.is_ok() {
        stats.inc_succeeded();
    } else {
        stats.inc_failed();
    }
    stats.inc_finished();
    result?.map_err(|e| anyhow!("{e:?}"))
}

impl App {
    pub async fn run(&self) -> Result<()> {
        if self.config.dev_worker_uri.is_some() {
            self.probe_dev_worker().await?;
            self.bg_poll_contracts().await;
        } else {
            tokio::join! {
                self.bg_update_worker_list(),
                self.bg_poll_contracts(),
            };
        }
        Ok(())
    }
}

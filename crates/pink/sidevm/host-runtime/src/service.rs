use crate::env::DynCacheOps;
use crate::{env::OcallAborted, run::WasmRun};
use crate::{ShortId, VmId};
use anyhow::{Context as _, Result};
use log::{debug, error, info, warn};
use pink_sidevm_env::query::AccountId;
use serde::{Deserialize, Serialize};
use std::future::Future;
use tokio::{
    sync::mpsc::{channel, Receiver, Sender},
    sync::oneshot::Sender as OneshotSender,
    task::JoinHandle,
};

pub type CommandSender = Sender<Command>;

#[derive(Debug)]
pub enum Report {
    VmTerminated { id: VmId, reason: ExitReason },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ExitReason {
    /// The program returned from `fn main`.
    Exited(i32),
    /// Stopped by an external Stop command.
    Stopped,
    /// The input channel has been closed, likely caused by a Stop command.
    InputClosed,
    /// The program panicked.
    Panicked,
    /// The task future has beed dropped, likely caused by a Stop command.
    Cancelled,
    /// Terminated due to gas checking.
    OcallAborted(OcallAborted),
    /// When a previous running instance restored from a checkpoint.
    Restore,
}

pub enum Command {
    // Stop the side VM instance.
    Stop,
    // Send a sidevm message to the instance.
    PushMessage(Vec<u8>),
    // Push a sidevm message to the instance.
    PushQuery {
        origin: AccountId,
        payload: Vec<u8>,
        reply_tx: OneshotSender<Vec<u8>>,
    },
}

pub struct ServiceRun {
    runtime: tokio::runtime::Runtime,
    report_rx: Receiver<Report>,
}

pub struct Spawner {
    runtime_handle: tokio::runtime::Handle,
    report_tx: Sender<Report>,
}

pub fn service() -> (ServiceRun, Spawner) {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();
    let runtime_handle = runtime.handle().clone();
    let (report_tx, report_rx) = channel(100);
    let run = ServiceRun { runtime, report_rx };
    let spawner = Spawner {
        runtime_handle,
        report_tx,
    };
    (run, spawner)
}

impl ServiceRun {
    pub fn blocking_run(mut self, mut event_handler: impl FnMut(Report)) {
        self.runtime.block_on(async {
            loop {
                match self.report_rx.recv().await {
                    None => {
                        info!(target: "sidevm", "The report channel is closed. Exiting service thread.");
                        break;
                    }
                    Some(report) => {
                        event_handler(report);
                    }
                }
            }
        })
    }
}

impl Spawner {
    pub fn start(
        &self,
        wasm_bytes: &[u8],
        max_memory_pages: u32,
        id: VmId,
        gas: u128,
        gas_per_breath: u128,
        cache_ops: DynCacheOps,
    ) -> Result<(CommandSender, JoinHandle<ExitReason>)> {
        let (cmd_tx, mut cmd_rx) = channel(128);
        let (mut wasm_run, env) =
            WasmRun::run(wasm_bytes, max_memory_pages, id, gas_per_breath, cache_ops)
                .context("Failed to create sidevm instance")?;
        env.set_gas(gas);
        let spawner = self.runtime_handle.clone();
        let handle = self.runtime_handle.spawn(async move {
            let vmid = ShortId(&id);
            loop {
                tokio::select! {
                    cmd = cmd_rx.recv() => {
                        match cmd {
                            None => {
                                info!(target: "sidevm", "[{vmid}] The command channel is closed. Exiting...");
                                break ExitReason::InputClosed;
                            }
                            Some(Command::Stop) => {
                                info!(target: "sidevm", "[{vmid}] Received stop command. Exiting...");
                                break ExitReason::Stopped;
                            }
                            Some(Command::PushMessage(msg)) => {
                                debug!(target: "sidevm", "[{vmid}] Sending message to sidevm.");
                                let push = env.push_message(msg);
                                spawner.spawn(async move {
                                    let vmid = ShortId(&id);
                                    match push.await {
                                        Ok(_) => {}
                                        Err(e) => {
                                            error!(target: "sidevm", "[{vmid}] Failed to send message to sidevm: {}", e);
                                        }
                                    }
                                });
                            }
                            Some(Command::PushQuery{ origin, payload, reply_tx }) => {
                                debug!(target: "sidevm", "[{vmid}] Pushing query to sidevm.");
                                let push = env.push_query(origin, payload, reply_tx);
                                spawner.spawn(async move {
                                    let vmid = ShortId(&id);
                                    match push.await {
                                        Ok(_) => {}
                                        Err(e) => {
                                            error!(target: "sidevm", "[{vmid}] Failed to push query to sidevm: {}", e);
                                        }
                                    }
                                });
                            }
                        }
                    }
                    rv = &mut wasm_run => {
                        match rv {
                            Ok(ret) => {
                                info!(target: "sidevm", "[{vmid}] The sidevm instance exited with {} normally.", ret);
                                break ExitReason::Exited(ret);
                            }
                            Err(err) => {
                                info!(target: "sidevm", "[{vmid}] The sidevm instance exited with error: {}", err);
                                match err.downcast::<crate::env::OcallAborted>() {
                                    Ok(err) => {
                                        break ExitReason::OcallAborted(err);
                                    }
                                    Err(_) => {
                                        break ExitReason::Panicked;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
        let report_tx = self.report_tx.clone();
        let handle = self.runtime_handle.spawn(async move {
            let vmid = ShortId(&id);
            let reason = match handle.await {
                Ok(r) => r,
                Err(err) => {
                    warn!(target: "sidevm", "[{vmid}] The sidevm instance exited with error: {}", err);
                    if err.is_cancelled() {
                        ExitReason::Cancelled
                    } else {
                        ExitReason::Panicked
                    }
                }
            };
            if let Err(err) = report_tx.send(Report::VmTerminated { id, reason }).await {
                warn!(target: "sidevm", "[{vmid}] Failed to send report to sidevm service: {}", err);
            }
            reason
        });
        Ok((cmd_tx, handle))
    }

    pub fn spawn(&self, fut: impl Future<Output = ()> + Send + 'static) -> JoinHandle<()> {
        self.runtime_handle.spawn(fut)
    }
}

use crate::env::CacheOps;
use crate::VmId;
use crate::{env::GasError, run::WasmRun};
use anyhow::{Context as _, Result};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::future::Future;
use tokio::{
    sync::mpsc::{channel, Receiver, Sender},
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
    GasError(GasError),
    /// When a previous running instance restored from a checkpoint.
    Restore,
}

pub enum Command {
    // Stop the side VM instance.
    Stop,
    // Send a sidevm message to the instance.
    PushMessage(Vec<u8>),
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
        cache_ops: CacheOps,
    ) -> Result<(CommandSender, JoinHandle<ExitReason>)> {
        let (cmd_tx, mut cmd_rx) = channel(100);
        let (mut wasm_run, env) =
            WasmRun::run(wasm_bytes, max_memory_pages, id, gas_per_breath, cache_ops)
                .context("Failed to create sidevm instance")?;
        env.set_gas(gas);
        let handle = self.runtime_handle.spawn(async move {
            loop {
                tokio::select! {
                    cmd = cmd_rx.recv() => {
                        match cmd {
                            None => {
                                info!(target: "sidevm", "The command channel is closed. Exiting...");
                                break ExitReason::InputClosed;
                            }
                            Some(Command::Stop) => {
                                info!(target: "sidevm", "Received stop command. Exiting...");
                                break ExitReason::Stopped;
                            }
                            Some(Command::PushMessage(msg)) => {
                                debug!(target: "sidevm", "Sending message to sidevm.");
                                match env.push_message(msg).await {
                                    Ok(_) => {}
                                    Err(e) => {
                                        error!(target: "sidevm", "Failed to send message to sidevm: {}", e);
                                        break ExitReason::Panicked;
                                    }
                                }
                            }
                        }
                    }
                    rv = &mut wasm_run => {
                        match rv {
                            Ok(ret) => {
                                info!(target: "sidevm", "The sidevm instance exited with {} normally.", ret);
                                break ExitReason::Exited(ret);
                            }
                            Err(err) => {
                                info!(target: "sidevm", "The sidevm instance exited with error: {}", err);
                                match err.downcast::<crate::env::GasError>() {
                                    Ok(err) => {
                                        break ExitReason::GasError(err);
                                    }
                                    Err(_) => {
                                        // TODO.kevin: Restart the instance?
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
            let reason = match handle.await {
                Ok(r) => r,
                Err(err) => {
                    warn!(target: "sidevm", "The sidevm instance exited with error: {}", err);
                    if err.is_cancelled() {
                        ExitReason::Cancelled
                    } else {
                        ExitReason::Panicked
                    }
                }
            };
            if let Err(err) = report_tx.send(Report::VmTerminated { id, reason }).await {
                warn!(target: "sidevm", "Failed to send report to sidevm service: {}", err);
            }
            reason
        });
        Ok((cmd_tx, handle))
    }

    pub fn spawn(&self, fut: impl Future<Output = ()> + Send + 'static) -> JoinHandle<()> {
        self.runtime_handle.spawn(fut)
    }
}

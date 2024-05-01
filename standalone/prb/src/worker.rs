use serde::{Deserialize, Serialize};

pub enum WorkerLifecycleCommand {
    ShouldRestart,
    ShouldForceRegister,
    ShouldUpdateEndpoint(Vec<String>),
    ShouldTakeCheckpoint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkerLifecycleState {
    Starting,
    Synchronizing,
    Preparing,
    Working,
    GatekeeperWorking,

    HasError(String),
    Restarting,
    Disabled,
}
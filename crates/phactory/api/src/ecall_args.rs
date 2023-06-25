use alloc::string::String;
use parity_scale_codec::{Decode, Encode};
use core::time::Duration;

#[derive(Debug, Encode, Decode, Default, Clone)]
pub struct InitArgs {
    /// The GK master key sealing path.
    pub sealing_path: String,

    /// The PRuntime persistent data storing path
    pub storage_path: String,

    /// Whether start to benchmark at start.
    pub init_bench: bool,

    /// The App version.
    pub version: String,

    /// The git commit hash which this binary was built from.
    pub git_revision: String,

    /// Enable checkpoint
    pub enable_checkpoint: bool,

    /// Checkpoint interval in seconds
    pub checkpoint_interval: u64,

    /// Remove corrupted checkpoint so that pruntime can restart to continue to load others.
    pub remove_corrupted_checkpoint: bool,

    /// Max number of checkpoint files kept
    pub max_checkpoint_files: u32,

    /// Number of cores used to run phat contracts
    pub cores: u32,

    /// The public rpc port with acl enabled
    pub public_port: Option<u16>,

    /// Only sync blocks into pruntime without dispatching messages.
    pub safe_mode_level: u8,

    /// Disable the RCU policy to update the Phactory state.
    pub no_rcu: bool,

    /// The timeout of getting the attestation report.
    pub ra_timeout: Duration,
}

pub use phala_git_revision::git_revision;

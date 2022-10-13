use alloc::string::{String, ToString};
use parity_scale_codec::{Decode, Encode};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Encode, Decode, Default, Clone)]
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

    /// Run the database garbage collection at given interval in blocks
    #[cfg_attr(feature = "serde", serde(default))]
    pub gc_interval: chain::BlockNumber,

    /// Number of cores used to run fat contracts
    pub cores: u32,

    /// The public rpc port with acl enabled
    pub public_port: Option<u16>,
}

pub fn git_revision() -> String {
    env!("PHALA_GIT_REVISION").to_string()
}


use parity_scale_codec::{Encode, Decode};
use alloc::string::{String, ToString};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Encode, Decode, Default, Clone)]
pub struct InitArgs {
    /// The GK master key sealing path.
    pub sealing_path: String,

    /// The log filter string to pass to env_logger.
    pub log_filter: String,

    /// Whether start to benchmark at start.
    pub init_bench: bool,

    /// The App version.
    pub version: String,

    /// The git commit hash which this binary was built from.
    pub git_revision: String,

    /// Geoip database path
    pub geoip_city_db: String,

    /// Enable checkpoint
    pub enable_checkpoint: bool,

    /// Checkpoint interval in seconds
    pub checkpoint_interval: u64,

    /// Remove corrupted checkpoint so that pruntime can restart to continue to load others.
    pub remove_corrupted_checkpoint: bool,
}

pub fn git_revision() -> String {
    env!("PHALA_GIT_REVISION").to_string()
}

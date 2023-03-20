use alloc::string::{String, ToString};
use parity_scale_codec::{Decode, Encode};

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

    /// Dump checkpoint for different key
    pub dump_checkpoint_for_key: Option<String>,

    /// Checkpoint interval in seconds
    pub checkpoint_interval: u64,

    /// Remove corrupted checkpoint.
    pub remove_corrupted_checkpoint: bool,

    /// Run the database garbage collection at given interval in blocks
    #[cfg_attr(feature = "serde", serde(default))]
    pub gc_interval: chain::BlockNumber,
}

pub fn git_revision() -> String {
    option_env!("PHALA_GIT_REVISION").unwrap_or("").to_string()
}

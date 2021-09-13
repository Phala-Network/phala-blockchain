
use parity_scale_codec::{Encode, Decode};
use alloc::string::{String, ToString};

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
}

pub fn git_revision() -> String {
    option_env!("PHALA_GIT_REVISION").unwrap_or("").to_string()
}

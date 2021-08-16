
use parity_scale_codec::{Encode, Decode};
use alloc::string::String;

#[derive(Debug, Encode, Decode)]
pub struct InitArgs {
    /// The GK master key sealing path
    pub sealing_path: String,

    /// The log filter string to pass to env_logger.
    pub log_filter: String,
}

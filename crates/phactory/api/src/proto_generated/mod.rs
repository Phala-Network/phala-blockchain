mod protos_codec_extensions;
#[allow(clippy::derive_partial_eq_without_eq, clippy::let_unit_value)]
mod pruntime_rpc;

pub use protos_codec_extensions::*;
pub use pruntime_rpc::*;

pub const PROTO_DEF: &str = include_str!("../../proto/pruntime_rpc.proto");

/// Helper struct used to compat the output of `get_info` for logging.
#[derive(Debug)]
pub struct Info<'a> {
    pub reg: bool,
    pub hdr: u32,
    pub phdr: u32,
    pub blk: u32,
    pub dev: bool,
    pub msgs: u64,
    pub ver: &'a str,
    pub git: &'a str,
    pub rmem: u64,
    pub mpeak: u64,
    pub rpeak: u64,
    pub rspike: u64,
    pub mfree: u64,
    pub whdr: bool,
    pub cluster: u64,
    pub gblk: u32,
}

impl PhactoryInfo {
    pub fn debug_info(&self) -> Info {
        let mem = self.memory_usage.clone().unwrap_or_default();
        Info {
            reg: self.registered,
            hdr: self.headernum,
            phdr: self.para_headernum,
            blk: self.blocknum,
            dev: self.dev_mode,
            msgs: self.pending_messages,
            ver: &self.version,
            git: &self.git_revision[0..8],
            rmem: mem.rust_used,
            rpeak: mem.rust_peak_used,
            rspike: mem.rust_spike,
            mpeak: mem.total_peak_used,
            mfree: mem.free,
            whdr: self.waiting_for_paraheaders,
            cluster: self
                .system
                .as_ref()
                .map(|s| s.number_of_clusters)
                .unwrap_or_default(),
            gblk: self.system.as_ref().map(|s| s.genesis_block).unwrap_or(0),
        }
    }
}

#[cfg(test)]
mod tests;

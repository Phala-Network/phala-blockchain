mod protos_codec_extensions;
#[allow(clippy::derive_partial_eq_without_eq)]
mod pruntime_rpc;

pub use protos_codec_extensions::*;
pub use pruntime_rpc::*;

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
    pub whdr: bool,
    pub cluster: bool,
    pub gblk: u32,
}

impl PhactoryInfo {
    pub fn debug_info(&self) -> Info {
        let (rmem, mpeak, rpeak) = match &self.memory_usage {
            Some(m) => (m.rust_used, m.total_peak_used, m.rust_peak_used),
            None => (0, 0, 0),
        };
        Info {
            reg: self.registered,
            hdr: self.headernum,
            phdr: self.para_headernum,
            blk: self.blocknum,
            dev: self.dev_mode,
            msgs: self.pending_messages,
            ver: &self.version,
            git: &self.git_revision[0..8],
            rmem,
            mpeak,
            rpeak,
            whdr: self.waiting_for_paraheaders,
            cluster: self
                .system
                .as_ref()
                .map(|s| s.number_of_clusters > 0)
                .unwrap_or(false),
            gblk: self.system.as_ref().map(|s| s.genesis_block).unwrap_or(0),
        }
    }
}

#[cfg(test)]
mod tests;

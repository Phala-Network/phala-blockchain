//! Platform abstraction layer for Trusted Execution Environments

use std::fmt::Debug;
use std::path::Path;

pub trait ErrorType: Debug + Into<anyhow::Error> {}
impl<T: Debug + Into<anyhow::Error>> ErrorType for T {}

pub trait Sealing {
    type SealError: ErrorType;
    type UnsealError: ErrorType;

    fn seal_data(&self, path: impl AsRef<Path>, data: &[u8]) -> Result<(), Self::SealError>;
    fn unseal_data(&self, path: impl AsRef<Path>) -> Result<Option<Vec<u8>>, Self::UnsealError>;
}

pub trait RA {
    type Error: ErrorType;
    fn create_attestation_report(&self, data: &[u8]) -> Result<(String, String, String), Self::Error>;
    fn quote_test(&self) -> Result<(), Self::Error>;
}

pub struct MemoryUsage {
    pub total_peak_used: usize,
    pub rust_used: usize,
    pub rust_peak_used: usize,
}

pub trait MemoryStats {
    fn memory_usage(&self) -> MemoryUsage;
}

pub trait Machine {
    fn machine_id(&self) -> Vec<u8>;
    fn cpu_core_num(&self) -> u32;
    fn cpu_feature_level(&self) -> u32;
}

pub trait ProtectedFileSystem {
    type IoError: ErrorType;
    type ReadFile: std::io::Read;
    type WriteFile: std::io::Write;

    fn open_protected_file(&self, path: impl AsRef<Path>, key: &[u8]) -> Result<Option<Self::ReadFile>, Self::IoError>;
    fn create_protected_file(&self, path: impl AsRef<Path>, key: &[u8]) -> Result<Self::WriteFile, Self::IoError>;
}

pub struct AppVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

pub trait AppInfo {
    fn app_version() -> AppVersion;
}

pub trait Platform: Sealing + RA + Machine + MemoryStats + ProtectedFileSystem + AppInfo + Clone {}
impl<T: Sealing + RA + Machine + MemoryStats + ProtectedFileSystem + AppInfo + Clone> Platform for T {}

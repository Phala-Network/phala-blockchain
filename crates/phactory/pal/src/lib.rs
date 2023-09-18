//! Platform abstraction layer for Trusted Execution Environments

use core::time::Duration;
use std::fmt::Debug;
use std::path::Path;

use phala_types::AttestationProvider;

pub use phactory_api::prpc::MemoryUsage;

pub trait ErrorType: Debug + Into<anyhow::Error> {}
impl<T: Debug + Into<anyhow::Error>> ErrorType for T {}

pub struct UnsealedData {
    pub data: Vec<u8>,
    pub svn: Vec<u8>,
}

pub trait Sealing {
    type SealError: ErrorType;
    type UnsealError: ErrorType;

    fn current_svn(&self) -> Result<Vec<u8>, Self::SealError>;
    fn seal_data(
        &self,
        path: impl AsRef<Path>,
        data: &[u8],
        svn: Option<&[u8]>,
    ) -> Result<(), Self::SealError>;
    fn unseal_data(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<Option<UnsealedData>, Self::UnsealError>;
}

pub trait RA {
    type Error: ErrorType;
    fn create_attestation_report(
        &self,
        provider: Option<AttestationProvider>,
        data: &[u8],
        timeout: Duration,
    ) -> Result<Vec<u8>, Self::Error>;
    fn quote_test(&self, provider: Option<AttestationProvider>) -> Result<(), Self::Error>;
    fn measurement(&self) -> Option<Vec<u8>>;
}

pub trait MemoryStats {
    fn memory_usage(&self) -> MemoryUsage;
}

pub trait Machine {
    fn machine_id(&self) -> Vec<u8>;
    fn cpu_core_num(&self) -> u32;
    fn cpu_feature_level(&self) -> u32;
}

pub struct AppVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

pub trait AppInfo {
    fn app_version() -> AppVersion;
}

pub trait Platform:
    Sealing + RA + Machine + MemoryStats + AppInfo + Clone + Send + 'static
{
}
impl<T: Sealing + RA + Machine + MemoryStats + AppInfo + Clone + Send + 'static> Platform for T {}

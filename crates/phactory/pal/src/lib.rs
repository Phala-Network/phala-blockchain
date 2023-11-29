//! Platform abstraction layer for Trusted Execution Environments

use std::fmt::Debug;
use std::path::Path;
use core::time::Duration;

use phala_types::AttestationProvider;

pub use phactory_api::prpc::MemoryUsage;

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
    fn create_attestation_report(
        &self,
        provider: Option<AttestationProvider>,
        data: &[u8],
        timeout: Duration,
    ) -> Result<Vec<u8>, Self::Error>;
    fn quote_test(&self, provider: Option<AttestationProvider>) -> Result<(), Self::Error>;
    fn measurement(&self) -> Option<Vec<u8>>;
    fn supported_attestation_methods(&self) -> Vec<String>;
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

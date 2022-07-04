use log::info;
use std::alloc::System;

use phactory_pal::{AppInfo, AppVersion, Machine, MemoryStats, MemoryUsage, Sealing, RA};
use phala_allocator::StatSizeAllocator;
use std::io::ErrorKind;
use std::str::FromStr as _;

use crate::ra;

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub(crate) struct GraminePlatform;

impl Sealing for GraminePlatform {
    type SealError = std::io::Error;
    type UnsealError = std::io::Error;

    fn seal_data(
        &self,
        path: impl AsRef<std::path::Path>,
        data: &[u8],
    ) -> Result<(), Self::SealError> {
        std::fs::write(path, data)?;
        Ok(())
    }

    fn unseal_data(
        &self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<Option<Vec<u8>>, Self::UnsealError> {
        match std::fs::read(path) {
            Err(err) if matches!(err.kind(), ErrorKind::NotFound) => Ok(None),
            other => other.map(Some),
        }
    }
}

impl RA for GraminePlatform {
    type Error = anyhow::Error;

    fn create_attestation_report(
        &self,
        data: &[u8],
    ) -> Result<(String, String, String), Self::Error> {
        // TODO.kevin: move the key out of the binary?
        const IAS_API_KEY_STR: &str = env!("IAS_API_KEY");
        ra::create_attestation_report(data, IAS_API_KEY_STR)
    }

    fn quote_test(&self) -> Result<(), Self::Error> {
        ra::create_quote_vec(&[0u8; 64]).map(|_| ())
    }
}

impl Machine for GraminePlatform {
    fn machine_id(&self) -> Vec<u8> {
        // TODO.kevin.must
        vec![]
    }

    fn cpu_core_num(&self) -> u32 {
        num_cpus::get() as _
    }

    fn cpu_feature_level(&self) -> u32 {
        let mut cpu_feature_level: u32 = 1;
        if is_x86_feature_detected!("avx2") {
            info!("CPU Support AVX2");
            cpu_feature_level += 1;

            if is_x86_feature_detected!("avx512f") {
                info!("CPU Support AVX512");
                cpu_feature_level += 1;
            }
        }
        cpu_feature_level
    }
}

#[global_allocator]
static ALLOCATOR: StatSizeAllocator<System> = StatSizeAllocator::new(System);

impl MemoryStats for GraminePlatform {
    fn memory_usage(&self) -> MemoryUsage {
        let stats = ALLOCATOR.stats();
        MemoryUsage {
            total_peak_used: 0,
            rust_used: stats.current_used,
            rust_peak_used: stats.peak_used,
        }
    }
}

impl AppInfo for GraminePlatform {
    fn app_version() -> AppVersion {
        let ver = version::Version::from_str(version::version!()).unwrap();
        AppVersion {
            major: ver.major,
            minor: ver.minor,
            patch: ver.patch,
        }
    }
}

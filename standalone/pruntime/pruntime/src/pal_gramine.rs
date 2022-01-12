use log::info;
use std::alloc::System;

use phactory_pal::{Machine, MemoryStats, MemoryUsage, ProtectedFileSystem, Sealing, RA};
use phala_allocator::StatSizeAllocator;

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub(crate) struct GraminePlatform;

impl Sealing for GraminePlatform {
    type SealError = anyhow::Error;
    type UnsealError = anyhow::Error;

    fn seal_data(
        &self,
        path: impl AsRef<std::path::Path>,
        data: &[u8],
    ) -> Result<(), Self::SealError> {
        todo!()
    }

    fn unseal_data(
        &self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<Option<Vec<u8>>, Self::UnsealError> {
        // todo!()
        Ok(None)
    }
}

pub struct ProtectedFile;

impl std::io::Read for ProtectedFile {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        todo!()
    }
}

impl std::io::Write for ProtectedFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        todo!()
    }

    fn flush(&mut self) -> std::io::Result<()> {
        todo!()
    }
}

impl ProtectedFileSystem for GraminePlatform {
    type IoError = std::io::Error;

    type ReadFile = ProtectedFile;

    type WriteFile = ProtectedFile;

    fn open_protected_file(
        &self,
        path: impl AsRef<std::path::Path>,
        key: &[u8],
    ) -> Result<Option<Self::ReadFile>, Self::IoError> {
        todo!()
    }

    fn create_protected_file(
        &self,
        path: impl AsRef<std::path::Path>,
        key: &[u8],
    ) -> Result<Self::WriteFile, Self::IoError> {
        todo!()
    }
}

impl RA for GraminePlatform {
    type Error = anyhow::Error;

    fn create_attestation_report(
        &self,
        data: &[u8],
    ) -> Result<(String, String, String), Self::Error> {
        todo!("TODO.kevin")
    }

    fn quote_test(&self) -> Result<(), Self::Error> {
        todo!("TODO.kevin")
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
        // TODO.kevin.must: Is CPUID in gramine trustable?
        // Atom doesn't support AVX
        if is_x86_feature_detected!("avx2") {
            info!("CPU Support AVX2");
            cpu_feature_level += 1;

            // Customer-level Core doesn't support AVX512
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

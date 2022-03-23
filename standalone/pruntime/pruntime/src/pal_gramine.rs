use log::info;
use std::alloc::System;
use parity_scale_codec::Encode;

use phactory_pal::{Machine, MemoryStats, MemoryUsage, ProtectedFileSystem, Sealing, RA};
use phala_allocator::StatSizeAllocator;
use std::fs::File;

use crate::ias;

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
        std::fs::write(path, data)?;
        Ok(())
    }

    fn unseal_data(
        &self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<Option<Vec<u8>>, Self::UnsealError> {
        match std::fs::read(path) {
            Ok(data) => Ok(Some(data)),
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => Ok(None),
                _ => Err(err.into()),
            },
        }
    }
}

pub struct ProtectedFile(File);

impl std::io::Read for ProtectedFile {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.read(buf)
    }
}

impl std::io::Write for ProtectedFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}

impl ProtectedFileSystem for GraminePlatform {
    type IoError = std::io::Error;

    type ReadFile = ProtectedFile;

    type WriteFile = ProtectedFile;

    fn open_protected_file(
        &self,
        _path: impl AsRef<std::path::Path>,
        _key: &[u8],
    ) -> Result<Option<Self::ReadFile>, Self::IoError> {
        unimplemented!()
    }

    fn create_protected_file(
        &self,
        _path: impl AsRef<std::path::Path>,
        _key: &[u8],
    ) -> Result<Self::WriteFile, Self::IoError> {
        unimplemented!()
    }
}

impl RA for GraminePlatform {
    type Error = anyhow::Error;

    fn create_attestation_report(
        &self,
        data: &[u8],
    ) -> Result<Vec<u8>, Self::Error> {
        // TODO.kevin: move the key out of the binary?
        const IAS_API_KEY_STR: &str = env!("IAS_API_KEY");

        let (attn_report, sig, cert) = ias::create_attestation_report(data, IAS_API_KEY_STR)?;
        let attestation_report = phala_types::AttestationReport::SgxIas {
            ra_report: attn_report.as_bytes().to_vec(),
            signature: sig.as_bytes().to_vec(),
            raw_signing_cert: cert.as_bytes().to_vec(),
        };

        Ok(Encode::encode(&attestation_report))
    }

    fn quote_test(&self) -> Result<(), Self::Error> {
        ias::create_quote_vec(&[0u8; 64]).map(|_| ())
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

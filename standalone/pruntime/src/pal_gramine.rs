use log::info;
use std::alloc::System;
use parity_scale_codec::Encode;
use anyhow::anyhow;

use phactory_pal::{
    AppInfo, AppVersion, Machine, MemoryStats, MemoryUsage, ProtectedFileSystem, Sealing, RA,
};
use phala_allocator::StatSizeAllocator;
use std::fs::File;
use std::io::ErrorKind;
use std::str::FromStr as _;

use crate::ias;

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

impl ProtectedFileSystem for GraminePlatform {
    type IoError = std::io::Error;

    type ReadFile = File;

    type WriteFile = File;

    fn open_protected_file(
        &self,
        path: impl AsRef<std::path::Path>,
        _key: &[u8],
    ) -> Result<Option<Self::ReadFile>, Self::IoError> {
        let todo = "Kevin: Use the key to encrypt the file";
        // We currently use the mrenclave protected fs to store the data, so no need to encrypt twice.
        // Gramine has a plan to support set key for individual files. We can turn back to use the key
        // once gramine finish the feature.

        match std::fs::File::open(path) {
            Err(err) if matches!(err.kind(), ErrorKind::NotFound) => Ok(None),
            other => other.map(Some),
        }
    }

    fn create_protected_file(
        &self,
        path: impl AsRef<std::path::Path>,
        _key: &[u8],
    ) -> Result<Self::WriteFile, Self::IoError> {
        std::fs::File::create(path)
    }
}

impl RA for GraminePlatform {
    type Error = anyhow::Error;

    fn create_attestation_report(
        &self,
        provider: String,
        data: &[u8],
    ) -> Result<Vec<u8>, Self::Error> {
        match provider.as_str() {
            "ias" => {
                // TODO.kevin: move the key out of the binary?
                const IAS_API_KEY_STR: &str = env!("IAS_API_KEY");

                let (attn_report, sig, cert) = ias::create_attestation_report(data, IAS_API_KEY_STR)?;
                let attestation_report = phala_types::AttestationReport::SgxIas {
                    ra_report: attn_report.as_bytes().to_vec(),
                    signature: sig.as_bytes().to_vec(),
                    raw_signing_cert: cert.as_bytes().to_vec(),
                };

                Ok(Encode::encode(&attestation_report))
            },
            "opt-out" => {
                Ok(Encode::encode(&phala_types::AttestationReport::OptOut))
            },
            _ => {
                Err(anyhow!("Unknown attestation provider `{}`", provider))
            }
        }
    }

    fn quote_test(&self, provider: String) -> Result<(), Self::Error> {
        match provider.as_str() {
            "ias" => {
                ias::create_quote_vec(&[0u8; 64]).map(|_| ())
            },
            "opt-out" => {
                Ok(())
            },
            _ => {
                Err(anyhow!("Unknown attestation provider `{}`", provider))
            }
        }
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

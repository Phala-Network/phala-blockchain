use anyhow::anyhow;
use parity_scale_codec::Encode;
use std::alloc::System;
use tracing::info;

use phactory_pal::{AppInfo, AppVersion, Machine, MemoryStats, MemoryUsage, Sealing, RA};
use phala_allocator::StatSizeAllocator;
use std::io::ErrorKind;
use std::str::FromStr as _;
use std::time::Duration;

use crate::ias;

use phala_types::AttestationProvider;

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
        provider: Option<AttestationProvider>,
        data: &[u8],
        timeout: Duration,
    ) -> Result<Vec<u8>, Self::Error> {
        let report = match provider {
            Some(AttestationProvider::Ias) => {
                // TODO.kevin: move the key out of the binary?
                const IAS_API_KEY_STR: &str = env!("IAS_API_KEY");

                let (attn_report, sig, cert) =
                    ias::create_attestation_report(data, IAS_API_KEY_STR, timeout)?;
                Some(phala_types::AttestationReport::SgxIas {
                    ra_report: attn_report.as_bytes().to_vec(),
                    signature: sig,
                    raw_signing_cert: cert,
                })
            }
            Some(AttestationProvider::Dcap) => {
                Some(phala_types::AttestationReport::SgxDcapRawQuote {
                    quote: ias::create_quote_vec(data)?,
                })
            }
            None => None,
            _ => anyhow::bail!("Unknown attestation provider `{:?}`", provider),
        };
        Ok(Encode::encode(&report))
    }

    fn quote_test(&self, provider: Option<AttestationProvider>) -> Result<(), Self::Error> {
        match provider {
            Some(AttestationProvider::Ias | AttestationProvider::Dcap) => {
                ias::create_quote_vec(&[0u8; 64]).map(|_| ())
            }
            None => Ok(()),
            _ => Err(anyhow!("Unknown attestation provider `{:?}`", provider)),
        }
    }

    fn measurement(&self) -> Option<Vec<u8>> {
        if is_gramine() {
            sgx_api_lite::target_info()
                .map(|info| info.mr_enclave.m.to_vec())
                .ok()
        } else {
            None
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

    #[cfg(target_arch = "x86_64")]
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

    #[cfg(not(target_arch = "x86_64"))]
    fn cpu_feature_level(&self) -> u32 {
        1
    }
}

#[global_allocator]
static ALLOCATOR: StatSizeAllocator<System> = StatSizeAllocator::new(System);

impl MemoryStats for GraminePlatform {
    fn memory_usage(&self) -> MemoryUsage {
        let stats = ALLOCATOR.stats();
        MemoryUsage {
            total_peak_used: (vm_peak().unwrap_or_default() * 1024) as _,
            rust_used: stats.current as _,
            rust_peak_used: stats.peak as _,
            free: (mem_free().unwrap_or_default() * 1024) as _,
            rust_spike: stats.spike as _,
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

fn vm_peak() -> Option<usize> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    for line in status.lines() {
        if line.starts_with("VmPeak:") {
            let peak = line.split_ascii_whitespace().nth(1)?;
            return peak.parse().ok();
        }
    }
    None
}

fn mem_free() -> Option<usize> {
    let status = std::fs::read_to_string("/proc/meminfo").ok()?;
    for line in status.lines() {
        if line.starts_with("MemFree:") {
            let peak = line.split_ascii_whitespace().nth(1)?;
            return peak.parse().ok();
        }
    }
    None
}

pub(crate) fn is_gramine() -> bool {
    lazy_static::lazy_static! {
        static ref IS_GRAMINE: bool =
            std::path::Path::new("/dev/attestation/user_report_data").exists();
    }
    *IS_GRAMINE
}

pub(crate) fn print_target_info() {
    use hex_fmt::HexFmt;
    if is_gramine() {
        println!("Running in Gramine-SGX");
        let target_info = sgx_api_lite::target_info().expect("Failed to get target info");
        let report =
            sgx_api_lite::report(&target_info, &[0; 64]).expect("Failed to get sgx report");
        println!("mr_enclave  : 0x{}", HexFmt(&report.body.mr_enclave.m));
        println!("mr_signer   : 0x{}", HexFmt(&report.body.mr_signer.m));
        println!(
            "isv_svn     : 0x{:?}",
            HexFmt(report.body.isv_svn.to_ne_bytes())
        );
        println!(
            "isv_prod_id : 0x{:?}",
            HexFmt(report.body.isv_prod_id.to_ne_bytes())
        );
    } else {
        println!("Running in Native mode");
    }
    println!("git revision: {}", phala_git_revision::git_revision());
}

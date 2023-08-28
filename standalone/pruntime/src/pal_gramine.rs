use anyhow::anyhow;
use parity_scale_codec::{Decode, Encode};
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

#[derive(Encode, Decode)]
struct SealedData {
    isvsvn: u16,
    cpusvn: [u8; 16],
    iv: [u8; 12],
    data: Vec<u8>,
}

fn get_sealing_key(isvsvn: u16, cpusvn: [u8; 16]) -> anyhow::Result<[u8; 32]> {
    const KEY_ID: [u8; 32] = *b"pruntime-sealed-data-key-id\0\0\0\0\0";
    let key128 = sgx_api_lite::get_mrenclave_sealing_key(isvsvn, cpusvn, KEY_ID)
        .or(Err(anyhow!("Failed to get sealing key")))?;
    // phala_crypto API uses AES256, so extend the key to 256 bits
    let mut key256 = [0; 32];
    key256[..16].copy_from_slice(&key128[..]);
    Ok(key256)
}

fn sgx_seal_data(data: &[u8]) -> anyhow::Result<SealedData> {
    let this_target_info =
        sgx_api_lite::target_info().or(Err(anyhow!("Failed to get target info")))?;
    let report = sgx_api_lite::report(&this_target_info, &[0; 64])
        .or(Err(anyhow!("Failed to get SGX report")))?;
    let isvsvn = report.body.isv_svn;
    let cpusvn = report.body.cpu_svn.svn;
    let key = get_sealing_key(isvsvn, cpusvn)
        .or(Err(anyhow!("Failed to get sealing key")))?;
    let iv = phactory::generate_random_iv();
    let mut data = data.to_vec();
    phala_crypto::aead::encrypt(&iv, &key, &mut data).or(Err(anyhow!("Failed to encrypt data")))?;
    Ok(SealedData {
        isvsvn,
        cpusvn,
        iv,
        data,
    })
}

fn sgx_unseal_data(data: &SealedData) -> anyhow::Result<Vec<u8>> {
    let key = get_sealing_key(data.isvsvn, data.cpusvn)
        .or(Err(anyhow!("Failed to get sealing key")))?;
    let mut enccypted_data = data.data.clone();
    let decrypted = phala_crypto::aead::decrypt(&data.iv, &key, &mut enccypted_data[..])
        .or(Err(anyhow!("Failed to decrypt sealed data",)))?;
    Ok(decrypted.to_vec())
}

impl Sealing for GraminePlatform {
    type SealError = anyhow::Error;
    type UnsealError = anyhow::Error;

    fn seal_data(
        &self,
        path: impl AsRef<std::path::Path>,
        data: &[u8],
    ) -> Result<(), Self::SealError> {
        if !is_gramine() {
            std::fs::write(path, data)?;
            return Ok(());
        }
        info!("Sealing data to {:?}", path.as_ref());
        let data = sgx_seal_data(data)?;
        let encoded = data.encode();
        std::fs::write(path, encoded)?;
        Ok(())
    }

    fn unseal_data(
        &self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<Option<Vec<u8>>, Self::UnsealError> {
        match std::fs::read(path) {
            Err(err) if matches!(err.kind(), ErrorKind::NotFound) => Ok(None),
            Ok(data) => {
                if !is_gramine() {
                    return Ok(Some(data));
                }
                let data = SealedData::decode(&mut &data[..])?;
                let data = sgx_unseal_data(&data)?;
                Ok(Some(data))
            }
            Err(err) => Err(err.into()),
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
        match provider {
            Some(AttestationProvider::Ias) => {
                // TODO.kevin: move the key out of the binary?
                const IAS_API_KEY_STR: &str = env!("IAS_API_KEY");

                let (attn_report, sig, cert) =
                    ias::create_attestation_report(data, IAS_API_KEY_STR, timeout)?;
                let attestation_report = Some(phala_types::AttestationReport::SgxIas {
                    ra_report: attn_report.as_bytes().to_vec(),
                    signature: sig,
                    raw_signing_cert: cert,
                });

                Ok(Encode::encode(&attestation_report))
            }
            None => Ok(Encode::encode(&None::<AttestationProvider>)),
            _ => Err(anyhow!("Unknown attestation provider `{:?}`", provider)),
        }
    }

    fn quote_test(&self, provider: Option<AttestationProvider>) -> Result<(), Self::Error> {
        match provider {
            Some(AttestationProvider::Ias) => ias::create_quote_vec(&[0u8; 64]).map(|_| ()),
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
        println!("cpu_svn     : 0x{}", HexFmt(&report.body.cpu_svn.svn));
    } else {
        println!("Running in Native mode");
    }
    println!("git revision: {}", phala_git_revision::git_revision());
}

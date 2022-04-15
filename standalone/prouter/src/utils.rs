use anyhow::{anyhow, Result};
use std::path::PathBuf;

pub fn push(path: &PathBuf, filename: &str) -> PathBuf {
    let mut path_dup = path.clone();
    path_dup.push(filename);

    path_dup
}

pub fn pathbuf_to_string(path: PathBuf) -> Result<String> {
    let ret = path.into_os_string().into_string()
        .map_err(|e| anyhow!("failed path conversion: {:?}", e))?;

    Ok(ret)
}

pub fn push_str(path: &PathBuf, filename: &str) -> Result<String> {
    let mut path_dup = path.clone();
    path_dup.push(filename);

    pathbuf_to_string(path_dup)
}

pub fn format_traffic(bytes: u64) -> Result<String> {
    let mut traffic_str: String = "".to_string();
    let num_kbytes = bytes as f64 / 1024f64;
    if num_kbytes < 1024f64 {
        traffic_str = format!("{:.3} KiB", num_kbytes);
    } else if num_kbytes < 1024f64 * 1024f64 {
        traffic_str = format!("{:.3} MiB", num_kbytes / 1024f64);
    } else if num_kbytes < 1024f64 * 1024f64 * 1024f64 {
        traffic_str = format!("{:.3} GiB", num_kbytes / 1024f64 / 1024f64);
    }

    Ok(traffic_str)
}

pub fn format_bandwidth(bandwidth: u32) -> Result<String> {
    Ok(format!("{:.3} KiB/s", bandwidth as f32 / 1024f32))
}

use anyhow::{anyhow, Result};
use std::path::PathBuf;

pub fn get_relative_filepath(path: &PathBuf, filename: &str) -> PathBuf {
    let mut path_dup = path.clone();
    path_dup.push(filename);

    path_dup
}

pub fn pathbuf_to_string(path: PathBuf) -> Result<String> {
    let ret = match path.into_os_string().into_string() {
        Ok(d) => d,
        Err(e) => return Err(anyhow!("{:?}", e)),
    };

    Ok(ret)
}

pub fn get_relative_filepath_str(path: &PathBuf, filename: &str) -> Result<String> {
    let mut path_dup = path.clone();
    path_dup.push(filename);

    pathbuf_to_string(path_dup)
}
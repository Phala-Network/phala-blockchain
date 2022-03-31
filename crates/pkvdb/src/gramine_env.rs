use crate::disk_env::PosixDiskEnv;
use crate::env::Env;
use crate::env::FileLock;
use crate::env::Logger;
use crate::env::RandomAccess;
use crate::error::err;
use crate::error::Result;
use crate::error::StatusCode;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

const RENAME_SUFFIX: &str = ".rename";


pub struct GramineEnv(pub PosixDiskEnv);

impl GramineEnv {
    pub fn new() -> Self {
        GramineEnv(PosixDiskEnv::new())
    }
}

// except rename all methods will proxy to inner PosixDiskEnv
impl Env for GramineEnv {
    fn open_sequential_file(&self, p: &Path) -> Result<Box<dyn Read>> {
        self.0.open_sequential_file(p)
    }

    fn open_random_access_file(&self, p: &Path) -> Result<Box<dyn RandomAccess>> {
        self.0.open_random_access_file(p)
    }

    fn open_writable_file(&self, p: &Path) -> Result<Box<dyn Write>> {
        self.0.open_writable_file(p)
    }

    fn open_appendable_file(&self, p: &Path) -> Result<Box<dyn Write>> {
        self.0.open_appendable_file(p)
    }

    fn exists(&self, p: &Path) -> Result<bool> {
        self.0.exists(p)
    }

    fn children(&self, p: &Path) -> Result<Vec<PathBuf>> {
        self.0.children(p)
    }

    fn size_of(&self, p: &Path) -> Result<usize> {
        self.0.size_of(p)
    }

    fn delete(&self, p: &Path) -> Result<()> {
        self.0.delete(p)
    }

    fn mkdir(&self, p: &Path) -> Result<()> {
        self.0.mkdir(p)
    }

    fn rmdir(&self, p: &Path) -> Result<()> {
        self.0.rmdir(p)
    }

    fn lock(&self, p: &Path) -> Result<FileLock> {
        let lock = FileLock {
            id: p.to_str().unwrap().to_string(),
        };
        Ok(lock)
    }

    fn unlock(&self, l: FileLock) -> Result<()> {
        Ok(())
    }

    fn new_logger(&self, p: &Path) -> Result<Logger> {
        self.0.new_logger(p)
    }

    fn micros(&self) -> u64 {
        self.0.micros()
    }

    fn sleep_for(&self, micros: u32) {
        self.0.sleep_for(micros)
    }

    // specific rename used in gramine
    // FIXME: current implementation doesn't support the old file exists maybe cause some unkown
    // bug
    fn rename(&self, old: &Path, new: &Path) -> Result<()> {
        if let Ok(exists) = self.exists(new) {
            if exists {
                let _ = self.delete(new);
            }
        }
        let mut new_file = self.open_writable_file(new)?;
        let mut old_file = self.open_sequential_file(old)?;
        let mut buffer = Vec::new();
        if let Err(_) = old_file.read_to_end(&mut buffer) {
            let _ = self.delete(new);
            return err(StatusCode::Corruption, "failed to read data from old file");
        }
        if let Err(e) = new_file.write_all(&buffer) {
            let _ = self.delete(new);
            return err(StatusCode::Corruption, e.to_string().as_str())?;
        }
        let _ = self.delete(old);
        Ok(())
    }
}

use rusty_leveldb::env::{Env, FileLock, Logger, RandomAccess};
use rusty_leveldb::disk_env::PosixDiskEnv;
use rusty_leveldb::error::{ err, Result, StatusCode };
use std::io::{Read, Write};
use std::path::{Path, PathBuf};


pub struct GramineEnv(PosixDiskEnv);

impl GramineEnv {
    
    pub fn new() -> Self {
        GramineEnv(PosixDiskEnv::new())
    }
}

impl Env for GramineEnv {
    
    fn open_sequential_file(&self, p: &Path) -> Result<Box<dyn Read>> {
        self.0.open_sequential_file(p)
    }

    fn open_random_access_file(&self, p: &Path) -> Result<Box<dyn RandomAccess>> {
        self.0.open_random_access_file(p)
    }

    fn open_writeable_file(&self, p: &Path) -> Result<Box<dyn Write>> {
        self.0.open_writeable_file(p)
    }

    fn open_appendable_file(&self, p: &Path) -> Result<Box<dyn Write>> {
        self.0.open_appendable_file(p)
    }

    fn exists(&self, p:&Path) -> Result<bool> {
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

    fn rename(&self, old: &Path, new: &Path) -> Result<()> {
        if let Ok(exists) = self.exists(new) {
            if exists {
                let _ = self.delete(new);
            }
        }
        let mut old_file = self.open_sequential_file(old)?;
        let mut new_file = self.open_writeable_file(new)?;
        if let Err(e) = std::io::copy(&mut old_file, &mut new_file) {
            return err(StatueCode::Corruption, "failed to copyt data from old to new");
        }

        Ok(())
    }
}

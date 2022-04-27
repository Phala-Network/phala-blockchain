use super::cache;
use anyhow::Result;
use rusty_leveldb::DB;

pub struct CacheDB(pub DB);

impl CacheDB {
    pub fn open(path: &str) -> Result<Self> {
        Ok(CacheDB(DB::open(path, Default::default())?))
    }
}

impl cache::DB for CacheDB {
    fn put(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        self.0.put(key, value)?;
        Ok(())
    }
}

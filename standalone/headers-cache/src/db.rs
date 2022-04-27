use super::cache;
use anyhow::Result;
use rusty_leveldb::DB;

pub struct CacheDB(pub DB);

impl CacheDB {
    pub fn open(path: &str) -> Result<Self> {
        Ok(CacheDB(DB::open(path, Default::default())?))
    }

    pub fn get(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        self.0.get(key)
    }

    pub fn get_genesis(&mut self) -> Option<Vec<u8>> {
        self.get(b"genesis")
    }

    pub fn put_genesis(&mut self, value: &[u8]) -> Result<()> {
        self.0.put(b"genesis", value)?;
        Ok(())
    }
}

impl cache::DB for CacheDB {
    fn put(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        self.0.put(key, value)?;
        Ok(())
    }
}

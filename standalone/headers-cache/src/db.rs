use crate::BlockNumber;

use super::cache;
use anyhow::Result;
use rusty_leveldb::DB;

pub struct CacheDB(pub DB);

fn genesis_key(block_number: BlockNumber) -> [u8; 7 + std::mem::size_of::<BlockNumber>()] {
    let mut key = *b"genesis****";
    key[7..].copy_from_slice(&block_number.to_be_bytes());
    key
}

impl CacheDB {
    pub fn open(path: &str) -> Result<Self> {
        Ok(CacheDB(DB::open(path, Default::default())?))
    }

    pub fn get(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        self.0.get(key)
    }

    pub fn get_genesis(&mut self, block_number: BlockNumber) -> Option<Vec<u8>> {
        self.get(&genesis_key(block_number))
    }

    pub fn put_genesis(&mut self, value: &[u8], block_number: BlockNumber) -> Result<()> {
        self.0.put(&genesis_key(block_number), value)?;
        Ok(())
    }
}

impl cache::DB for CacheDB {
    fn put(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        self.0.put(key, value)?;
        Ok(())
    }
}

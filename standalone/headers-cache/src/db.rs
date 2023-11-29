use crate::BlockNumber;

use anyhow::Result;
use rocksdb::DB;
use std::{mem::size_of, sync::Arc};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Counters {
    pub header: Option<BlockNumber>,
    pub para_header: Option<BlockNumber>,
    pub storage_changes: Option<BlockNumber>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Metadata {
    pub genesis: Vec<BlockNumber>,
    pub recent_imported: Counters,
    pub higest: Counters,
    #[serde(default)]
    pub checked: Counters,
}

macro_rules! update_field {
    ($self:ident, $field:ident, $value:expr) => {{
        $self.recent_imported.$field = Some($value);
        let higest = $self.higest.$field.unwrap_or_default().max($value);
        $self.higest.$field = Some(higest);
    }};
}

impl Metadata {
    pub fn update_header(&mut self, block: BlockNumber) {
        update_field!(self, header, block);
    }

    pub fn update_para_header(&mut self, block: BlockNumber) {
        update_field!(self, para_header, block);
    }

    pub fn update_storage_changes(&mut self, block: BlockNumber) {
        update_field!(self, storage_changes, block);
    }

    pub fn put_genesis(&mut self, block: BlockNumber) {
        if !self.genesis.contains(&block) {
            self.genesis.push(block);
        }
    }
}

#[derive(Clone)]
pub struct CacheDB(Arc<DB>);

fn mk_key(prefix: u8, block_number: BlockNumber) -> [u8; size_of::<BlockNumber>() + 1] {
    let mut key = [prefix; size_of::<BlockNumber>() + 1];
    key[1..].copy_from_slice(&block_number.to_be_bytes());
    key
}

impl CacheDB {
    pub fn open(path: &str) -> Result<Self> {
        Ok(CacheDB(Arc::new(DB::open_default(path)?)))
    }

    pub fn flush(&self) -> Result<()> {
        self.0.flush()?;
        Ok(())
    }

    fn get(&self, prefix: u8, block: BlockNumber) -> Option<Vec<u8>> {
        self.0.get(mk_key(prefix, block)).ok().flatten()
    }

    fn put(&self, prefix: u8, block: BlockNumber, value: &[u8]) -> Result<()> {
        self.0.put(mk_key(prefix, block), value)?;
        Ok(())
    }

    pub fn get_header(&self, block: BlockNumber) -> Option<Vec<u8>> {
        self.get(b'h', block)
    }

    pub fn put_header(&self, block: BlockNumber, value: &[u8]) -> Result<()> {
        self.put(b'h', block, value)
    }

    pub fn get_para_header(&self, block: BlockNumber) -> Option<Vec<u8>> {
        self.get(b'p', block)
    }

    pub fn put_para_header(&self, block: BlockNumber, value: &[u8]) -> Result<()> {
        self.put(b'p', block, value)
    }

    pub fn get_storage_changes(&self, block: BlockNumber) -> Option<Vec<u8>> {
        self.get(b'c', block)
    }

    pub fn put_storage_changes(&self, block: BlockNumber, value: &[u8]) -> Result<()> {
        self.put(b'c', block, value)
    }

    pub fn get_genesis(&self, block: BlockNumber) -> Option<Vec<u8>> {
        self.get(b'g', block)
    }

    pub fn put_genesis(&self, block_number: BlockNumber, value: &[u8]) -> Result<()> {
        self.put(b'g', block_number, value)
    }

    pub fn get_metadata(&self) -> Result<Option<Metadata>> {
        let metadata = self
            .0
            .get(b"m-metadata")?
            .map(|encoded| {
                let result: Result<Metadata> = serde_json::from_slice(&encoded).map_err(Into::into);
                result
            })
            .transpose()?;
        Ok(metadata)
    }

    pub fn put_metadata(&self, metadata: &Metadata) -> Result<()> {
        let encoded = serde_json::to_vec(metadata)?;
        self.0.put(b"m-metadata", encoded).map_err(Into::into)
    }
}

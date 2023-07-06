use parity_scale_codec::{Decode, Error};

pub use self::rocksdb::RocksDB;
pub use hashdb::RocksHashDB;

#[cfg(test)]
pub(crate) use rocksdb::with_cache_dir;

mod traits;
mod hashdb;
mod rocksdb;

pub type DecodedDBValue = (Vec<u8>, i32);

fn decode_value(value: Option<Vec<u8>>) -> Result<Option<DecodedDBValue>, Error> {
    match value {
        None => Ok(None),
        Some(value) => {
            let (d, rc): (Vec<u8>, i32) = Decode::decode(&mut &value[..])?;
            Ok(Some((d, rc)))
        }
    }
}

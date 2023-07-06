use parity_scale_codec::{Decode, Error};

pub use self::rocksdb::RocksDB;
pub use hashdb::HashDB;
pub type RocksHashDB<H> = HashDB<H, RocksDB>;
#[cfg(test)]
pub(crate) use cache_dir::with as with_cache_dir;

mod hashdb;
mod rocksdb;
mod redb;

mod serializing;
mod traits;
mod cache_dir;

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

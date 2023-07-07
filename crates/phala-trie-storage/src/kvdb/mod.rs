use parity_scale_codec::{Decode, Error};

pub use self::rocksdb::RocksDB;
pub use self::redb::Redb;

pub use hashdb::HashDB;
pub type HashRocksDB<H> = HashDB<H, RocksDB>;
pub type HashRedb<H> = HashDB<H, Redb>;

#[cfg(test)]
pub(crate) use cache_dir::with as with_cache_dir;

mod hashdb;
mod rocksdb;
mod redb;

mod serializing;
mod cache_dir;

pub mod traits;

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

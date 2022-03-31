use crate::block::Block;
use crate::blockhandle::BlockHandle;
use crate::env::RandomAccess;
use crate::error::{err, Result, StatusCode};
use crate::filter;
use crate::filter_block::FilterBlockReader;
use crate::log::unmask_crc;
use crate::options::{self, CompressionType, Options};
use crate::table_builder;

use crc::crc32::{self, Hasher32};
use integer_encoding::FixedInt;
use snap::read::FrameDecoder;
use std::io::Read;

/// Reads the data for the specified block handle from a file.
fn read_bytes(f: &dyn RandomAccess, location: &BlockHandle) -> Result<Vec<u8>> {
    let mut buf = vec![0; location.size()];
    f.read_at(location.offset(), &mut buf).map(|_| buf)
}

/// Reads a serialized filter block from a file and returns a FilterBlockReader.
pub fn read_filter_block(
    src: &dyn RandomAccess,
    location: &BlockHandle,
    policy: filter::BoxedFilterPolicy,
) -> Result<FilterBlockReader> {
    if location.size() == 0 {
        return err(
            StatusCode::InvalidArgument,
            "no filter block in empty location",
        );
    }
    let buf = read_bytes(src, location)?;
    Ok(FilterBlockReader::new_owned(policy, buf))
}

/// Reads a table block from a random-access source.
/// A table block consists of [bytes..., compress (1B), checksum (4B)]; the handle only refers to
/// the location and length of [bytes...].
pub fn read_table_block(
    opt: Options,
    f: &dyn RandomAccess,
    location: &BlockHandle,
) -> Result<Block> {
    // The block is denoted by offset and length in BlockHandle. A block in an encoded
    // table is followed by 1B compression type and 4B checksum.
    // The checksum refers to the compressed contents.
    let buf = read_bytes(f, location)?;
    let compress = read_bytes(
        f,
        &BlockHandle::new(
            location.offset() + location.size(),
            table_builder::TABLE_BLOCK_COMPRESS_LEN,
        ),
    )?;
    let cksum = read_bytes(
        f,
        &BlockHandle::new(
            location.offset() + location.size() + table_builder::TABLE_BLOCK_COMPRESS_LEN,
            table_builder::TABLE_BLOCK_CKSUM_LEN,
        ),
    )?;

    if !verify_table_block(&buf, compress[0], unmask_crc(u32::decode_fixed(&cksum))) {
        return err(
            StatusCode::Corruption,
            &format!(
                "checksum verification failed for block at {}",
                location.offset()
            ),
        );
    }

    if let Some(ctype) = options::int_to_compressiontype(compress[0] as u32) {
        match ctype {
            CompressionType::CompressionNone => Ok(Block::new(opt, buf)),
            CompressionType::CompressionSnappy => {
                let mut decoded = vec![];
                FrameDecoder::new(buf.as_slice()).read_to_end(&mut decoded)?;
                Ok(Block::new(opt, decoded))
            }
        }
    } else {
        err(StatusCode::InvalidData, "invalid compression type")
    }
}

/// Verify checksum of block
fn verify_table_block(data: &[u8], compression: u8, want: u32) -> bool {
    let mut digest = crc32::Digest::new(crc32::CASTAGNOLI);
    digest.write(data);
    digest.write(&[compression; 1]);
    digest.sum32() == want
}

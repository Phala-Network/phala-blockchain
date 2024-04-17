use crate::pool_operator::DB;
use anyhow::{anyhow, Result};
use log::{debug, error};
use parity_scale_codec::{Decode, Encode};
use phactory_api::blocks::HeadersToSync;
use sp_consensus_grandpa::AuthorityList;
use subxt::ext::sp_runtime::traits::Header;
use std::sync::Arc;

fn encode_u32(val: u32) -> [u8; 4] {
    use byteorder::{ByteOrder, BigEndian};
    let mut buf = [0; 4];
    BigEndian::write_u32(&mut buf, val);
    return buf;
}

pub fn find_valid_num(db: Arc<DB>, start_at: u32, set_id: u64) -> (u32, u64, Option<AuthorityList>) {
    let mut next_number = start_at;
    let mut current_set_id = set_id;
    let mut current_authorities = None;

    for result in db.iterator(rocksdb::IteratorMode::From(&encode_u32(start_at), rocksdb::Direction::Forward)) {
        if let Ok((last_number, next_authorities)) = verify(result, current_set_id, &current_authorities) {
            next_number = last_number + 1;
            current_set_id += 1;
            current_authorities = Some(next_authorities);
        } else {
            break
        }
    }

    (next_number, current_set_id, current_authorities)
}

pub fn verify(result: core::result::Result<(Box<[u8]>, Box<[u8]>), rocksdb::Error>, set_id: u64, authorities: &Option<AuthorityList>) -> Result<(u32, AuthorityList)> {
    let (_, value) = result?;
    let headers = HeadersToSync::decode(&mut &value[..])?;
    let last_header = headers.last().expect("should have header");
    let next_authorities = match phactory_api::blocks::find_scheduled_change(&last_header.header) {
        Some(sc) => sc.next_authorities,
        None => return Err(anyhow!("No scheduled change")),
    };

    let mut prev = None;
    for header in &headers {
        if prev.is_some() && prev.unwrap() != *header.header.parent_hash() {
            return Err(anyhow!(
                "#{} parent is {}, but we have {}", header.header.number, header.header.parent_hash(), prev.unwrap()
            ))
        }
        prev = Some(header.header.hash());
    }
    return Ok((last_header.header.number, next_authorities));

    let authorities = match authorities {
        Some(authorities) => authorities,
        None => return Ok((last_header.header.number, next_authorities)),
    };
    let justifications = last_header.justification.as_ref().expect("last header from proof api should has justification");
    pherry::verify_with_prev_authority_set(
        set_id,
        authorities,
        &last_header.header,
        justifications,
    )?;
    Ok((last_header.header.number, next_authorities))

}

pub fn get_current_point(db: Arc<DB>, num: u32) -> Option<HeadersToSync> {
    let mut iter = db.iterator(rocksdb::IteratorMode::From(&encode_u32(num), rocksdb::Direction::Forward));
    if let Some(Ok((_, value))) = iter.next() {
        match HeadersToSync::decode(&mut &value[..]) {
            Ok(headers) => return Some(headers),
            Err(_) => {},
        };
    }
    None
}

pub fn get_previous_authority_set_change_number(db: Arc<DB>, num:u32) -> Option<u32> {
    let mut iter = db.iterator(rocksdb::IteratorMode::From(&encode_u32(num), rocksdb::Direction::Reverse));
    if let Some(Ok((_, value))) = iter.next() {
        match HeadersToSync::decode(&mut &value[..]) {
            Ok(headers) => return Some(headers.last().unwrap().header.number),
            Err(_) => {},
        };
    }
    None
}

pub fn put_headers_to_db(
    headers_db: Arc<DB>,
    new_headers: HeadersToSync,
    known_chaintip: u32,
) -> Result<u32> {
    let first_new_number = new_headers.first().unwrap().header.number;
    let mut headers = match get_current_point(headers_db.clone(), first_new_number) {
        Some(headers) => {
            let _ = headers_db.delete(encode_u32(std::u32::MAX));
            headers
        },
        None => vec![], 
    };
    for header in &mut headers {
        header.justification = None;
    }
    headers.extend(new_headers);
    let headers = headers;

    let mut last_num: Option<u32> = None;
    for header in &headers {
        if let Some(num) = last_num {
            assert!(num + 1 == header.header.number, "prev {}, current {}, not match", num, header.header.number);
        }
        last_num = Some(header.header.number);
    }

    let from = headers.first().unwrap().header.number;
    let to = headers.last().unwrap().header.number;
    let with_authority_change = phactory_api::blocks::find_scheduled_change(&headers.last().unwrap().header).is_some();

    let key = if with_authority_change {
        encode_u32(to)
    } else if to >= known_chaintip {
        encode_u32(std::u32::MAX)
    } else {
        error!("Should not happen: prove_finality API returns a non-chaintip block without authority set change");
        encode_u32(to)
    };

    let encoded_val = headers.encode();
    let headers_size = encoded_val.len();

    if let Err(err) = headers_db.put(key, encoded_val) {
        anyhow::bail!("Failed to write DB {err}");
    }

    let justification = headers.last().unwrap().justification.as_ref().unwrap();
    debug!(
        "put into headers_db: from {} to {}, count {}, justification size: {}, headers size: {}",
        from,
        to,
        to - from + 1,
        justification.len(),
        headers_size,
    );

    Ok(to)
}

pub fn delete_tail_headers(
    headers_db: Arc<DB>,
) {
    headers_db.delete(encode_u32(std::u32::MAX));
}
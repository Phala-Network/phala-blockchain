use crate::pool_operator::DB;
use anyhow::Result;
use log::{debug, error};
use parity_scale_codec::{Decode, Encode};
use phactory_api::blocks::HeadersToSync;
use std::sync::Arc;

fn encode_u32(val: u32) -> [u8; 4] {
    use byteorder::{ByteOrder, BigEndian};
    let mut buf = [0; 4];
    BigEndian::write_u32(&mut buf, val);
    return buf;
}

pub fn get_headers_with_justification(db: &DB, from: u32) -> Option<HeadersToSync> {
    get_current_point(&db, from)
        .map(|headers| headers
            .into_iter()
            .filter(|header| header.header.number >= from)
            .collect::<Vec<_>>()
        )
}

pub fn get_current_point(db: &DB, num: u32) -> Option<HeadersToSync> {
    let mut iter = db.iterator(rocksdb::IteratorMode::From(&encode_u32(num), rocksdb::Direction::Forward));
    match iter.next() {
        Some(Ok((_, value))) => HeadersToSync::decode(&mut &value[..]).ok(),
        _ => None,
    }
}

pub fn get_previous_authority_set_change_number(db: &DB, num:u32) -> Option<u32> {
    let mut iter = db.iterator(rocksdb::IteratorMode::From(&encode_u32(num), rocksdb::Direction::Reverse));
    match iter.next() {
        Some(Ok((_, value))) => {
            HeadersToSync::decode(&mut &value[..])
                .ok()
                .and_then(|headers| headers.last().map(|h| h.header.number))
        },
        _ => None,
    }
}

pub fn put_headers_to_db(
    headers_db: &DB,
    headers: HeadersToSync,
) -> Result<u32> {
    let mut headers = headers;
    let first_header = &headers.first().expect("at least has one header").header;
    let last_header = &headers.last().expect("at least has one header").header;
    let with_authority_change = phactory_api::blocks::find_scheduled_change(&last_header).is_some();

    if !with_authority_change {
        let mut existing_headers = get_current_point(&headers_db, std::u32::MAX).unwrap_or_default();
        if existing_headers.last().map(|h| h.header.number + 1 >= first_header.number).unwrap_or(false) {
            let mut existing_headers = existing_headers
                .into_iter()
                .filter(|h| h.header.number < first_header.number)
                .collect::<HeadersToSync>();
            for header in &mut existing_headers {
                header.justification = None;
            }
            existing_headers.append(&mut headers);
            headers = existing_headers;
        }
    }

    let mut last_num: Option<u32> = None;
    for header in &headers {
        if let Some(num) = last_num {
            assert!(num + 1 == header.header.number, "prev {}, current {}, not match", num, header.header.number);
        }
        last_num = Some(header.header.number);
    }

    let from = headers.first().unwrap().header.number;
    let to = headers.last().unwrap().header.number;

    let key = if with_authority_change {
        encode_u32(to)
    } else {
        encode_u32(std::u32::MAX)
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
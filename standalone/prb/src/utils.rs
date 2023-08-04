use anyhow::Result;
use futures::future::try_join_all;
use log::{debug, error};
use parity_scale_codec::Decode;
use phaxt::ChainApi;
use subxt::storage::address::Yes;
use subxt::storage::StorageAddress;
use tokio::task::JoinHandle;

pub static CONTENT_TYPE_JSON: &str = "application/json";
pub static CONTENT_TYPE_BIN: &str = "application/octet-stream";

pub async fn join_handles(handles: Vec<JoinHandle<()>>) {
    match try_join_all(handles).await {
        Ok(_) => {
            debug!("Joint task finished.");
        }
        Err(err) => {
            error!("Fatal error: {}", err);
            std::process::exit(100);
        }
    }
}

pub(crate) fn write_storage_address_root_bytes<Address: StorageAddress>(
    addr: &Address,
    out: &mut Vec<u8>,
) {
    out.extend(sp_core::hashing::twox_128(addr.pallet_name().as_bytes()));
    out.extend(sp_core::hashing::twox_128(addr.entry_name().as_bytes()));
}

pub(crate) fn storage_address_bytes<Address: StorageAddress>(
    addr: &Address,
    metadata: &subxt::metadata::Metadata,
) -> Result<Vec<u8>, subxt::error::Error> {
    let mut bytes = Vec::new();
    write_storage_address_root_bytes(addr, &mut bytes);
    addr.append_entry_bytes(metadata, &mut bytes)?;
    Ok(bytes)
}

pub async fn fetch_storage_bytes<'a, Address, T>(
    api: &'a ChainApi,
    address: &'a Address,
) -> Result<Option<T>>
where
    Address: StorageAddress<IsFetchable = Yes> + 'a,
    T: Decode,
{
    let metadata = api.metadata();
    let address = storage_address_bytes(address, &metadata)?;
    let fetched = api
        .storage()
        .at_latest()
        .await?
        .fetch_raw(address.as_slice())
        .await?;
    if let Some(fetched) = fetched {
        let mut fetched = fetched.as_slice();
        Ok(Some(T::decode(&mut fetched)?))
    } else {
        Ok(None)
    }
}

#[macro_export]
macro_rules! with_retry {
    ($f:expr, $c:expr, $s:expr) => {{
        let mut retry_count: u64 = 0;
        loop {
            let r = $f.await;
            match r {
                Err(e) => {
                    warn!("Attempt #{retry_count}({}): {}", stringify!($f), &e);
                    retry_count += 1;
                    if (retry_count - 1) == $c {
                        break Err(e);
                    }
                    tokio::time::sleep(std::time::Duration::from_millis($s)).await;
                }
                _ => break r,
            }
        }
    }};
}

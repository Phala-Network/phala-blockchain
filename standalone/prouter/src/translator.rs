use anyhow::{anyhow, Result};
#[allow(unused_imports)]
use log::{debug, error, info, warn};
use phaxt::ParachainApi;

pub async fn get_pnetwork_ident_by_pk(api: &mut &ParachainApi, pubkey: [u8; 32]) -> Option<String> {
    let phala_network_ident_storage_iter = &mut api
        .storage()
        .phala_registry()
        .phala_network_ident_iter(None)
        .await.ok()?;

    while let Some((_, ident_info)) = phala_network_ident_storage_iter.next().await.ok()? {
        if ident_info.pubkey.0 == pubkey {
            return Some(ident_info.pnetwork_ident);
        }
    }

    None
}
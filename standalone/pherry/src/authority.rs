use anyhow::{anyhow, bail, Context, Result};
use codec::Decode;
use hash_db::{HashDB, EMPTY_PREFIX};
use log::info;
use phactory_api::blocks::{AuthoritySet, AuthoritySetChange};
use phaxt::RelaychainApi;
use sc_consensus_grandpa::GrandpaJustification;
use sp_consensus_grandpa::{AuthorityList, SetId};
use sp_trie::trie_types::TrieDBBuilder;
use sp_trie::{MemoryDB, Trie};

use crate::types::UnsigedBlock;
use crate::{get_header_at, types::Header};

type VersionedAuthorityList = (u8, AuthorityList);

pub struct StorageKeys;

impl StorageKeys {
    pub fn authorities_v0() -> &'static [u8] {
        b":grandpa_authorities"
    }
    pub fn authorities_v1() -> Vec<u8> {
        phaxt::dynamic::storage_key("Grandpa", "Authorities")
    }
    pub fn current_set_id() -> Vec<u8> {
        phaxt::dynamic::storage_key("Grandpa", "CurrentSetId")
    }
}

pub async fn get_authority_with_proof_at(
    api: &RelaychainApi,
    header: &Header,
) -> Result<AuthoritySetChange> {
    let authority_proof = crate::chain_client::read_proofs(
        api,
        Some(header.hash()),
        vec![
            StorageKeys::authorities_v0(),
            &StorageKeys::current_set_id(),
            &StorageKeys::authorities_v1(),
        ],
    )
    .await?;
    let mut mdb = MemoryDB::<sp_core::Blake2Hasher>::default();
    for value in authority_proof.iter() {
        mdb.insert(EMPTY_PREFIX, value);
    }
    let trie = TrieDBBuilder::new(&mdb, &header.state_root).build();

    let id_key = StorageKeys::current_set_id();
    let alt_authorities_key = StorageKeys::authorities_v1();
    let old_authorities_key: &[u8] = StorageKeys::authorities_v0();

    // Read auth list
    let mut auth_list = None;
    for key in [&alt_authorities_key, old_authorities_key] {
        if let Some(value) = trie.get(key).context("Check grandpa authorities failed")? {
            let list: AuthorityList = if key == old_authorities_key {
                VersionedAuthorityList::decode(&mut value.as_slice())
                    .expect("Failed to decode VersionedAuthorityList")
                    .1
            } else {
                AuthorityList::decode(&mut value.as_slice())
                    .expect("Failed to decode AuthorityList")
            };
            auth_list = Some(list);
            break;
        }
    }
    let list = auth_list.ok_or_else(|| anyhow!("No grandpa authorities found"))?;

    // Read auth id
    let Ok(Some(id_value)) = trie.get(&id_key) else {
        bail!("Check grandpa set id failed");
    };
    let id: SetId = Decode::decode(&mut id_value.as_slice()).context("Failed to decode set id")?;
    Ok(AuthoritySetChange {
        authority_set: AuthoritySet { list, id },
        authority_proof,
    })
}

pub async fn verify(api: &RelaychainApi, header: &Header, mut justifications: &[u8]) -> Result<()> {
    if header.number == 0 {
        return Ok(());
    }
    let prev_header = get_header_at(api, Some(header.number - 1)).await?.0;
    let auth_set = get_authority_with_proof_at(api, &prev_header).await?;
    let justification: GrandpaJustification<UnsigedBlock> =
        Decode::decode(&mut justifications).context("Failed to decode justification")?;
    if (
        justification.justification.commit.target_hash,
        justification.justification.commit.target_number,
    ) != (header.hash(), header.number)
    {
        bail!("Invalid commit target in grandpa justification");
    }
    justification
        .verify(auth_set.authority_set.id, &auth_set.authority_set.list)
        .context("Failed to verify justification")?;
    info!(
        "Verified grandpa justification at block {}, auth_id={}",
        header.number, auth_set.authority_set.id
    );
    Ok(())
}

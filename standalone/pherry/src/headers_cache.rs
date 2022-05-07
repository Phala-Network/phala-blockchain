use crate::GRANDPA_ENGINE_ID;
use anyhow::{anyhow, Result};
use codec::{Decode, Encode};
use phactory_api::blocks::AuthoritySetChange;
use phaxt::{
    subxt::{self, rpc::NumberOrHex},
    BlockNumber, Header, ParachainApi, RelaychainApi,
};
use std::io::{Read, Write};

use log::info;

pub use phactory_api::blocks::GenesisBlockInfo;

#[derive(Decode, Encode, Debug)]
pub struct BlockInfo {
    pub header: Header,
    pub justification: Option<Vec<u8>>,
    pub para_header: Option<ParaHeader>,
    pub authority_set_change: Option<AuthoritySetChange>,
}

#[derive(Decode, Encode, Debug)]
pub struct ParaHeader {
    /// Finalized parachain header number
    pub fin_header_num: BlockNumber,
    pub proof: Vec<Vec<u8>>,
}

pub trait DB {
    fn put(&mut self, key: &[u8], value: &[u8]) -> Result<()>;
}

#[derive(Clone)]
pub struct Record<'a> {
    payload: &'a [u8],
}

impl<'a> Record<'a> {
    pub fn new(payload: &'a [u8]) -> Self {
        Self { payload }
    }

    pub fn read<'buf>(mut input: impl Read, buffer: &'a mut Vec<u8>) -> Result<Option<Self>> {
        let mut len_buf = [0u8; 4];

        if input.read(&mut len_buf)? != 4 {
            // EOF
            return Ok(None);
        }

        let length = u32::from_be_bytes(len_buf) as usize;

        if length > buffer.len() {
            buffer.resize(length, 0);
        }

        input.read_exact(&mut buffer[..length])?;

        Ok(Some(Self::new(&buffer[..length])))
    }

    pub fn write(&self, mut writer: impl Write) -> Result<usize> {
        let length = self.payload.len() as u32;
        writer.write_all(&length.to_be_bytes())?;
        writer.write_all(self.payload)?;
        Ok(self.payload.len() + 4)
    }

    pub fn payload(&self) -> &'a [u8] {
        self.payload
    }

    pub fn header(&self) -> Result<Header> {
        Ok(Decode::decode(&mut &self.payload[..])?)
    }
}

/// Read headers from grabbed file
pub fn read_items(
    mut input: impl Read,
    mut f: impl FnMut(Record<'_>) -> Result<bool>,
) -> Result<u32> {
    let mut count = 0_u32;
    let mut buffer = vec![0u8; 1024 * 100];
    loop {
        match Record::read(&mut input, &mut buffer)? {
            None => break,
            Some(record) => {
                count += 1;
                if f(record)? {
                    break;
                }
            }
        }
    }
    Ok(count)
}

/// Import header logs into database.
pub fn import_headers(input: impl Read, to_db: &mut impl DB) -> Result<u32> {
    read_items(input, |record| {
        let header = record.header()?;
        to_db.put(&header.number.to_be_bytes(), record.payload())?;
        Ok(false)
    })
}

/// Dump headers from the chain to a log file.
pub async fn grap_headers_to_file(
    api: &RelaychainApi,
    para_api: &ParachainApi,
    start_at: BlockNumber,
    count: BlockNumber,
    justification_interval: BlockNumber,
    mut output: impl Write,
) -> Result<BlockNumber> {
    grab_headers(
        api,
        para_api,
        start_at,
        count,
        justification_interval,
        |info| {
            if info.justification.is_some() {
                info!("Got justification at {}", info.header.number);
            }
            let encoded = info.encode();
            Record::new(&encoded).write(&mut output)?;
            Ok(())
        },
    )
    .await
}

pub async fn get_set_id(api: &RelaychainApi, block: BlockNumber) -> Result<(u64, bool)> {
    let (block, hash) = crate::get_block_at(&api.client, Some(block)).await?;
    let set_id = api.storage().grandpa().current_set_id(Some(hash)).await?;
    Ok((set_id, block.justifications.is_some()))
}

async fn grab_headers(
    api: &RelaychainApi,
    para_api: &ParachainApi,
    start_at: BlockNumber,
    count: BlockNumber,
    justification_interval: u32,
    mut f: impl FnMut(BlockInfo) -> Result<()>,
) -> Result<BlockNumber> {
    if start_at == 0 {
        anyhow::bail!("start block must be > 0");
    }
    if count == 0 {
        return Ok(0);
    }

    let header_hash = crate::get_header_hash(&api.client, Some(start_at - 1)).await?;
    let mut last_set = api
        .storage()
        .grandpa()
        .current_set_id(Some(header_hash))
        .await?;
    let mut skip_justitication = justification_interval;
    let mut grabbed = 0;

    let para_id = crate::get_paraid(para_api, None).await?;
    info!("para_id: {}", para_id);

    for block_number in start_at.. {
        let header;
        let justifications;
        let hash;
        if skip_justitication == 0 {
            let (block, header_hash) =
                match crate::get_block_at(&api.client, Some(block_number)).await {
                    Ok(x) => x,
                    Err(e) => {
                        if e.to_string().contains("not found") {
                            break;
                        }
                        return Err(e);
                    }
                };
            header = block.block.header;
            justifications = block.justifications;
            hash = header_hash;
        } else {
            let (hdr, hdr_hash) = match crate::get_header_at(&api.client, Some(block_number)).await
            {
                Ok(x) => x,
                Err(e) => {
                    if e.to_string().contains("not found") {
                        break;
                    }
                    return Err(e);
                }
            };
            header = hdr;
            hash = hdr_hash;
            justifications = None;
        };
        let set_id = api.storage().grandpa().current_set_id(Some(hash)).await?;
        let mut justifications = justifications;
        let authority_set_change = if last_set != set_id {
            info!(
                "Authority set changed at block {} from {} to {}",
                header.number, last_set, set_id,
            );
            if justifications.is_none() {
                justifications = Some(
                    api.client
                        .rpc()
                        .block(Some(hash))
                        .await?
                        .ok_or(anyhow!("Failed to fetch block"))?
                        .justifications
                        .ok_or(anyhow!("No justification for block changing set_id"))?,
                );
            }
            Some(crate::get_authority_with_proof_at(&api, hash).await?)
        } else {
            None
        };

        let justification = justifications
            .map(|v| v.into_justification(GRANDPA_ENGINE_ID))
            .flatten();

        skip_justitication = skip_justitication.saturating_sub(1);
        last_set = set_id;

        let para_header = if justification.is_none() {
            None
        } else {
            skip_justitication = justification_interval;
            crate::get_finalized_header_with_paraid(api, para_id, hash).await?
        };

        let para_header = para_header.map(|(header, proof)| ParaHeader {
            fin_header_num: header.number,
            proof: proof,
        });

        f(BlockInfo {
            header,
            justification,
            para_header,
            authority_set_change,
        })?;
        grabbed += 1;
        if count == grabbed {
            break;
        }
    }
    Ok(grabbed)
}

pub async fn fetch_genesis_info(
    api: &RelaychainApi,
    genesis_block_number: BlockNumber,
) -> Result<GenesisBlockInfo> {
    let genesis_block = crate::get_block_at(&api.client, Some(genesis_block_number))
        .await?
        .0
        .block;
    let hash = api
        .client
        .rpc()
        .block_hash(Some(subxt::BlockNumber::from(NumberOrHex::Number(
            genesis_block_number as _,
        ))))
        .await?
        .expect("No genesis block?");
    let set_proof = crate::get_authority_with_proof_at(api, hash).await?;
    Ok(GenesisBlockInfo {
        block_header: genesis_block.header,
        authority_set: set_proof.authority_set,
        proof: set_proof.authority_proof,
    })
}

#[derive(Clone)]
pub(crate) struct Client {
    base_uri: String,
}

impl Client {
    pub fn new(uri: &str) -> Self {
        Self {
            base_uri: uri.to_string(),
        }
    }

    async fn request<T: Decode>(&self, url: &str) -> Result<T> {
        let body = reqwest::get(url).await?.bytes().await?;
        Ok(T::decode(&mut &body[..])?)
    }

    pub async fn get_headers(&self, block_number: BlockNumber) -> Result<Vec<BlockInfo>> {
        let url = format!("{}/headers/{}", self.base_uri, block_number);
        self.request(&url).await
    }

    pub async fn get_genesis(&self, block_number: BlockNumber) -> Result<GenesisBlockInfo> {
        let url = format!("{}/genesis/{}", self.base_uri, block_number);
        self.request(&url).await
    }
}

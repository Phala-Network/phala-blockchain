use crate::types::ConvertTo;
use crate::{types::Header, GRANDPA_ENGINE_ID};
use anyhow::{anyhow, Result};
use codec::{Decode, Encode};
use phaxt::{
    subxt::{self, rpc::types::NumberOrHex},
    BlockNumber, ParachainApi, RelaychainApi,
};
use reqwest::Response;
use std::borrow::Cow;
use std::io::{self, Read, Write};

use futures::stream::Stream;
use log::{debug, error, info, warn};
use tokio::io::{AsyncRead, AsyncReadExt};

pub use phactory_api::blocks::{AuthoritySetChange, BlockHeaderWithChanges, GenesisBlockInfo};

#[derive(Decode, Encode, Debug, Clone)]
pub struct BlockInfo {
    pub header: Header,
    pub justification: Option<Vec<u8>>,
    pub para_header: Option<ParaHeader>,
    pub authority_set_change: Option<AuthoritySetChange>,
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct ParaHeader {
    /// Finalized parachain header number
    pub fin_header_num: BlockNumber,
    pub proof: Vec<Vec<u8>>,
}

#[derive(Clone)]
pub struct Record<'a> {
    payload: Cow<'a, [u8]>,
}

impl<'a> Record<'a> {
    pub fn new(payload: &'a [u8]) -> Self {
        Self {
            payload: payload.into(),
        }
    }

    pub fn read(mut input: impl Read, buffer: &'a mut Vec<u8>) -> Result<Option<Self>> {
        let mut len_buf = [0u8; 4];

        match input.read_exact(&mut len_buf) {
            Ok(_) => {}
            Err(e) => {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    return Ok(None);
                } else {
                    return Err(e.into());
                }
            }
        }

        let length = u32::from_be_bytes(len_buf) as usize;

        if length > buffer.len() {
            buffer.resize(length, 0);
        }

        input.read_exact(&mut buffer[..length])?;

        Ok(Some(Self::new(&buffer[..length])))
    }

    pub async fn async_read<'b>(
        mut input: impl AsyncRead + Unpin,
        buffer: &'b mut Vec<u8>,
    ) -> Result<Option<Record<'b>>, std::io::Error> {
        let mut len_buf = [0u8; 4];

        match input.read_exact(&mut len_buf).await {
            Ok(_) => {}
            Err(e) => {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    return Ok(None);
                } else {
                    return Err(e);
                }
            }
        }

        let length = u32::from_be_bytes(len_buf) as usize;

        if length > buffer.len() {
            buffer.resize(length, 0);
        }

        input.read_exact(&mut buffer[..length]).await?;

        Ok(Some(Record::new(&buffer[..length])))
    }

    pub fn write(&self, mut writer: impl Write) -> Result<usize> {
        let length = self.payload.len() as u32;
        writer.write_all(&length.to_be_bytes())?;
        writer.write_all(&self.payload)?;
        Ok(self.payload.len() + 4)
    }

    pub fn payload(&'a self) -> &'a [u8] {
        &self.payload
    }

    pub fn header(&self) -> Result<Header> {
        Ok(Decode::decode(&mut &self.payload[..])?)
    }

    pub fn to_owned(&self) -> Record<'static> {
        Record {
            payload: self.payload.to_vec().into(),
        }
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

/// Read headers from grabbed file asynchronously.
pub async fn async_read_items(
    mut input: impl AsyncRead + Unpin,
    mut f: impl FnMut(Record<'_>) -> Result<bool> + Unpin,
) -> Result<u32> {
    let mut count = 0_u32;
    let mut buffer = vec![0u8; 1024 * 100];
    loop {
        match Record::async_read(&mut input, &mut buffer).await? {
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

/// Read headers from grabbed file asynchronously.
pub fn read_items_stream(
    mut input: impl AsyncRead + Unpin,
) -> impl Stream<Item = io::Result<Record<'static>>> {
    async_stream::stream! {
        let mut buffer = vec![0u8; 1024 * 100];
        loop {
            match Record::async_read(&mut input, &mut buffer).await {
                Ok(None) => break,
                Ok(Some(record)) => {
                    yield Ok(record.to_owned());
                },
                Err(e) => {
                    yield Err(e);
                    break;
                },
            }
        }
    }
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

/// Dump parachain headers from the chain to a log file.
pub async fn grap_para_headers_to_file(
    api: &ParachainApi,
    start_at: BlockNumber,
    count: BlockNumber,
    mut output: impl Write,
) -> Result<BlockNumber> {
    grab_para_headers(api, start_at, count, |header| {
        if header.number % 1000 == 0 {
            info!("Got para header at {}", header.number);
        }
        let encoded = header.encode();
        Record::new(&encoded).write(&mut output)?;
        Ok(())
    })
    .await
}

/// Dump storage changes from the chain to a log file.
pub async fn grap_storage_changes_to_file(
    api: &ParachainApi,
    start_at: BlockNumber,
    count: BlockNumber,
    batch_size: BlockNumber,
    mut output: impl Write,
) -> Result<BlockNumber> {
    grab_storage_changes(api, start_at, count, batch_size, true, |changes| {
        if changes.block_header.number % 1000 == 0 {
            info!("Got storage changes at {}", changes.block_header.number);
        }
        let encoded = changes.encode();
        Record::new(&encoded).write(&mut output)?;
        Ok(())
    })
    .await
}

pub async fn get_set_id(api: &RelaychainApi, block: BlockNumber) -> Result<(u64, bool)> {
    let (block, hash) = crate::get_block_at(api, Some(block)).await?;
    let set_id = api.current_set_id(Some(hash)).await?;
    Ok((set_id, block.justifications.is_some()))
}

pub async fn grab_headers(
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

    let header_hash = crate::get_header_hash(api, Some(start_at - 1)).await?;
    let mut last_set = api.current_set_id(Some(header_hash)).await?;
    let mut skip_justitication = 0_u32;
    let mut grabbed = 0;

    let para_id = para_api.get_paraid(None).await?;
    info!("para_id: {}", para_id);

    for block_number in start_at.. {
        let header;
        let justifications;
        let hash;
        if skip_justitication == 0 {
            let (block, header_hash) = match crate::get_block_at(api, Some(block_number)).await {
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
            let (hdr, hdr_hash) = match crate::get_header_at(api, Some(block_number)).await {
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
        let set_id = api.current_set_id(Some(hash)).await?;
        let mut justifications = justifications;
        let authority_set_change = if last_set != set_id {
            info!(
                "Authority set changed at block {} from {} to {}",
                header.number, last_set, set_id,
            );
            if justifications.is_none() {
                let just_data = api
                    .rpc()
                    .block(Some(hash))
                    .await?
                    .ok_or_else(|| anyhow!("Failed to fetch block"))?
                    .justifications
                    .ok_or_else(|| anyhow!("No justification for block changing set_id"))?;
                justifications = Some(just_data.convert_to());
            }
            Some(crate::get_authority_with_proof_at(api, hash).await?)
        } else {
            None
        };

        let justification = justifications.and_then(|v| v.into_justification(GRANDPA_ENGINE_ID));

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
            proof,
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

pub async fn grab_para_headers(
    api: &ParachainApi,
    start_at: BlockNumber,
    count: BlockNumber,
    mut f: impl FnMut(Header) -> Result<()>,
) -> Result<BlockNumber> {
    if count == 0 {
        return Ok(0);
    }

    let mut grabbed = 0;

    for block_number in start_at.. {
        let header = match crate::get_header_at(api, Some(block_number)).await {
            Err(e) if e.to_string().contains("not found") => {
                break;
            }
            other => other?.0,
        };
        f(header)?;
        grabbed += 1;
        if count == grabbed {
            break;
        }
    }
    Ok(grabbed)
}

pub async fn grab_storage_changes(
    api: &ParachainApi,
    start_at: BlockNumber,
    count: BlockNumber,
    batch_size: BlockNumber,
    with_root: bool,
    mut f: impl FnMut(BlockHeaderWithChanges) -> Result<()>,
) -> Result<BlockNumber> {
    if count == 0 {
        return Ok(0);
    }

    let to = start_at.saturating_add(count - 1);
    let mut grabbed = 0;

    for from in (start_at..=to).step_by(batch_size as _) {
        let to = to.min(from.saturating_add(batch_size - 1));
        let changes =
            crate::fetch_storage_changes_with_root_or_not(api, None, from, to, with_root).await?;
        for blk in changes {
            f(blk)?;
            grabbed += 1;
        }
    }

    Ok(grabbed)
}

pub async fn fetch_genesis_info(
    api: &RelaychainApi,
    genesis_block_number: BlockNumber,
) -> Result<GenesisBlockInfo> {
    let genesis_block = crate::get_block_at(api, Some(genesis_block_number))
        .await?
        .0
        .block;
    let hash = api
        .rpc()
        .block_hash(Some(subxt::rpc::types::BlockNumber::from(
            NumberOrHex::Number(genesis_block_number as _),
        )))
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
pub struct Client {
    base_uri: String,
    http_client: reqwest::Client,
}

impl Client {
    pub fn new(uri: &str) -> Self {
        Self {
            base_uri: uri.to_string(),
            http_client: reqwest::Client::new(),
        }
    }

    async fn request(&self, url: &str) -> Result<Response> {
        let response = self.http_client.get(url).send().await.map_err(|err| {
            warn!("Failed to fetch data from cache: {err}");
            err
        })?;
        let status = response.status();
        info!("Requested cache from {url} ({})", status.as_u16());
        if !status.is_success() {
            anyhow::bail!(
                "Failed to fetch data from cache with status={}",
                status.as_u16()
            );
        }
        Ok(response)
    }

    async fn request_scale<T: Decode>(&self, url: &str) -> Result<T> {
        let response = self.request(url).await?;
        let body = response.bytes().await.map_err(|err| {
            error!("Failed to read cache response: {err}");
            err
        })?;
        let decoded = T::decode(&mut &body[..]).map_err(|err| {
            error!("Failed to decode cache response: {err}");
            err
        })?;
        Ok(decoded)
    }

    pub async fn ping(&self) -> Result<()> {
        let url = format!("{}/state", self.base_uri);
        let res = self.request(&url).await?;
        if let Ok(t) = res.text().await {
            debug!("Pinging headers cache {}:\n{}", self.base_uri, t);
        } else {
            warn!(
                "Pinging headers cache {} and got unexcepted response.",
                self.base_uri
            );
        }
        Ok(())
    }

    pub async fn get_headers(&self, block_number: BlockNumber) -> Result<Vec<BlockInfo>> {
        let url = format!("{}/headers/{block_number}", self.base_uri);
        self.request_scale(&url).await
    }

    pub async fn get_parachain_headers(
        &self,
        start_number: BlockNumber,
        count: BlockNumber,
    ) -> Result<Vec<Header>> {
        let url = format!("{}/parachain-headers/{start_number}/{count}", self.base_uri);
        self.request_scale(&url).await
    }

    pub async fn get_storage_changes(
        &self,
        start_number: BlockNumber,
        count: BlockNumber,
    ) -> Result<Vec<BlockHeaderWithChanges>> {
        let url = format!("{}/storage-changes/{start_number}/{count}", self.base_uri);
        self.request_scale(&url).await
    }

    pub async fn get_genesis(&self, block_number: BlockNumber) -> Result<GenesisBlockInfo> {
        let url = format!("{}/genesis/{block_number}", self.base_uri);
        self.request_scale(&url).await
    }
}

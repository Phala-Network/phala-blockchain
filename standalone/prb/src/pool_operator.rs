pub use crate::khala;
use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use parity_scale_codec::{Decode, Encode};
use rocksdb::{DBCompactionStyle, DBWithThreadMode, MultiThreaded, Options};
use schnorrkel::keys::Keypair;
use serde::{Deserialize, Serialize};
use sp_core::crypto::{AccountId32, Ss58AddressFormat, Ss58Codec};
use sp_core::sr25519::Pair as Sr25519Pair;
use sp_core::Pair;
use tokio_stream::StreamExt;

static PHALA_SS58_FORMAT_U8: u8 = 30;
lazy_static! {
    static ref PHALA_SS58_FORMAT: Ss58AddressFormat = Ss58AddressFormat::from(PHALA_SS58_FORMAT_U8);
}

static PO_LIST: &str = "po_list";
static PO_BY_PID: &str = "po:pid:";

pub type DB = DBWithThreadMode<MultiThreaded>;

pub fn get_options(max_open_files: Option<i32>) -> Options {
    // Current tuning based off of the total ordered example, flash
    // storage example on
    // https://github.com/facebook/rocksdb/wiki/RocksDB-Tuning-Guide
    let mut opts = Options::default();
    opts.create_if_missing(true);
    opts.set_compaction_style(DBCompactionStyle::Level);
    opts.set_write_buffer_size(67_108_864); // 64mb
    opts.set_max_write_buffer_number(3);
    opts.set_target_file_size_base(67_108_864); // 64mb
    opts.set_level_zero_file_num_compaction_trigger(8);
    opts.set_level_zero_slowdown_writes_trigger(17);
    opts.set_level_zero_stop_writes_trigger(24);
    opts.set_num_levels(4);
    opts.set_max_bytes_for_level_base(536_870_912); // 512mb
    opts.set_max_bytes_for_level_multiplier(8.0);
    opts.set_keep_log_file_num(10);

    if let Some(max_open_files) = max_open_files {
        opts.set_max_open_files(max_open_files);
    }

    opts
}

#[derive(Clone)]
pub struct PoolOperator {
    pub pid: u64,
    pub pair: Sr25519Pair,
    pub proxied: Option<AccountId32>,
}

#[derive(Clone, Encode, Decode)]
pub struct PoolOperatorForEncode {
    pub pid: u64,
    pub pair: [u8; 96],
    pub proxied: Option<AccountId32>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PoolOperatorForSerialize {
    pub pid: u64,
    pub operator_account_id: String,
    pub proxied_account_id: Option<String>,
}

impl From<&PoolOperator> for PoolOperatorForSerialize {
    fn from(v: &PoolOperator) -> Self {
        let operator_account_id: AccountId32 = v.pair.public().into();
        let operator_account_id = operator_account_id.to_ss58check_with_version(*PHALA_SS58_FORMAT);
        let proxied_account_id = v
            .proxied
            .as_ref()
            .map(|a| a.to_ss58check_with_version(*PHALA_SS58_FORMAT));
        Self {
            pid: v.pid,
            operator_account_id,
            proxied_account_id,
        }
    }
}

impl From<&PoolOperator> for PoolOperatorForEncode {
    fn from(v: &PoolOperator) -> Self {
        let pair = v.pair.as_ref().to_bytes();
        Self {
            pid: v.pid,
            pair,
            proxied: v.proxied.clone(),
        }
    }
}

impl From<&PoolOperatorForEncode> for PoolOperator {
    fn from(v: &PoolOperatorForEncode) -> Self {
        let pair = Sr25519Pair::from(Keypair::from_bytes(v.pair.as_ref()).expect("parse key"));
        Self {
            pid: v.pid,
            pair,
            proxied: v.proxied.clone(),
        }
    }
}

pub trait PoolOperatorAccess {
    fn get_pid_list(&self) -> Result<Vec<u64>>;
    fn set_pid_list(&self, new_list: Vec<u64>) -> Result<Vec<u64>>;
    fn get_all_po(&self) -> Result<Vec<PoolOperator>>;
    fn get_po(&self, pid: u64) -> Result<Option<PoolOperator>>;
    fn set_po(&self, pid: u64, po: PoolOperator) -> Result<PoolOperator>;
}

impl PoolOperatorAccess for DB {
    fn get_pid_list(&self) -> Result<Vec<u64>> {
        let key = PO_LIST.to_string();
        let l = self.get(key)?;
        if l.is_none() {
            return Ok(Vec::new());
        }
        let mut l = &l.unwrap()[..];
        let l: Vec<u64> = Vec::decode(&mut l)?;
        Ok(l)
    }
    fn set_pid_list(&self, new_list: Vec<u64>) -> Result<Vec<u64>> {
        let key = PO_LIST.to_string();
        let b = new_list.encode();
        self.put(key, b)?;
        self.get_pid_list()
    }
    fn get_all_po(&self) -> Result<Vec<PoolOperator>> {
        let curr_pid_list = self.get_pid_list()?;
        let mut ret = Vec::new();
        for id in curr_pid_list {
            let i = self
                .get_po(id)?
                .ok_or(anyhow!(format!("po record #{id} not found!")))?;
            ret.push(i);
        }
        Ok(ret)
    }
    fn get_po(&self, pid: u64) -> Result<Option<PoolOperator>> {
        let key = format!("{PO_BY_PID}:{pid}");
        let b = self.get(key)?;
        if b.is_none() {
            return Ok(None);
        }
        let mut b = &b.unwrap()[..];
        let po = PoolOperatorForEncode::decode(&mut b)?;
        Ok(Some((&po).into()))
    }
    fn set_po(&self, pid: u64, po: PoolOperator) -> Result<PoolOperator> {
        let mut pl = self.get_pid_list()?;
        pl.retain(|&i| i != pid);
        pl.push(pid);
        let key = format!("{PO_BY_PID}:{pid}");
        let b = PoolOperatorForEncode::from(&po);
        let b = b.encode();
        self.put(key, b)?;
        let r = self.get_po(pid)?;
        let _ = self.set_pid_list(pl)?;
        Ok(r.unwrap())
    }
}
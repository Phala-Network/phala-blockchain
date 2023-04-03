use rocksdb::{DBCommon, DBCompactionStyle, DBWithThreadMode, MultiThreaded, Options, WriteBatch};
use std::path::Path;
use std::sync::Arc;

type DB = DBWithThreadMode<MultiThreaded>;

fn get_options(max_open_files: Option<i32>) -> Options {
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

    if let Some(max_open_files) = max_open_files {
        opts.set_max_open_files(max_open_files);
    }

    opts
}

pub struct TxManager {
    pub db: Arc<DB>,
}

impl TxManager {
    pub fn new(path_base: &str) -> anyhow::Result<Self> {
        let opts = get_options(None);
        let path = Path::new(path_base).join("po");
        let mut db = DB::open(&opts, path)?;
        Ok(TxManager { db: Arc::new(db) })
    }
}

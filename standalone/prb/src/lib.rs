// #![feature(async_closure)]
// #![feature(addr_parse_ascii)]

pub mod api;
pub mod cli;
pub mod configurator;
pub mod datasource;
pub mod db;
pub mod lifecycle;
pub mod pruntime;
pub mod tx;
pub mod utils;
pub mod wm;
pub mod worker;

#[subxt::subxt(runtime_metadata_url = "wss://khala.api.onfinality.io:443/public-ws")]
pub mod khala {}

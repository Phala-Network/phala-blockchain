#![feature(async_closure)]
#![feature(addr_parse_ascii)]
#![feature(layout_for_ptr)]

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

#[subxt::subxt(
    runtime_metadata_url = "wss://khala.api.onfinality.io:443/public-ws",
    substitute_type(
        type = "phala_mq::types::SignedMessage",
        with = "::phala_types::messaging::SignedMessage"
    ),
    substitute_type(type = "sp_core::sr25519::Public", with = "::sp_core::sr25519::Public"),
    substitute_type(
        type = "phala_pallets::compute::computation::pallet::SessionInfo",
        with = "::phala_pallets::compute::computation::pallet::SessionInfo"
    )
)]
pub mod khala {}

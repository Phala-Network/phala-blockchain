use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TcbInfo {
    pub id: String,
    pub version: u8,
    pub issue_date: String,
    pub next_update: String,
    pub fmspc: String,
    pub pce_id: String,
    pub tcb_type: u32,
    pub tcb_evaluation_data_number: u32,
    pub tcb_levels: Vec<TcbLevel>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TcbLevel {
    pub tcb: Tcb,
    pub tcb_date: String,
    pub tcb_status: String,
    #[serde(rename = "advisoryIDs", default)]
    pub advisory_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tcb {
    #[serde(rename = "sgxtcbcomponents")]
    pub components: Vec<TcbComponents>,
    #[serde(rename = "pcesvn")]
    pub pce_svn: u16,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TcbComponents {
    pub svn: u8,
}

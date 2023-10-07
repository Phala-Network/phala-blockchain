#![no_std]
extern crate alloc;

use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    Ping,
    Callback { call_data: Vec<u8> },
}

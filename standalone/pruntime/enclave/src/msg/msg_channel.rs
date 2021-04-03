
use crate::std::vec::Vec;
use crate::String;
use crate::str::FromStr;
use serde::{Deserialize, Serialize};
use core::fmt::Display;
use phala_types::message::{Message,SignedMessage};
use phala_types::{WorkerMessage};
extern crate runtime as chain;


#[derive(Debug,Default,Serialize, Deserialize)]
pub struct MsgChannel<M:Message> {
    pub sequence: u64,
    pub queue: Vec<SignedMessage<M>>,
}

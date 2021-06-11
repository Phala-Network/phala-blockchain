use alloc::vec::Vec;
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

pub type Path = Vec<u8>;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, Serialize, Deserialize)]
pub enum Origin {
    /// Anonymous message
    Anonymous,
    /// Runtime pallets
    Runtime,
    /// A confidential contract running in some pRuntime.
    Contract(Vec<u8>),
    /// An chain user
    Account(Vec<u8>),
    /// A remote location (parachain, etc.)
    Multilocaiton(Vec<u8>),
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct Message {
    pub sender: Origin,
    pub destination: Path,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct SignedMessage {
    pub message: Message,
    pub sequence: u64,
    pub signature: Vec<u8>,
}

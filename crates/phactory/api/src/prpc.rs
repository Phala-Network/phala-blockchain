pub use crate::proto_generated::*;
use alloc::vec::Vec;
use phala_types::messaging::{MessageOrigin, ChainedMessage, AppointedMessage, Signature as MessageSignature};
pub use prpc::{client, server, Message};
pub type EgressMessages = Vec<(MessageOrigin, Vec<(ChainedMessage, MessageSignature)>)>;
pub type AppointedEgressMessages = Vec<(MessageOrigin, Vec<(AppointedMessage, MessageSignature)>)>;

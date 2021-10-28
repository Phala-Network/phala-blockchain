pub use crate::proto_generated::*;
use alloc::vec::Vec;
use phala_types::messaging::{MessageOrigin, SignedMessageV2};
pub use prpc::{client, server, Message};
pub type EgressMessages = Vec<(MessageOrigin, Vec<SignedMessageV2>)>;

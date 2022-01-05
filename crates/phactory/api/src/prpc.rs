pub use crate::proto_generated::*;
use alloc::vec::Vec;
use phala_types::messaging::{
    AppointedMessage, ChainedMessage, MessageOrigin, Signature as MessageSignature,
};
pub use prpc::{client, server, Message};
pub type EgressMessages = Vec<(
    MessageOrigin,
    Vec<(ChainedMessage, MessageSignature)>,
    Vec<(AppointedMessage, MessageSignature)>,
)>;

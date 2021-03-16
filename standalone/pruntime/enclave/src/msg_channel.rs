use crate::std::vec::Vec;
use parity_scale_codec::Encode;
use phala_types::{SignedWorkerMessage, WorkerMessage, WorkerMessagePayload};
use sp_core::ecdsa;
use sp_core::Pair;

/// An one-way async message channel
pub struct MsgChannel {
    pub sequence: u64,
    pub queue: Vec<SignedWorkerMessage>,
}

impl MsgChannel {
    /// Push an item to the message queue
    pub fn push(&mut self, item: WorkerMessagePayload, pair: &ecdsa::Pair) {
        let data = WorkerMessage {
            payload: item,
            sequence: self.sequence,
        };
        // Encapsulate with signature
        let sig = pair.sign(&Encode::encode(&data));
        let signed = SignedWorkerMessage {
            data,
            signature: sig.0.to_vec(),
        };
        // Update the queue
        self.queue.push(signed);
        self.sequence += 1;
    }
    /// Called on received messages and drop them
    pub fn received(&mut self, seq: u64) {
        if seq > self.sequence {
            // Something bad happened
            println!(
                "MsgChannel::received(): error - received seq {} larger than max seq {}",
                seq, self.sequence
            );
            return;
        }
        self.queue.retain(|item| item.data.sequence > seq);
    }
}

impl Default for MsgChannel {
    fn default() -> Self {
        Self {
            sequence: 0u64,
            queue: Vec::new(),
        }
    }
}

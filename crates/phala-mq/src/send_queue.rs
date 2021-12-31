use crate::{
    AppointedMessage, ChainedMessage, Message, MessageOrigin, MessageSigner, MqHash, Mutex,
    SenderId, Signature, SigningMessage, Appointment,
};
use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
struct Channel {
    next_sequence: u64,
    last_hash: MqHash,
    messages: Vec<(ChainedMessage, Signature)>,

    next_appointment_sequence: u64,
    appointed_seqs: Vec<u64>,
    appointing: u8,
    appointed_egress_messages: Vec<(AppointedMessage, Signature)>,
    /// Number of pending appointments.
    dummy: bool,
}

#[derive(Clone, Default)]
pub struct MessageSendQueue {
    inner: Arc<Mutex<BTreeMap<SenderId, Channel>>>,
}

pub struct SequenceInfo {
    pub next_sequence: u64,
    pub next_ap_sequence: u64,
    pub ap_sequences: Vec<u64>,
}

impl Serialize for MessageSendQueue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let inner = self.inner.lock();
        let inner = &*inner;
        inner.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for MessageSendQueue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let inner = BTreeMap::<SenderId, Channel>::deserialize(deserializer)?;
        Ok(MessageSendQueue {
            inner: Arc::new(Mutex::new(inner)),
        })
    }
}

impl MessageSendQueue {
    pub fn new() -> Self {
        MessageSendQueue {
            inner: Default::default(),
        }
    }

    pub fn channel<Si: MessageSigner>(&self, sender: SenderId, signer: Si) -> MessageChannel<Si> {
        MessageChannel::new(self.clone(), sender, signer)
    }

    pub fn enqueue_message(
        &self,
        sender: SenderId,
        constructor: impl FnOnce(u64, MqHash) -> (ChainedMessage, Signature),
    ) {
        let mut inner = self.inner.lock();
        let entry = inner.entry(sender).or_default();
        let (message, signature) = constructor(entry.next_sequence, entry.last_hash);
        let hash = message.hash;
        if !entry.dummy {
            if log::log_enabled!(target: "mq", log::Level::Debug) {
                log::debug!(target: "mq",
                    "Sending message, from={}, to={:?}, seq={}, payload_hash={}",
                    message.message.sender,
                    message.message.destination,
                    entry.next_sequence,
                    hex::encode(sp_core::blake2_256(&message.message.payload)),
                );
            } else {
                log::info!(target: "mq",
                        "Sending message, from={}, to={:?}, seq={}, hash={:?}",
                    message.message.sender,
                    message.message.destination,
                    entry.next_sequence,
                    &hash,
                );
            }
            entry.messages.push((message, signature));
        }
        entry.next_sequence += 1;
        entry.last_hash = hash;
    }

    pub fn enqueue_appointed_message(
        &self,
        sender: SenderId,
        constructor: impl FnOnce() -> (AppointedMessage, Signature),
    ) {
        let mut inner = self.inner.lock();
        let entry = inner.entry(sender).or_default();
        let (message, signature) = constructor();
        if !entry.dummy {
            log::info!(target: "mq",
                "Sending appointed message, from={}, to={:?}, seq={}",
                message.message.sender,
                message.message.destination,
                entry.next_sequence,
            );
            entry.appointed_egress_messages.push((message, signature));
        }
    }

    pub fn appoint_next(&self, sender: SenderId) -> Option<u64> {
        let mut inner = self.inner.lock();
        let entry = inner.entry(sender).or_default();
        // Max number of appointments per sender.
        const MAX_APPOINTMENTS: usize = 8;
        if entry.appointed_seqs.len() >= MAX_APPOINTMENTS {
            return None;
        }
        let seq = entry.next_appointment_sequence;
        entry.next_appointment_sequence += 1;
        entry.appointed_seqs.push(seq);
        entry.appointing += 1;
        Some(seq)
    }

    pub fn set_dummy_mode(&self, sender: SenderId, dummy: bool) {
        let mut inner = self.inner.lock();
        let entry = inner.entry(sender).or_default();
        entry.dummy = dummy;
    }

    pub fn last_hash(&self, sender: &SenderId) -> MqHash {
        let inner = self.inner.lock();
        inner
            .get(sender)
            .map_or(Default::default(), |ch| ch.last_hash)
    }

    pub fn all_messages(&self) -> Vec<(ChainedMessage, Signature)> {
        let inner = self.inner.lock();
        inner
            .iter()
            .flat_map(|(_k, v)| v.messages.iter().cloned())
            .collect()
    }

    pub fn all_messages_grouped(
        &self,
    ) -> BTreeMap<MessageOrigin, Vec<(ChainedMessage, Signature)>> {
        let inner = self.inner.lock();
        inner
            .iter()
            .map(|(k, v)| (k.clone(), v.messages.clone()))
            .collect()
    }

    pub fn messages(&self, sender: &SenderId) -> Vec<(ChainedMessage, Signature)> {
        let inner = self.inner.lock();
        inner
            .get(sender)
            .map(|x| x.messages.clone())
            .unwrap_or_default()
    }

    pub fn count_messages(&self) -> usize {
        self.inner
            .lock()
            .iter()
            .map(|(_k, v)| v.messages.len())
            .sum()
    }

    /// Purge the messages which are aready accepted on chain.
    pub fn purge(&self, next_sequence_for: impl Fn(&SenderId) -> SequenceInfo) {
        let mut inner = self.inner.lock();
        for (k, v) in inner.iter_mut() {
            let info = next_sequence_for(k);
            v.messages
                .retain(|msg| msg.0.sequence >= info.next_sequence);
            v.appointed_egress_messages.retain(|msg| {
                msg.0.sequence >= info.next_ap_sequence
                    || info.ap_sequences.contains(&msg.0.sequence)
            });
            v.appointed_seqs.retain(|&seq| {
                seq >= info.next_ap_sequence
                    || info.ap_sequences.contains(&seq)
            });
        }
    }
}

pub use msg_channel::*;
mod msg_channel {
    use super::*;
    use crate::{types::Path, MessageSigner, SenderId};

    #[derive(Clone, Serialize, Deserialize)]
    pub struct MessageChannel<Si> {
        #[serde(skip)]
        #[serde(default = "crate::checkpoint_helper::global_send_mq")]
        queue: MessageSendQueue,
        sender: SenderId,
        signer: Si,
    }

    impl<Si> MessageChannel<Si> {
        pub fn new(queue: MessageSendQueue, sender: SenderId, signer: Si) -> Self {
            MessageChannel {
                queue,
                sender,
                signer,
            }
        }
    }

    impl<Si: MessageSigner + Clone> MessageChannel<Si> {
        fn prepare_with_data(
            &self,
            payload: alloc::vec::Vec<u8>,
            to: impl Into<Path>,
            hash: MqHash,
        ) -> SigningMessage<Si> {
            let sender = self.sender.clone();
            let signer = self.signer.clone();
            let message = Message {
                sender,
                destination: to.into().into(),
                payload,
            };
            SigningMessage {
                message,
                signer,
                hash,
            }
        }
    }

    impl<T: MessageSigner + Clone> crate::traits::MessageChannel for MessageChannel<T> {
        fn push_data(&self, payload: Vec<u8>, to: impl Into<Path>, hash: MqHash) {
            let signing = self.prepare_with_data(payload, to, hash);
            self.queue
                .enqueue_message(self.sender.clone(), move |sequence, parent_hash| {
                    signing.sign_chained(sequence, parent_hash)
                })
        }

        /// Set the channel to dummy mode which increasing the sequence but dropping the message.
        fn set_dummy(&self, dummy: bool) {
            self.queue.set_dummy_mode(self.sender.clone(), dummy);
        }
    }

    impl<T: MessageSigner + Clone> crate::traits::MessagePrepareChannel for MessageChannel<T> {
        type Signer = T;

        fn prepare_with_data(
            &self,
            payload: alloc::vec::Vec<u8>,
            to: impl Into<Path>,
        ) -> SigningMessage<Self::Signer> {
            self.prepare_with_data(payload, to, Default::default())
        }
    }
}

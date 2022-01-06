use crate::{
    AppointedMessage, Appointment, BindTopic, ChainedMessage, Message, MessageOrigin,
    MessageSigner, MqHash, Mutex, SenderId, Signature,
};
use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};
use parity_scale_codec::Encode as _;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum Error {
    ChannelNotFound,
    QuotaExceeded,
    InvalidSequence,
    MqDisabled,
}

type MqResult<T> = Result<T, Error>;

#[derive(Serialize, Deserialize)]
struct Channel {
    signer: MessageSigner,

    next_sequence: u64,
    last_hash: MqHash,
    messages: Vec<(ChainedMessage, Signature)>,

    next_appointment_sequence: u64,
    appointed_seqs: Vec<u64>,
    appointing: u8,
    appointed_messages: Vec<(AppointedMessage, Signature)>,

    dummy: bool,
}

#[derive(Clone, Default)]
pub struct MessageSendQueue {
    inner: Arc<Mutex<MessageSendQueueInner>>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct MessageSendQueueInner {
    channels: BTreeMap<SenderId, Channel>,
    enabled: bool,
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
        let inner = MessageSendQueueInner::deserialize(deserializer)?;
        Ok(MessageSendQueue {
            inner: Arc::new(Mutex::new(inner)),
        })
    }
}

impl MessageSendQueue {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn enable(&self) {
        let mut inner = self.inner.lock();
        inner.enabled = true;
    }

    pub fn disable(&self) {
        let mut inner = self.inner.lock();
        inner.enabled = false;
    }

    /// Get message channel given sender id.
    ///
    /// If channel does not exist, create one.
    pub fn channel(&self, sender: SenderId, signer: MessageSigner) -> MessageChannel {
        let mut inner = self.inner.lock();
        let _entry = inner
            .channels
            .entry(sender.clone())
            .or_insert_with(move || Channel {
                signer,
                next_sequence: 0,
                last_hash: MqHash::default(),
                messages: Vec::new(),
                next_appointment_sequence: 0,
                appointed_seqs: Vec::new(),
                appointing: 0,
                appointed_messages: Vec::new(),
                dummy: false,
            });

        MessageChannel::new(sender, self.clone())
    }

    /// Enqueue a hash-chained message.
    pub fn enqueue_message(&self, message: Message, hash: MqHash) -> MqResult<()> {
        let mut inner = self.inner.lock();
        if !inner.enabled {
            return Err(Error::MqDisabled);
        }
        let entry = inner
            .channels
            .get_mut(&message.sender)
            .ok_or(Error::ChannelNotFound)?;

        let (message, signature) = {
            let parent_hash = entry.last_hash;
            let hash = crate::hash(&(entry.next_sequence, &parent_hash, hash).encode());
            let message = ChainedMessage::new(message, entry.next_sequence, hash, parent_hash);
            let signature = entry.signer.sign(&message.encode());
            (message, signature)
        };

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
        Ok(())
    }

    /// Enqueue an appointed message.
    ///
    /// The sequence must be returned from `fn make_appointment` and has not been resolved yet.
    pub fn enqueue_appointed_message(&self, message: Message, sequence: u64) -> MqResult<()> {
        let mut inner = self.inner.lock();
        let entry = inner
            .channels
            .get_mut(&message.sender)
            .ok_or(Error::ChannelNotFound)?;
        if !entry.dummy {
            if !entry.appointed_seqs.contains(&sequence) {
                log::warn!(target: "mq",
                    "Trying to send an appointed message with invalid sequence, from={}, to={:?}, seq={}",
                    message.sender,
                    message.destination,
                    sequence,
                );
                return Err(Error::InvalidSequence);
            }
            if !entry
                .appointed_messages
                .iter()
                .any(|(m, _)| m.sequence == sequence)
            {
                log::info!(target: "mq",
                    "Sending appointed message, from={}, to={:?}, seq={}",
                    message.sender,
                    message.destination,
                    sequence,
                );
                let message = AppointedMessage::new(message, sequence);
                let signature = entry.signer.sign(&message.encode());
                entry.appointed_messages.push((message, signature));
            } else {
                log::info!(target: "mq",
                    "Appointed message already exists in queue, from={}, to={:?}, seq={}",
                    message.sender,
                    message.destination,
                    sequence,
                );
            }
        }
        Ok(())
    }

    /// Make an appointment for a later message.
    ///
    /// Returns the sequence number of the appointment. If the channel is not found or max number
    /// of appointments is reached, returns `None`.
    pub fn make_appointment(&self, sender: &SenderId) -> MqResult<u64> {
        let mut inner = self.inner.lock();
        if !inner.enabled {
            return Err(Error::MqDisabled);
        }
        let entry = inner
            .channels
            .get_mut(sender)
            .ok_or(Error::ChannelNotFound)?;
        // Max number of appointments per sender.
        const MAX_APPOINTMENTS: usize = 8;
        if entry.appointed_seqs.len() >= MAX_APPOINTMENTS {
            return Err(Error::QuotaExceeded);
        }
        let seq = entry.next_appointment_sequence;
        entry.next_appointment_sequence += 1;
        entry.appointed_seqs.push(seq);
        entry.appointing += 1;
        Ok(seq)
    }

    /// Convert the pending appointments to a mq message.
    ///
    /// This should be called at the end of each block dispatch.
    pub fn commit_appointments(&self) {
        let mut messages = vec![];
        {
            let mut inner = self.inner.lock();
            if !inner.enabled {
                return;
            }
            for (sender, channel) in inner.channels.iter_mut() {
                if channel.appointing == 0 {
                    continue;
                }
                let payload = Appointment::new(channel.appointing).encode();
                channel.appointing = 0;
                let message = Message {
                    sender: sender.clone(),
                    destination: Appointment::topic().into(),
                    payload,
                };
                let data_to_sign = message.encode();
                let hash = crate::hash(&data_to_sign);
                messages.push((message, hash));
            }
        }
        for (message, hash) in messages {
            self.enqueue_message(message, hash)
                .expect("BUG: Failed to enqueue message");
        }
    }

    pub fn set_dummy_mode(&self, sender: SenderId, dummy: bool) -> MqResult<()> {
        let mut inner = self.inner.lock();
        let entry = inner
            .channels
            .get_mut(&sender)
            .ok_or(Error::ChannelNotFound)?;
        entry.dummy = dummy;
        Ok(())
    }

    pub fn last_hash(&self, sender: &SenderId) -> MqHash {
        let inner = self.inner.lock();
        inner
            .channels
            .get(sender)
            .map_or(Default::default(), |ch| ch.last_hash)
    }

    pub fn all_messages(&self) -> Vec<(ChainedMessage, Signature)> {
        let inner = self.inner.lock();
        inner
            .channels
            .iter()
            .flat_map(|(_k, v)| v.messages.iter().cloned())
            .collect()
    }

    pub fn all_messages_grouped(
        &self,
    ) -> BTreeMap<
        MessageOrigin,
        (
            Vec<(ChainedMessage, Signature)>,
            Vec<(AppointedMessage, Signature)>,
        ),
    > {
        let inner = self.inner.lock();
        inner
            .channels
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    (v.messages.clone(), v.appointed_messages.clone()),
                )
            })
            .collect()
    }

    pub fn messages(&self, sender: &SenderId) -> Vec<(ChainedMessage, Signature)> {
        let inner = self.inner.lock();
        inner
            .channels
            .get(sender)
            .map(|x| x.messages.clone())
            .unwrap_or_default()
    }

    pub fn count_messages(&self) -> usize {
        self.inner
            .lock()
            .channels
            .iter()
            .map(|(_k, v)| v.messages.len() + v.appointed_messages.len())
            .sum()
    }

    /// Purge the messages which are aready accepted on chain.
    pub fn purge(&self, next_sequence_for: impl Fn(&SenderId) -> SequenceInfo) {
        let mut inner = self.inner.lock();
        for (k, v) in inner.channels.iter_mut() {
            let info = next_sequence_for(k);
            v.messages
                .retain(|msg| msg.0.sequence >= info.next_sequence);
            v.appointed_messages.retain(|msg| {
                msg.0.sequence >= info.next_ap_sequence
                    || info.ap_sequences.contains(&msg.0.sequence)
            });
            v.appointed_seqs
                .retain(|&seq| seq >= info.next_ap_sequence || info.ap_sequences.contains(&seq));
        }
    }
}

pub use msg_channel::*;
mod msg_channel {
    use super::*;
    use crate::{types::Path, SenderId};

    #[derive(Clone, Serialize, Deserialize)]
    pub struct MessageChannel {
        #[serde(skip)]
        #[serde(default = "crate::checkpoint_helper::global_send_mq")]
        queue: MessageSendQueue,
        sender: SenderId,
    }

    impl MessageChannel {
        pub fn new(sender: SenderId, queue: MessageSendQueue) -> Self {
            Self { sender, queue }
        }
    }

    impl crate::traits::MessageChannel for MessageChannel {
        fn push_data(&self, payload: Vec<u8>, to: impl Into<Path>, hash: MqHash) {
            let message = Message {
                sender: self.sender.clone(),
                destination: to.into().into(),
                payload,
            };
            self.queue
                .enqueue_message(message, hash)
                .expect("BUG: Since the channel exists, this should nerver fail");
        }

        /// Set the channel to dummy mode which increasing the sequence but dropping the message.
        fn set_dummy(&self, dummy: bool) {
            self.queue
                .set_dummy_mode(self.sender.clone(), dummy)
                .expect("BUG: Since the channel exists, this should nerver fail");
        }

        fn make_appointment(&self) -> Option<u64> {
            self.queue.make_appointment(&self.sender).ok()
        }
    }

    impl crate::traits::MessagePreparing for MessageChannel {
        fn prepare_with_data(&self, payload: alloc::vec::Vec<u8>, to: impl Into<Path>) -> Message {
            let sender = self.sender.clone();
            Message {
                sender,
                destination: to.into().into(),
                payload,
            }
        }
    }
}

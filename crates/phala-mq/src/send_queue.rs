use crate::{
    AppointedMessage, Appointment, ChainedMessage, Message, MessageOrigin, MessageSigner, MqHash,
    Mutex, SenderId, Signature, SigningMessage,
};
use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};
use parity_scale_codec::Encode as _;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum Error {
    ChannelNotFound,
    QuotaExceeded,
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
        Default::default()
    }

    /// Get message channel given sender id.
    ///
    /// If channel does not exist, create one.
    pub fn channel(&self, sender: SenderId, signer: MessageSigner) -> MessageChannel {
        let mut inner = self.inner.lock();
        let _entry = inner.entry(sender.clone()).or_insert_with(move || Channel {
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

    pub fn enqueue_message(&self, sender: SenderId, message: SigningMessage) -> MqResult<()> {
        let mut inner = self.inner.lock();
        let entry = inner.get_mut(&sender).ok_or(Error::ChannelNotFound)?;
        let (message, signature) =
            message.sign_chained(entry.next_sequence, entry.last_hash, &entry.signer);
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

    pub fn enqueue_appointed_message(
        &self,
        sender: SenderId,
        message: Message,
        sequence: u64,
    ) -> MqResult<()> {
        let mut inner = self.inner.lock();
        let entry = inner.get_mut(&sender).ok_or(Error::ChannelNotFound)?;
        if !entry.dummy {
            log::info!(target: "mq",
                "Sending appointed message, from={}, to={:?}, seq={}",
                message.sender,
                message.destination,
                sequence,
            );
            let message = AppointedMessage::new(message, sequence);
            let signature = entry.signer.sign(&message.encode());
            entry.appointed_messages.push((message, signature));
        }
        Ok(())
    }

    pub fn make_appointment(&self, sender: &SenderId) -> MqResult<u64> {
        let mut inner = self.inner.lock();
        let entry = inner.get_mut(sender).ok_or(Error::ChannelNotFound)?;
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

    pub fn commit_appointments(&self) {
        let mut inner = self.inner.lock();
        for channel in inner.values_mut() {
            Appointment::new(channel.appointing);
            channel.appointing = 0;
            todo!("TODO.kevin.must.")
        }
    }

    pub fn set_dummy_mode(&self, sender: SenderId, dummy: bool) -> MqResult<()> {
        let mut inner = self.inner.lock();
        let entry = inner.get_mut(&sender).ok_or(Error::ChannelNotFound)?;
        entry.dummy = dummy;
        Ok(())
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
    ) -> BTreeMap<
        MessageOrigin,
        (
            Vec<(ChainedMessage, Signature)>,
            Vec<(AppointedMessage, Signature)>,
        ),
    > {
        let inner = self.inner.lock();
        inner
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
            let message = SigningMessage {
                message: Message {
                    sender: self.sender.clone(),
                    destination: to.into().into(),
                    payload,
                },
                hash,
            };
            self.queue
                .enqueue_message(self.sender.clone(), message)
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

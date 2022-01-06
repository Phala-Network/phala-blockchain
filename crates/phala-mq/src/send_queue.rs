use crate::{
    AppointedMessage, Appointment, BindTopic, ChainedMessage, Error, Message, MessageOrigin,
    MessageSigner, MqHash, MqResult, Mutex, SenderId, Signature,
};
use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};
use parity_scale_codec::Encode as _;
use serde::{Deserialize, Serialize};

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

#[derive(Default)]
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
        if message.destination.is_sys_topic() {
            return Err(Error::Forbidden);
        }
        self.do_enqueue_message(message, hash)
    }

    fn do_enqueue_message(&self, message: Message, hash: MqHash) -> MqResult<()> {
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
    /// If there is already an message with the same sequence, the new one will be dropped.
    pub fn enqueue_appointed_message(&self, message: Message, sequence: u64) -> MqResult<()> {
        if message.destination.is_sys_topic() {
            return Err(Error::Forbidden);
        }
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
            self.do_enqueue_message(message, hash)
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
            .filter_map(|(k, v)| {
                if v.messages.len() == 0 && v.appointed_messages.len() == 0 {
                    return None;
                }
                Some((
                    k.clone(),
                    (v.messages.clone(), v.appointed_messages.clone()),
                ))
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
        pub(super) queue: MessageSendQueue,
        pub(super) sender: SenderId,
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
            match self.queue.enqueue_message(message, hash) {
                Ok(_) => (),
                Err(err) => {
                    log::error!("BUG: Failed to enqueue message: {:?}", err);
                }
            }
        }

        /// Set the channel to dummy mode which increasing the sequence but dropping the message.
        fn set_dummy(&self, dummy: bool) {
            self.queue
                .set_dummy_mode(self.sender.clone(), dummy)
                .expect("BUG: Since the channel exists, this should nerver fail");
        }

        fn make_appointment(&self) -> MqResult<u64> {
            self.queue.make_appointment(&self.sender)
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        traits::{MessageChannel as _, MessagePreparing as _},
        MessageChannel, MessageOrigin, MessageSendQueue, MessageSigner,
    };

    impl MessageChannel {
        #[cfg(test)]
        fn inspect<T>(&self, block: impl FnOnce(&Channel) -> T) -> Option<T> {
            self.queue
                .inner
                .lock()
                .channels
                .get(&self.sender)
                .map(block)
        }
    }

    struct TestQueue {
        queue: MessageSendQueue,
        channel: MessageChannel,
    }

    fn new_queue() -> TestQueue {
        let queue = MessageSendQueue::new();
        queue.enable();
        let channel = queue.channel(
            MessageOrigin::Gatekeeper,
            MessageSigner::Test(b"key".to_vec()),
        );
        TestQueue { queue, channel }
    }

    #[test]
    fn should_reject_duplicates_messages() {
        let TestQueue { queue, channel } = new_queue();

        let sequence = channel.make_appointment().unwrap();

        let message = channel.prepare_with_data(b"foo".to_vec(), b"bar".to_vec());
        assert!(queue.enqueue_appointed_message(message, sequence).is_ok());

        let message = channel.prepare_with_data(b"foo".to_vec(), b"bar".to_vec());
        assert!(queue.enqueue_appointed_message(message, sequence).is_ok());

        assert_eq!(channel.inspect(|c| c.appointed_messages.len()), Some(1));
    }

    #[test]
    fn should_reject_future_messages() {
        let TestQueue { queue, channel } = new_queue();

        let sequence = channel.make_appointment().unwrap();
        let message = channel.prepare_with_data(b"foo".to_vec(), b"bar".to_vec());
        assert!(matches!(
            queue.enqueue_appointed_message(message, sequence + 1),
            Err(crate::Error::InvalidSequence)
        ));
        assert_eq!(channel.inspect(|c| c.appointed_messages.len()), Some(0));
    }

    #[test]
    fn over_appointments() {
        const MAX_APPOINTMENTS: usize = 8;

        let TestQueue { queue, channel } = new_queue();

        for _ in 0..MAX_APPOINTMENTS {
            assert!(channel.make_appointment().is_ok());
        }
        assert!(channel.make_appointment().is_err());
        queue.commit_appointments();
        let messages = queue.all_messages_grouped();
        insta::assert_debug_snapshot!(messages);
    }

    #[test]
    fn purge_messages() {
        let TestQueue { queue, channel } = new_queue();

        for _ in 0..3 {
            let sequence = channel.make_appointment().unwrap();
            let message = channel.prepare_with_data(b"foo".to_vec(), b"bar".to_vec());
            queue.enqueue_appointed_message(message, sequence).unwrap();
        }
        let _ = channel.make_appointment().unwrap();
        queue.commit_appointments();

        channel.inspect(|ch| {
            assert_eq!(ch.messages.len(), 1);
            assert_eq!(ch.appointed_messages.len(), 3);
            assert_eq!(ch.appointed_seqs.len(), 4);
        });

        queue.purge(|_| crate::SequenceInfo {
            next_sequence: 1,
            next_ap_sequence: 0,
            ap_sequences: vec![],
        });
        channel.inspect(|ch| {
            assert_eq!(ch.messages.len(), 0);
            assert_eq!(ch.appointed_messages.len(), 3);
            assert_eq!(ch.appointed_seqs.len(), 4);
        });

        queue.purge(|_| crate::SequenceInfo {
            next_sequence: 1,
            next_ap_sequence: 3,
            ap_sequences: vec![0, 1, 2],
        });
        channel.inspect(|ch| {
            assert_eq!(ch.messages.len(), 0);
            assert_eq!(ch.appointed_messages.len(), 3);
            assert_eq!(ch.appointed_seqs.len(), 4);
        });

        queue.purge(|_| crate::SequenceInfo {
            next_sequence: 1,
            next_ap_sequence: 3,
            ap_sequences: vec![0, 2],
        });
        channel.inspect(|ch| {
            assert_eq!(ch.messages.len(), 0);
            assert_eq!(ch.appointed_messages.len(), 2);
            assert_eq!(ch.appointed_seqs.len(), 3);
        });

        queue.purge(|_| crate::SequenceInfo {
            next_sequence: 1,
            next_ap_sequence: 4,
            ap_sequences: vec![0, 2, 3],
        });
        channel.inspect(|ch| {
            assert_eq!(ch.messages.len(), 0);
            assert_eq!(ch.appointed_messages.len(), 2);
            assert_eq!(ch.appointed_seqs.len(), 3);
        });

        queue.purge(|_| crate::SequenceInfo {
            next_sequence: 1,
            next_ap_sequence: 4,
            ap_sequences: vec![3],
        });
        channel.inspect(|ch| {
            assert_eq!(ch.messages.len(), 0);
            assert_eq!(ch.appointed_messages.len(), 0);
            assert_eq!(ch.appointed_seqs.len(), 1);
        });

        queue.purge(|_| crate::SequenceInfo {
            next_sequence: 1,
            next_ap_sequence: 4,
            ap_sequences: vec![],
        });
        channel.inspect(|ch| {
            assert_eq!(ch.messages.len(), 0);
            assert_eq!(ch.appointed_messages.len(), 0);
            assert_eq!(ch.appointed_seqs.len(), 0);
        });
    }

    #[test]
    fn multiple_appointments_should_be_synced_to_chain() {
        let TestQueue { queue, channel } = new_queue();
        for _ in 0..5 {
            channel.make_appointment().unwrap();
        }
        channel.inspect(|ch| {
            assert_eq!(ch.messages.len(), 0);
            assert_eq!(ch.appointing, 5);
            assert_eq!(ch.appointed_seqs.len(), 5);
        });

        queue.commit_appointments();
        channel.inspect(|ch| {
            assert_eq!(ch.messages.len(), 1);
            assert_eq!(ch.appointing, 0);
            assert_eq!(ch.appointed_seqs.len(), 5);
        });
    }

    #[test]
    fn forbid_push_sys_messages() {
        let TestQueue { queue, channel } = new_queue();

        for topic in [b"sys/foo" as &[u8], b"^sys/foo"] {
            let message = channel.prepare_with_data(vec![], topic);
            let sequence = channel.make_appointment().unwrap();
            assert!(matches!(
                queue.enqueue_appointed_message(message, sequence),
                Err(crate::Error::Forbidden)
            ));

            let message = channel.prepare_with_data(vec![], topic);
            assert!(matches!(
                queue.enqueue_message(message, Default::default()),
                Err(crate::Error::Forbidden)
            ));
        }

        let message = channel.prepare_message(&Appointment::new(1));
        assert!(matches!(
            queue.enqueue_message(message, Default::default()),
            Err(crate::Error::Forbidden)
        ));
    }
}

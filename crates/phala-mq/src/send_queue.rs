use crate::{
    Message, MessageOrigin, MessageSigner, Mutex, SenderId, SignedMessage, SigningMessage,
};
use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
struct Channel {
    sequence: u64,
    messages: Vec<SignedMessage>,
    dummy: bool,
}

#[derive(Clone, Default)]
pub struct MessageSendQueue {
    inner: Arc<Mutex<BTreeMap<SenderId, Channel>>>,
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
        constructor: impl FnOnce(u64) -> SignedMessage,
    ) {
        let mut inner = self.inner.lock();
        let entry = inner.entry(sender).or_default();
        if !entry.dummy {
            let message = constructor(entry.sequence);
            log::info!(target: "mq",
                "Sending message, from={}, to={:?}, seq={}",
                message.message.sender,
                message.message.destination,
                entry.sequence,
            );
            entry.messages.push(message);
        }
        entry.sequence += 1;
    }

    pub fn set_dummy_mode(&self, sender: SenderId, dummy: bool) {
        let mut inner = self.inner.lock();
        let entry = inner.entry(sender).or_default();
        entry.dummy = dummy;
    }

    pub fn all_messages(&self) -> Vec<SignedMessage> {
        let inner = self.inner.lock();
        inner
            .iter()
            .flat_map(|(_k, v)| v.messages.iter().cloned())
            .collect()
    }

    pub fn all_messages_grouped(&self) -> BTreeMap<MessageOrigin, Vec<SignedMessage>> {
        let inner = self.inner.lock();
        inner
            .iter()
            .map(|(k, v)| (k.clone(), v.messages.clone()))
            .collect()
    }

    pub fn messages(&self, sender: &SenderId) -> Vec<SignedMessage> {
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
    pub fn purge(&self, next_sequence_for: impl Fn(&SenderId) -> u64) {
        let mut inner = self.inner.lock();
        for (k, v) in inner.iter_mut() {
            let seq = next_sequence_for(k);
            v.messages.retain(|msg| msg.sequence >= seq);
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
        #[serde(default = "get_global_message_send_queue")]
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
        ) -> SigningMessage<Si> {
            let sender = self.sender.clone();
            let signer = self.signer.clone();
            let message = Message {
                sender,
                destination: to.into().into(),
                payload,
            };
            SigningMessage { message, signer }
        }
    }

    impl<T: MessageSigner + Clone> crate::traits::MessageChannel for MessageChannel<T> {
        fn push_data(&self, payload: Vec<u8>, to: impl Into<Path>) {
            let signing = self.prepare_with_data(payload, to);
            self.queue
                .enqueue_message(self.sender.clone(), move |sequence| signing.sign(sequence))
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
            self.prepare_with_data(payload, to)
        }
    }

    // const _: () = {
    //     use core::fmt;
    //     use serde::ser::SerializeStruct;
    //     use core::marker::PhantomData;
    //     use serde::de::{self, SeqAccess, Visitor};

    //     impl<Signer: Serialize> Serialize for MessageChannel<Signer> {
    //         fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    //         where
    //             S: serde::Serializer,
    //         {
    //             let mut state = serializer.serialize_struct("MessageChannel", 2)?;
    //             state.serialize_field("sender", &self.sender)?;
    //             state.serialize_field("signer", &self.signer)?;
    //             state.end()
    //         }
    //     }

    //     impl<'de, Signer: Deserialize<'de>> Deserialize<'de> for MessageChannel<Signer> {
    //         fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    //         where
    //             D: serde::Deserializer<'de>,
    //         {
    //             const FIELDS: &[&str] = &["sender", "signer"];
    //             deserializer.deserialize_struct("MessageChannel", FIELDS, MessageChannelVisitor<Signer>)
    //         }
    //     }

    //     #[derive(Default)]
    //     struct MessageChannelVisitor<Signer>(PhantomData<Signer>);

    //     impl<'de, Signer> Visitor<'de> for MessageChannelVisitor<Signer>
    //     where
    //         Signer: Deserialize<'de>,
    //     {
    //         type Value = MessageChannel<Signer>;

    //         fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    //             formatter.write_str("struct MessageChannel")
    //         }

    //         fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
    //         where
    //             V: SeqAccess<'de>,
    //         {
    //             let sender = seq
    //                 .next_element()?
    //                 .ok_or_else(|| de::Error::invalid_length(0, &self))?;
    //             let signer = seq
    //                 .next_element()?
    //                 .ok_or_else(|| de::Error::invalid_length(1, &self))?;
    //             let queue = global_message_send_queue();
    //             Ok(MessageChannel::new(queue, sender, signer))
    //         }
    //     }
    // };

    // fn global_message_dispatcher() -> MessageSendQueue {
    //     todo!("TODO.kevin.must")
    // }

    fn get_global_message_send_queue() -> MessageSendQueue {
        todo!("TODO.kevin.must")
    }
}

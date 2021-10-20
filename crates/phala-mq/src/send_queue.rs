use crate::{
    Message, MessageOrigin, MessageSigner, Mutex, SenderId, SignedMessage, SigningMessage,
};
use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};

#[derive(Default)]
struct Channel {
    sequence: u64,
    messages: Vec<SignedMessage>,
    dummy: bool,
}

#[derive(Clone, Default)]
pub struct MessageSendQueue {
    inner: Arc<Mutex<BTreeMap<SenderId, Channel>>>,
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

    #[derive(Clone)]
    pub struct MessageChannel<Si: MessageSigner> {
        queue: MessageSendQueue,
        sender: SenderId,
        signer: Si,
    }

    impl<Si: MessageSigner> MessageChannel<Si> {
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
                .enqueue_message(self.sender.clone(), move |sequence| {
                    signing.sign(sequence)
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
            self.prepare_with_data(payload, to)
        }
    }
}

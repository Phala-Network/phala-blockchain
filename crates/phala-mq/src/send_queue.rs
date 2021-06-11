use crate::types::{SignedMessage, Message, Origin};
use crate::Mutex;
use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};

#[derive(Clone)]
pub struct MessageSendQueue {
    // Map: sender -> (sequence, messages)
    inner: Arc<Mutex<BTreeMap<Origin, (u64, Vec<SignedMessage>)>>>,
}

impl MessageSendQueue {
    pub fn new() -> Self {
        MessageSendQueue {
            inner: Default::default(),
        }
    }
    pub fn create_handle<Si: Signer>(&self, sender: Origin, signer: Si) -> MessageSendHandle<Si> {
        MessageSendHandle::new(self.clone(), sender, signer)
    }

    pub fn queue_message(
        &self,
        sender: Origin,
        constructor: impl FnOnce(u64) -> SignedMessage,
    ) {
        let mut inner = self.inner.lock();
        let entry = inner.entry(sender).or_default();
        let message = constructor(entry.0);
        entry.1.push(message);
        entry.0 += 1;
    }

    pub fn all_messages(&self) -> Vec<SignedMessage> {
        let inner = self.inner.lock();
        inner
            .iter()
            .flat_map(|(k, v)| v.1.iter().cloned())
            .collect()
    }

    pub fn messages(&self, sender: &Origin) -> Vec<SignedMessage> {
        let inner = self.inner.lock();
        inner.get(sender).map(|x| x.1.clone()).unwrap_or(Vec::new())
    }

    /// Purge the messages which are aready accepted on chain.
    pub fn purge(&self, next_sequence_for: impl Fn(&Origin) -> u64) {
        let mut inner = self.inner.lock();
        for (k, v) in inner.iter_mut() {
            let seq = next_sequence_for(k);
            v.1.retain(|msg| msg.sequence >= seq);
        }
    }
}

pub use msg_handle::*;
mod msg_handle {
    use super::*;
    use crate::types::Path;

    pub trait Signer {
        fn sign(&self, sequence: u64, message: &Message) -> Vec<u8>;
    }

    #[derive(Clone)]
    pub struct MessageSendHandle<Si: Signer> {
        queue: MessageSendQueue,
        sender: Origin,
        signer: Si,
    }

    impl<Si: Signer> MessageSendHandle<Si> {
        pub fn new(queue: MessageSendQueue, sender: Origin, signer: Si) -> Self {
            MessageSendHandle {
                queue,
                sender,
                signer,
            }
        }

        pub fn send(&self, payload: Vec<u8>, to: impl Into<Path>) {
            let sender = self.sender.clone();
            let signer = &self.signer;

            self.queue.queue_message(sender.clone(), move |sequence| {
                let message = Message {
                    sender: sender,
                    destination: to.into(),
                    payload,
                };
                let signature = signer.sign(sequence, &message);
                SignedMessage { message, sequence, signature }
            })
        }
    }
}

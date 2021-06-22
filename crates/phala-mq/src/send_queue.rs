use crate::types::{Message, MessageToBeSigned, SignedMessage};
use crate::{MessageOrigin, MessageSigner, Mutex, SenderId};
use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};

#[derive(Clone, Default)]
pub struct MessageSendQueue {
    // Map: sender -> (last sequence, messages)
    inner: Arc<Mutex<BTreeMap<SenderId, (u64, Vec<SignedMessage>)>>>,
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
        let message = constructor(entry.0);
        entry.1.push(message);
        entry.0 += 1;
    }

    pub fn all_messages(&self) -> Vec<SignedMessage> {
        let inner = self.inner.lock();
        inner
            .iter()
            .flat_map(|(_k, v)| v.1.iter().cloned())
            .collect()
    }

    pub fn all_messages_grouped(&self) -> BTreeMap<MessageOrigin, Vec<SignedMessage>> {
        let inner = self.inner.lock();
        inner
            .iter()
            .map(|(k, v)| (k.clone(), v.1.clone()))
            .collect()
    }

    pub fn messages(&self, sender: &SenderId) -> Vec<SignedMessage> {
        let inner = self.inner.lock();
        inner.get(sender).map(|x| x.1.clone()).unwrap_or(Vec::new())
    }

    /// Purge the messages which are aready accepted on chain.
    pub fn purge(&self, next_sequence_for: impl Fn(&SenderId) -> u64) {
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
    use crate::{types::Path, BindTopic, MessageSigner, SenderId};
    use parity_scale_codec::Encode;

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

        pub fn send_data(&self, payload: Vec<u8>, to: impl Into<Path>) {
            let sender = self.sender.clone();
            let signer = &self.signer;

            self.queue.enqueue_message(sender.clone(), move |sequence| {
                let message = Message {
                    sender,
                    destination: to.into().into(),
                    payload,
                };
                let be_signed = MessageToBeSigned {
                    message: &message,
                    sequence,
                }
                .encode();
                let signature = signer.sign(&be_signed);
                // TODO.kevin: log
                // info!("send msg: data[{}], sig[{}], seq={}", be_signed.len(), signature.len(), sequence);
                SignedMessage {
                    message,
                    sequence,
                    signature,
                }
            })
        }

        pub fn sendto<M: Encode>(&self, message: &M, to: impl Into<Path>) {
            self.send_data(message.encode(), to)
        }

        pub fn send<M: Encode + BindTopic>(&self, message: &M) {
            self.sendto(message, <M as BindTopic>::TOPIC)
        }
    }
}

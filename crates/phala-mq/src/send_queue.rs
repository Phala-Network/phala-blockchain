use crate::{
    Message, MessageOrigin, MessageSigner, Mutex, SenderId, SignedMessage, SigningMessage,
};
use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Channel {
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

            if log::log_enabled!(target: "phala_mq", log::Level::Debug) {
                log::debug!(target: "phala_mq",
                    "Sending message, from={}, to={:?}, seq={}, payload_hash={}",
                    message.message.sender,
                    message.message.destination,
                    entry.sequence,
                    hex::encode(sp_core::blake2_256(&message.message.payload)),
                );
            } else {
                log::info!(target: "phala_mq",
                    "Sending message, from={}, to={:?}, seq={}",
                    message.message.sender,
                    message.message.destination,
                    entry.sequence,
                );
            }
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

    pub fn dump_state(&self, sender: &SenderId) -> Option<Channel> {
        let inner = self.inner.lock();
        inner.get(sender).cloned()
    }

    pub fn load_state(&self, sender: &SenderId, state: Channel) {
        let mut inner = self.inner.lock();
        inner.insert(sender.clone(), state);
    }
}

pub use msg_channel::*;
mod msg_channel {
    use super::*;
    use crate::{types::Path, MessageSigner, SenderId};

    #[derive(Clone, Serialize, Deserialize, ::scale_info::TypeInfo)]
    pub struct MessageChannel<Si> {
        #[codec(skip)]
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
        type Signer = T;

        fn push_data(&self, payload: Vec<u8>, to: impl Into<Path>) {
            let signing = self.prepare_with_data(payload, to);
            self.queue
                .enqueue_message(self.sender.clone(), move |sequence| signing.sign(sequence))
        }

        /// Set the channel to dummy mode which increasing the sequence but dropping the message.
        fn set_dummy(&self, dummy: bool) {
            self.queue.set_dummy_mode(self.sender.clone(), dummy);
        }

        fn set_signer(&mut self, signer: Self::Signer) {
            self.signer = signer;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{MessageChannel as _, MessagePrepareChannel as _};

    use alloc::vec::Vec;
    use sp_core::Pair as _;
    use type_info_stringify::type_info_stringify;

    crate::bind_topic!(TestMessage, b"/test");
    #[derive(parity_scale_codec::Encode)]
    struct TestMessage(Vec<u8>);

    #[derive(Clone, Serialize, Deserialize)]
    struct TestSigner;
    impl MessageSigner for TestSigner {
        fn sign(&self, _data: &[u8]) -> Vec<u8> {
            vec![0u8; 32]
        }
    }

    #[test]
    fn it_works() {
        env_logger::builder()
            .filter_level(log::LevelFilter::Info)
            .is_test(true)
            .try_init()
            .ok();

        let mut mq = MessageSendQueue::new();
        let mut ch =
            msg_channel::MessageChannel::new(mq.clone(), MessageOrigin::Reserved, TestSigner);

        ch.clone().set_signer(TestSigner);
        ch.set_dummy(false);
        ch.push_message(&TestMessage(b"hello".to_vec()));

        assert_eq!(mq.count_messages(), 1);

        let msg = ch.prepare_message(&TestMessage(b"hello".to_vec()));
        assert_eq!(msg.message.sender, MessageOrigin::Reserved);

        let serde_ch = serde_cbor::to_vec(&ch).unwrap();
        ch = crate::checkpoint_helper::using_send_mq(&mut mq, move || {
            serde_cbor::from_reader(&serde_ch[..]).unwrap()
        });
        ch.push_message(&TestMessage(b"hello".to_vec()));
        assert_eq!(mq.count_messages(), 2);

        insta::assert_debug_snapshot!(mq.all_messages_grouped());
        insta::assert_debug_snapshot!(mq.all_messages());
        insta::assert_debug_snapshot!(mq.messages(&MessageOrigin::Reserved));

        let mq2 = MessageSendQueue::default();
        let mut dump_ch = mq.dump_state(&MessageOrigin::Reserved).unwrap();
        let serde_ch = serde_cbor::to_vec(&dump_ch).unwrap();
        dump_ch = serde_cbor::from_reader(&serde_ch[..]).unwrap();
        mq2.load_state(&MessageOrigin::Reserved, dump_ch);

        assert_eq!(mq2.count_messages(), 2);
        assert_eq!(
            mq2.messages(&MessageOrigin::Reserved),
            mq.messages(&MessageOrigin::Reserved)
        );

        let serde_mq = serde_cbor::to_vec(&mq).unwrap();
        mq = serde_cbor::from_reader(&serde_mq[..]).unwrap();
        assert_eq!(mq.count_messages(), 2);
    }

    #[test]
    fn test_serde_mq() {
        env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .is_test(true)
            .try_init()
            .ok();
        let mut mq = MessageSendQueue::new();
        let mut ch = msg_channel::MessageChannel::new(
            mq.clone(),
            MessageOrigin::Reserved,
            crate::Sr25519Signer::from(sp_core::sr25519::Pair::from_seed(&[0u8; 32])),
        );
        let serde_ch = serde_cbor::to_vec(&ch).unwrap();
        ch = crate::checkpoint_helper::using_send_mq(&mut mq, move || {
            serde_cbor::from_reader(&serde_ch[..]).unwrap()
        });
        ch.clone().push_message(&TestMessage(b"hello".to_vec()));
        assert_eq!(mq.count_messages(), 1);
    }

    #[test]
    fn dump_type_info() {
        insta::assert_display_snapshot!(type_info_stringify::<
            msg_channel::MessageChannel<crate::Sr25519Signer>,
        >());
    }
}

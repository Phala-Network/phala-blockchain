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
            log::error!(
                "MsgChannel::received(): error - received seq {} larger than max seq {}",
                seq,
                self.sequence
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

pub mod osp {
    // OSP (Optinal Secret Protocol): A topic using OSP means it accepting either Payload::Plain or Payload::Encrypted Message.

    pub use decrypt::*;
    pub use encrypt::*;

    mod encrypt {}

    mod decrypt {
        use crate::cryptography::{self, AeadCipher};
        use anyhow::Context;
        use core::{any, marker::PhantomData};
        use parity_scale_codec::Decode;
        use phala_mq::{BindTopic, MessageOrigin, ReceiveError, TypedReceiver};
        use ring::agreement::EphemeralPrivateKey;

        #[derive(Decode, Debug)]
        pub enum OspPayload<T> {
            Plain(T),
            Encrypted(AeadCipher),
        }

        impl<T: BindTopic> BindTopic for OspPayload<T> {
            const TOPIC: &'static [u8] = T::TOPIC;
        }

        pub trait Peeler {
            type Wrp;
            type Msg;
            fn peel(&self, msg: Self::Wrp) -> Result<Self::Msg, anyhow::Error>;
        }

        pub struct PlainPeeler<T>(PhantomData<T>);

        impl<T> Peeler for PlainPeeler<T> {
            type Wrp = T;
            type Msg = T;
            fn peel(&self, msg: Self::Wrp) -> Result<Self::Msg, anyhow::Error> {
                Ok(msg)
            }
        }

        pub struct OspPeeler<T> {
            privkey: EphemeralPrivateKey,
            _t: PhantomData<T>,
        }

        impl<T> OspPeeler<T> {
            pub fn new(privkey: EphemeralPrivateKey) -> Self {
                OspPeeler {
                    privkey: privkey,
                    _t: PhantomData,
                }
            }
        }

        impl<T: Decode> Peeler for OspPeeler<T> {
            type Wrp = OspPayload<T>;
            type Msg = T;
            fn peel(&self, msg: Self::Wrp) -> Result<Self::Msg, anyhow::Error> {
                match msg {
                    OspPayload::Plain(msg) => Ok(msg),
                    OspPayload::Encrypted(cipher) => {
                        let msg = cryptography::decrypt(&cipher, &self.privkey)
                            .context("Decrypt Osp encrypted message failed")?
                            .msg;
                        let msg = Decode::decode(&mut &msg[..]).map_err(|_| {
                            anyhow::anyhow!("SCALE decode Osp decrypted data failed")
                        })?;
                        Ok(msg)
                    }
                }
            }
        }

        pub struct PeelingReceiver<Msg, Wrp, Plr> {
            receiver: TypedReceiver<Wrp>,
            peeler: Plr,
            _msg: PhantomData<Msg>,
        }

        impl<Msg, Wrp> PeelingReceiver<Msg, Wrp, PlainPeeler<Msg>> {
            pub fn new_plain(receiver: TypedReceiver<Wrp>) -> Self {
                PeelingReceiver {
                    receiver,
                    peeler: PlainPeeler(Default::default()),
                    _msg: Default::default(),
                }
            }
        }

        impl<Msg, Wrp> PeelingReceiver<Msg, Wrp, OspPeeler<Msg>> {
            pub fn new_osp(receiver: TypedReceiver<Wrp>, privkey: EphemeralPrivateKey) -> Self {
                PeelingReceiver {
                    receiver,
                    peeler: OspPeeler::new(privkey),
                    _msg: Default::default(),
                }
            }
        }

        impl<Msg, Plr, Wrp> PeelingReceiver<Msg, Wrp, Plr>
        where
            Plr: Peeler<Wrp = Wrp, Msg = Msg>,
            Msg: Decode,
            Wrp: Decode,
        {
            pub fn try_next(&mut self) -> Result<Option<(u64, Msg, MessageOrigin)>, anyhow::Error> {
                let omsg = self
                    .receiver
                    .try_next()
                    .map_err(|e| anyhow::anyhow!("{}", e))?;
                let (seq, msg, origin) = match omsg {
                    Some(x) => x,
                    None => return Ok(None),
                };
                let msg = self.peeler.peel(msg)?;
                Ok(Some((seq, msg, origin)))
            }

            pub fn peek_seq(&self) -> Result<Option<u64>, ReceiveError> {
                self.receiver.peek_seq()
            }
        }
    }
}

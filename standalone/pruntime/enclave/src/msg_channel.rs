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

    mod encrypt {
    }

    mod decrypt {
        use crate::cryptography::{self, AeadCipher};
        use core::marker::PhantomData;
        use parity_scale_codec::Decode;
        use anyhow::Context;
        use ring::agreement::EphemeralPrivateKey;

        #[derive(Decode)]
        enum OspPayload<T> {
            Plain(T),
            Encrypted(AeadCipher),
        }

        trait Peeler {
            type Outer;
            type Inner;
            type Error;
            fn peel(&self, msg: Self::Outer) -> Result<Self::Inner, Self::Error>;
        }

        #[derive(Default)]
        struct PlainPeeler<T>(PhantomData<T>);

        impl<T> Peeler for PlainPeeler<T> {
            type Outer = T;
            type Inner = T;
            type Error = ();
            fn peel(&self, msg: Self::Outer) -> Result<Self::Inner, Self::Error> {
                Ok(msg)
            }
        }

        struct OspPeeler<T> {
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
            type Outer = OspPayload<T>;
            type Inner = T;
            type Error = anyhow::Error;
            fn peel(&self, msg: Self::Outer) -> Result<Self::Inner, Self::Error> {
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
    }
}

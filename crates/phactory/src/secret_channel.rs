pub use receiver::*;
pub use sender::*;

use parity_scale_codec::{Decode, Encode};
use phactory_api::crypto::EncryptedData;

#[derive(Encode, Decode, Debug)]
pub enum Payload<T> {
    Plain(T),
    Encrypted(EncryptedData),
}

mod sender {
    use crate::contracts::Data as OpaqueData;
    use phactory_api::crypto::{ecdh, EncryptedData};
    use phala_crypto::ecdh::EcdhPublicKey;
    use phala_mq::traits::{MessageChannel, MessageChannelBase, MessagePrepareChannel};
    use phala_mq::Path;

    pub type KeyPair = ecdh::EcdhKey;

    #[derive(Clone)]
    pub struct SecretMessageChannel<'a, MsgChan> {
        key: &'a KeyPair,
        mq: &'a MsgChan,
    }

    pub struct BoundSecretMessageChannel<'a, MsgChan> {
        inner: SecretMessageChannel<'a, MsgChan>,
        remote_pubkey: Option<&'a ecdh::EcdhPublicKey>,
    }

    impl<'a, MsgChan: Clone> SecretMessageChannel<'a, MsgChan> {
        pub fn new(key: &'a KeyPair, mq: &'a MsgChan) -> Self {
            SecretMessageChannel { key, mq }
        }

        pub fn bind_remote_key(
            &self,
            remote_pubkey: Option<&'a ecdh::EcdhPublicKey>,
        ) -> BoundSecretMessageChannel<'a, MsgChan> {
            BoundSecretMessageChannel {
                inner: self.clone(),
                remote_pubkey,
            }
        }
    }

    impl<'a, MsgChan> BoundSecretMessageChannel<'a, MsgChan> {
        fn encrypt_payload(&self, data: Vec<u8>) -> super::Payload<OpaqueData> {
            if let Some(remote_pubkey) = self.remote_pubkey {
                let iv = crate::generate_random_iv();
                let data = EncryptedData::encrypt(self.inner.key, remote_pubkey, iv, &data)
                    .expect("Encrypt message failed?");
                super::Payload::Encrypted(data)
            } else {
                super::Payload::Plain(OpaqueData(data))
            }
        }
    }

    impl<'a, MsgChan: MessageChannelBase> MessageChannelBase
        for BoundSecretMessageChannel<'a, MsgChan>
    {
        fn last_hash(&self) -> phala_mq::MqHash {
            self.inner.mq.last_hash()
        }
    }

    impl<'a, MsgChan: MessageChannel> MessageChannel for BoundSecretMessageChannel<'a, MsgChan> {
        fn push_data(&self, data: Vec<u8>, to: impl Into<Path>, hash: phala_mq::MqHash) {
            let payload = self.encrypt_payload(data);
            self.inner.mq.push_message_to(&payload, to, hash)
        }
    }

    impl<'a, MsgChan: MessagePrepareChannel> phala_mq::traits::MessagePrepareChannel
        for BoundSecretMessageChannel<'a, MsgChan>
    {
        type Signer = MsgChan::Signer;

        fn prepare_with_data(
            &self,
            data: Vec<u8>,
            to: impl Into<Path>,
        ) -> phala_mq::SigningMessage<Self::Signer> {
            let payload = self.encrypt_payload(data);
            self.inner.mq.prepare_message_to(&payload, to)
        }
    }

    pub fn bind_remote<'a, MsgChan: Clone>(
        channel: &'a MsgChan,
        ecdh_key: &'a ecdh::EcdhKey,
        remote_pubkey: Option<&'a EcdhPublicKey>,
    ) -> BoundSecretMessageChannel<'a, MsgChan> {
        SecretMessageChannel::new(ecdh_key, channel).bind_remote_key(remote_pubkey)
    }
}

mod receiver {
    use super::Payload;
    use core::marker::PhantomData;
    use parity_scale_codec::Decode;
    use phactory_api::crypto::ecdh;
    use phala_mq::{MessageOrigin, ReceiveError, TypedReceiver};
    use serde::{Deserialize, Serialize};
    pub type SecretReceiver<Msg> = PeelingReceiver<Msg, Payload<Msg>, SecretPeeler<Msg>>;

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

    #[derive(Serialize, Deserialize)]
    pub struct SecretPeeler<T> {
        #[serde(with = "super::ecdh_serde")]
        ecdh_key: ecdh::EcdhKey,
        _t: PhantomData<T>,
    }

    impl<T> SecretPeeler<T> {
        pub fn new(ecdh_key: ecdh::EcdhKey) -> Self {
            SecretPeeler {
                ecdh_key,
                _t: PhantomData,
            }
        }
    }

    impl<T: Decode> Peeler for SecretPeeler<T> {
        type Wrp = Payload<T>;
        type Msg = T;
        fn peel(&self, msg: Self::Wrp) -> Result<Self::Msg, anyhow::Error> {
            match msg {
                Payload::Plain(msg) => Ok(msg),
                Payload::Encrypted(msg) => {
                    let data = msg.decrypt(&self.ecdh_key).map_err(|err| {
                        anyhow::anyhow!("SecretPeeler decrypt message failed: {:?}", err)
                    })?;
                    let msg = Decode::decode(&mut &data[..])
                        .map_err(|_| anyhow::anyhow!("SCALE decode decrypted data failed"))?;
                    Ok(msg)
                }
            }
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct PeelingReceiver<Msg, Wrp, Plr> {
        #[serde(bound(serialize = "", deserialize = ""))]
        receiver: TypedReceiver<Wrp>,
        peeler: Plr,
        _msg: PhantomData<Msg>,
    }

    impl<Msg, Wrp> PeelingReceiver<Msg, Wrp, PlainPeeler<Msg>> {
        #[allow(unused)]
        pub fn new_plain(receiver: TypedReceiver<Wrp>) -> Self {
            PeelingReceiver {
                receiver,
                peeler: PlainPeeler(Default::default()),
                _msg: Default::default(),
            }
        }
    }

    impl<Msg, Wrp> PeelingReceiver<Msg, Wrp, SecretPeeler<Msg>> {
        pub fn new_secret(receiver: TypedReceiver<Wrp>, ecdh_key: ecdh::EcdhKey) -> Self {
            PeelingReceiver {
                receiver,
                peeler: SecretPeeler::new(ecdh_key),
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

        pub fn peek_ind(&self) -> Result<Option<u64>, ReceiveError> {
            self.receiver.peek_ind()
        }
    }
}

pub(crate) mod ecdh_serde {
    use crate::EcdhKey;
    use phala_serde_more as more;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(ecdh: &EcdhKey, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        more::scale_bytes::serialize(&ecdh.secret(), serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<EcdhKey, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secret = more::scale_bytes::deserialize(deserializer)?;
        Ok(EcdhKey::from_secret(&secret).or(Err(serde::de::Error::custom("invalid ECDH key")))?)
    }
}

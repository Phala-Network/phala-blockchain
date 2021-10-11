pub use receiver::*;
pub use sender::*;

use crate::light_validation::utils::storage_map_prefix_blake2_128_concat;
use phactory_api::crypto::EncryptedData;
use parity_scale_codec::{Decode, Encode};

#[derive(Encode, Decode, Debug)]
pub enum Payload<T> {
    Plain(T),
    Encrypted(EncryptedData),
}

mod sender {
    use phactory_api::crypto::{ecdh, EncryptedData};
    use parity_scale_codec::Encode;
    use phala_mq::{BindTopic, Path, Sr25519MessageChannel};

    pub type KeyPair = ecdh::EcdhKey;

    #[allow(unused)] // TODO.kevin: remove this.
    pub struct SecretMessageChannel<'a> {
        key: &'a KeyPair,
        mq: &'a Sr25519MessageChannel,
        key_map: &'a dyn Fn(&[u8]) -> Option<ecdh::EcdhPublicKey>,
    }

    #[allow(unused)] // TODO.kevin: remove this.
    impl<'a> SecretMessageChannel<'a> {
        pub fn new(
            key: &'a KeyPair,
            mq: &'a Sr25519MessageChannel,
            key_map: &'a dyn Fn(&[u8]) -> Option<ecdh::EcdhPublicKey>,
        ) -> Self {
            SecretMessageChannel { key, mq, key_map }
        }

        pub fn pubkey_for_topic(&self, topic: &[u8]) -> Option<ecdh::EcdhPublicKey> {
            (self.key_map)(topic)
        }

        pub fn sendto<M: Encode>(
            &self,
            to: impl Into<Path>,
            message: &M,
            remote_pubkey: Option<&ecdh::EcdhPublicKey>,
        ) {
            let data = message.encode();
            let payload = if let Some(remote_pubkey) = remote_pubkey {
                let iv = crate::generate_random_iv();
                let data = EncryptedData::encrypt(self.key, remote_pubkey, iv, &data)
                    .expect("Encrypt message failed?");
                super::Payload::Encrypted(data)
            } else {
                super::Payload::Plain(message)
            };
            self.mq.send_data(payload.encode(), to)
        }

        pub fn send<M: Encode + BindTopic>(
            &self,
            message: &M,
            remote_pubkey: Option<&ecdh::EcdhPublicKey>,
        ) {
            self.sendto(<M as BindTopic>::topic(), message, remote_pubkey)
        }
    }
}

mod receiver {
    use super::Payload;
    use core::marker::PhantomData;
    use phactory_api::crypto::ecdh;
    use parity_scale_codec::Decode;
    use phala_mq::{MessageOrigin, ReceiveError, TypedReceiver};
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

    pub struct SecretPeeler<T> {
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

    pub struct PeelingReceiver<Msg, Wrp, Plr> {
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

/// Calculates the Substrate storage key prefix for a StorageMap
pub fn storage_prefix_for_topic_pubkey(topic: &[u8]) -> Vec<u8> {
    use phala_pallets::pallet_mq::StorageMapTrait as _;

    type TopicKey = phala_pallets::pallet_registry::TopicKey<chain::Runtime>;

    let module_prefix = TopicKey::module_prefix();
    let storage_prefix = TopicKey::storage_prefix();

    storage_map_prefix_blake2_128_concat(module_prefix, storage_prefix, &topic)
}

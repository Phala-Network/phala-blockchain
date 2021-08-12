pub use receiver::*;
pub use sender::*;

use crate::light_validation::utils::storage_map_prefix_blake2_128_concat;
use crate::std::vec::Vec;

mod sender {
    use crate::std::vec::Vec;
    use enclave_api::crypto::{ecdh, EncryptedData};
    use parity_scale_codec::Encode;
    use phala_mq::{BindTopic, Path, Sr25519MessageChannel};

    pub type KeyPair = ecdh::EcdhKey;

    pub struct SecretMessageChannel<'a> {
        key: &'a KeyPair,
        mq: &'a Sr25519MessageChannel,
        key_map: &'a dyn Fn(&Path) -> Option<ecdh::EcdhPublicKey>,
    }

    impl<'a> SecretMessageChannel<'a> {
        pub fn new(
            key: &'a KeyPair,
            mq: &'a Sr25519MessageChannel,
            key_map: &'a dyn Fn(&Path) -> Option<ecdh::EcdhPublicKey>,
        ) -> Self {
            SecretMessageChannel { key, mq, key_map }
        }

        pub fn pubkey_for_topic(&self, topic: &Path) -> Option<ecdh::EcdhPublicKey> {
            (self.key_map)(topic)
        }

        pub fn sendto<M: Encode>(
            &self,
            to: impl Into<Path>,
            message: &M,
            remote_pubkey: &ecdh::EcdhPublicKey,
        ) {
            let data = message.encode();
            let iv = crate::generate_random_iv();
            let payload = EncryptedData::encrypt(self.key, &remote_pubkey, iv, &data)
                .expect("Encrypt message failed?");
            self.mq.send_data(payload.encode(), to)
        }

        pub fn send<M: Encode + BindTopic>(
            &self,
            message: &M,
            remote_pubkey: &ecdh::EcdhPublicKey,
        ) {
            self.sendto(<M as BindTopic>::topic(), message, remote_pubkey)
        }
    }
}

mod receiver {
    use core::marker::PhantomData;
    use enclave_api::crypto::{ecdh, EncryptedData};
    use parity_scale_codec::Decode;
    use phala_mq::{MessageOrigin, ReceiveError, TypedReceiver};

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
                ecdh_key: ecdh_key,
                _t: PhantomData,
            }
        }
    }

    impl<T: Decode> Peeler for SecretPeeler<T> {
        type Wrp = EncryptedData;
        type Msg = T;
        fn peel(&self, msg: Self::Wrp) -> Result<Self::Msg, anyhow::Error> {
            let data = msg
                .decrypt(&self.ecdh_key)
                .map_err(|err| anyhow::anyhow!("SecretPeeler decrypt message failed: {:?}", err))?;
            let msg = Decode::decode(&mut &data[..])
                .map_err(|_| anyhow::anyhow!("SCALE decode decrypted data failed"))?;
            Ok(msg)
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
pub fn storage_prefix_for_topic_pubkey(topic: &phala_mq::Path) -> Vec<u8> {
    use phala_pallets::pallet_mq::StorageMapTrait as _;

    type TopicKey = phala_pallets::pallet_registry::TopicKey<chain::Runtime>;

    let module_prefix = TopicKey::module_prefix();
    let storage_prefix = TopicKey::storage_prefix();

    storage_map_prefix_blake2_128_concat(module_prefix, storage_prefix, &topic)
}

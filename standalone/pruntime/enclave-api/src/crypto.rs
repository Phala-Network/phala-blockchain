use alloc::vec::Vec;
use parity_scale_codec::{Decode, Encode};

pub use phala_crypto::{aead, ecdh, CryptoError};

#[derive(Clone, Encode, Decode)]
pub struct EncryptedData {
    pub iv: aead::IV,
    pub pubkey: ecdh::EcdhPublicKey,
    pub data: Vec<u8>,
}

impl EncryptedData {
    pub fn decrypt(&self, key: &ecdh::EcdhKey) -> Result<Vec<u8>, CryptoError> {
        let sk = ecdh::agree(key, &self.pubkey)?;
        let mut tmp_data = self.data.clone();
        let msg = aead::decrypt(&self.iv, &sk, &mut tmp_data)?;
        Ok(msg.to_vec())
    }

    pub fn encrypt(
        key: &ecdh::EcdhKey,
        remote_pubkey: &ecdh::EcdhPublicKey,
        iv: aead::IV,
        data: &[u8],
    ) -> Result<Self, CryptoError> {
        let sk = ecdh::agree(&key, &remote_pubkey[..])?;
        let mut data = data.to_vec();
        aead::encrypt(&iv, &sk, &mut data)?;
        Ok(Self {
            iv,
            pubkey: key.public(),
            data,
        })
    }
}

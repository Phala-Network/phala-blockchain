use aead::{AeadCore, AeadInPlace, NewAead};
use aead_io::{DecryptBE32BufReader, EncryptBE32BufWriter};
use alloc::vec::Vec;
use core::convert::TryInto;
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, MAX_TAG_LEN};
use typenum::{consts::*, Unsigned};

const TAG_LEN: usize = MAX_TAG_LEN;

pub struct Aes128Gcm {
    key: LessSafeKey,
}

impl Aes128Gcm {
    pub fn from_key(key: [u8; 16]) -> Self {
        Self::new((&key).into())
    }
}

impl NewAead for Aes128Gcm {
    type KeySize = U16;

    fn new(key: &aead::Key<Self>) -> Self {
        let unbound_key = UnboundKey::new(&ring::aead::AES_128_GCM, key)
            .expect("Aes128Gcm::new should always succeed");
        Self {
            key: LessSafeKey::new(unbound_key),
        }
    }
}

impl AeadCore for Aes128Gcm {
    type NonceSize = U12;
    type TagSize = U16;
    type CiphertextOverhead = U0;
}

impl AeadInPlace for Aes128Gcm {
    fn encrypt_in_place_detached(
        &self,
        nonce: &aead::Nonce<Self>,
        associated_data: &[u8],
        buffer: &mut [u8],
    ) -> aead::Result<aead::Tag<Self>> {
        let nonce = Nonce::assume_unique_for_key((*nonce).into());
        let aad = Aad::from(associated_data);
        self.key
            .seal_in_place_separate_tag(nonce, aad, buffer)
            .map(|tag| {
                let tag: [u8; TAG_LEN] = tag.as_ref().try_into().expect("Tag length should be 16");
                tag.into()
            })
            .map_err(|_| aead::Error)
    }

    fn decrypt_in_place_detached(
        &self,
        nonce: &aead::Nonce<Self>,
        associated_data: &[u8],
        buffer: &mut [u8],
        tag: &aead::Tag<Self>,
    ) -> aead::Result<()> {
        let nonce = Nonce::assume_unique_for_key((*nonce).into());
        let aad = Aad::from(associated_data);
        let mut tmp_buffer = alloc::vec![];
        tmp_buffer.extend_from_slice(buffer);
        tmp_buffer.extend_from_slice(tag.as_ref());
        let out = self
            .key
            .open_in_place(nonce, aad, &mut tmp_buffer)
            .map_err(|_| aead::Error)?;
        if buffer.len() != out.len() {
            return Err(aead::Error);
        }
        buffer.copy_from_slice(out);
        Ok(())
    }

    fn decrypt_in_place(
        &self,
        nonce: &aead::Nonce<Self>,
        associated_data: &[u8],
        buffer: &mut dyn aead::Buffer,
    ) -> aead::Result<()> {
        let nonce = Nonce::assume_unique_for_key((*nonce).into());
        let aad = Aad::from(associated_data);

        if buffer.len() < Self::TagSize::to_usize() {
            return Err(aead::Error);
        }

        let tag_pos = buffer.len() - Self::TagSize::to_usize();
        let _ = self
            .key
            .open_in_place(nonce, aad, buffer.as_mut())
            .map_err(|_| aead::Error)?;
        buffer.truncate(tag_pos);
        Ok(())
    }
}

pub fn new_aes128gcm_reader<R>(
    key: [u8; 16],
    reader: R,
) -> DecryptBE32BufReader<Aes128Gcm, Vec<u8>, R> {
    let aead = Aes128Gcm::from_key(key);
    DecryptBE32BufReader::from_aead(aead, Vec::with_capacity(256), reader)
        .expect("Create DecryptBE32BufReader should always success with valid capacity")
}

pub fn new_aes128gcm_writer<W: std::io::Write>(
    key: [u8; 16],
    nonce: [u8; 7],
    writer: W,
) -> EncryptBE32BufWriter<Aes128Gcm, Vec<u8>, W> {
    let aead = Aes128Gcm::from_key(key);
    EncryptBE32BufWriter::from_aead(aead, (&nonce[..]).into(), Vec::with_capacity(128), writer)
        .expect("Create EncryptBE32BufWriter should always success with valid capacity")
}

#[cfg(test)]
mod tests {
    use super::*;

    const LONG_TEXT: &[u8] = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Curabitur eu erat non turpis viverra mollis vel a mauris. Vestibulum luctus justo vitae diam ultrices, eget vehicula velit consectetur. Sed ut sapien odio. Nullam non porttitor augue. Duis euismod, augue sed blandit eleifend, leo enim rhoncus lacus, in efficitur metus massa quis justo. Nunc velit quam, aliquam vitae enim ut, facilisis molestie odio. Phasellus nec euismod nisi, sit amet dignissim arcu. Nullam pulvinar aliquam purus ut aliquet. Sed iaculis, odio in luctus molestie, purus dui vehicula est, sed egestas erat diam sed arcu. Cras venenatis magna vitae tristique mattis.";

    #[test]
    fn test_rw_short_text_aeadio() {
        encrypt_decrypt_aeadio(b"hello world!");
    }

    #[test]
    fn test_rw_long_text_aeadio() {
        encrypt_decrypt_aeadio(LONG_TEXT);
    }

    fn encrypt_decrypt_aeadio(plaintext: &[u8]) {
        use std::io::{Read, Write};

        let key = b"super secret key";

        let ciphertext = {
            let aead = Aes128Gcm::from_key(*key);
            let mut ciphertext = Vec::default();

            {
                let nonce = Default::default();
                let mut writer = EncryptBE32BufWriter::from_aead(
                    aead,
                    &nonce,
                    Vec::with_capacity(128),
                    &mut ciphertext,
                )
                .unwrap();
                writer.write_all(plaintext).unwrap();
                writer.flush().unwrap();
            }

            ciphertext
        };

        let decrypted = {
            let aead = Aes128Gcm::from_key(*key);
            let mut reader = DecryptBE32BufReader::from_aead(
                aead,
                Vec::with_capacity(256),
                ciphertext.as_slice(),
            )
            .unwrap();
            let mut out = Vec::new();
            let _ = reader.read_to_end(&mut out).unwrap();
            out
        };
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_stream_short_text() {
        encrypt_decrypt(b"hello world!");
    }

    #[test]
    fn test_stream_long_text() {
        encrypt_decrypt(LONG_TEXT);
    }

    #[cfg(test)]
    fn encrypt_decrypt(plaintext: &[u8]) {
        use std::io::{Read, Write};

        let key = b"super secret key";

        let ciphertext = {
            let mut ciphertext = Vec::default();

            {
                let nonce = Default::default();
                let mut writer = new_aes128gcm_writer(*key, nonce, &mut ciphertext);
                writer.write_all(plaintext).unwrap();
                writer.flush().unwrap();
            }

            ciphertext
        };

        let decrypted = {
            let mut reader = new_aes128gcm_reader(*key, &ciphertext[..]);
            let mut out = Vec::new();
            let _ = reader.read_to_end(&mut out).unwrap();
            out
        };
        assert_eq!(decrypted, plaintext);
    }
}

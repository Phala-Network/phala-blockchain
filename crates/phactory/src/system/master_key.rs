use std::path::PathBuf;
use std::vec::Vec;

use parity_scale_codec::{Decode, Encode};
use phala_types::{wrap_content_to_sign, SignedContentType};
use sp_core::sr25519;

use phala_crypto::sr25519::{Signature, Signing, Sr25519SecretKey};

use crate::pal::Sealing;

/// Master key filepath
pub const MASTER_KEY_FILE: &str = "master_key.seal";

#[derive(Debug, Encode, Decode, Clone)]
struct PersistentMasterKey {
    secret: Sr25519SecretKey,
    signature: Signature,
}

#[derive(Debug, Encode, Decode, PartialEq, Eq, Clone, ::scale_info::TypeInfo)]
pub struct RotatedMasterKey {
    pub rotation_id: u64,
    pub block_height: chain::BlockNumber,
    pub secret: Sr25519SecretKey,
}

#[derive(Debug, Encode, Decode, Clone)]
struct MasterKeyHistory {
    rotations: Vec<RotatedMasterKey>,
}

#[derive(Debug, Encode, Decode, Clone)]
struct PersistentMasterKeyHistory {
    payload: MasterKeyHistory,
    signature: Signature,
}

#[derive(Debug, Encode, Decode)]
enum MasterKeySeal {
    // Deprecated.
    V1(PersistentMasterKey),
    V2(PersistentMasterKeyHistory),
}

fn master_key_file_path(sealing_path: &str) -> PathBuf {
    PathBuf::from(sealing_path).join(MASTER_KEY_FILE)
}

/// Seal master key seeds with signature to ensure integrity
pub fn seal(
    sealing_path: String,
    master_key_history: &[RotatedMasterKey],
    identity_key: &sr25519::Pair,
    sys: &impl Sealing,
) {
    let payload = MasterKeyHistory {
        rotations: master_key_history.to_owned(),
    };
    let encoded = payload.encode();
    let wrapped = wrap_content_to_sign(&encoded, SignedContentType::MasterKeyStore);
    let signature = identity_key.sign_data(&wrapped);

    let data = MasterKeySeal::V2(PersistentMasterKeyHistory { payload, signature });
    let filepath = master_key_file_path(&sealing_path);
    info!("Seal master key to {}", filepath.as_path().display());
    // TODO.shelven: seal with identity key so the newly handovered pRuntime do not need to do an extra sync to get master
    // key
    sys.seal_data(filepath, &data.encode())
        .expect("Seal master key failed");
}

pub fn gk_master_key_exists(sealing_path: &str) -> bool {
    master_key_file_path(sealing_path).exists()
}

/// Unseal local master key seeds and verify signature
///
/// This function could panic a lot.
pub fn try_unseal(
    sealing_path: String,
    identity_key: &sr25519::Pair,
    sys: &impl Sealing,
) -> Vec<RotatedMasterKey> {
    let filepath = master_key_file_path(&sealing_path);
    info!("Unseal master key from {}", filepath.as_path().display());
    let sealed_data = match sys
        .unseal_data(&filepath)
        .expect("Unseal master key failed")
    {
        Some(data) => data,
        None => {
            warn!("No sealed master key");
            return vec![];
        }
    };

    let versioned_data =
        MasterKeySeal::decode(&mut &sealed_data[..]).expect("Failed to decode sealed master key");

    #[allow(clippy::infallible_destructuring_match)]
    let secrets = match versioned_data {
        MasterKeySeal::V1(data) => {
            assert!(
                identity_key.verify_data(&data.signature, &data.secret),
                "Broken sealed master key"
            );
            vec![RotatedMasterKey {
                rotation_id: 0,
                block_height: 0,
                secret: data.secret,
            }]
        }
        MasterKeySeal::V2(data) => {
            let encoded = data.payload.encode();
            let wrapped = wrap_content_to_sign(&encoded, SignedContentType::MasterKeyStore);
            assert!(
                identity_key.verify_data(&data.signature, &wrapped),
                "Broken sealed master key history"
            );
            data.payload.rotations
        }
    };

    secrets
}

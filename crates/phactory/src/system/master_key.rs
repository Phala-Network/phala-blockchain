use std::path::PathBuf;
use std::vec::Vec;

use parity_scale_codec::{Decode, Encode};
use sp_core::sr25519;

use phala_crypto::sr25519::{Persistence, Signature, Signing, Sr25519SecretKey};

use crate::pal::Sealing;

/// Master key filepath
pub const MASTER_KEY_FILE: &str = "master_key.seal";

#[derive(Debug, Encode, Decode, Clone)]
struct PersistentMasterKey {
    secret: Sr25519SecretKey,
    signature: Signature,
}

#[derive(Debug, Encode, Decode, Clone)]
struct MasterKeyHistory {
    secrets: Vec<Sr25519SecretKey>,
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

fn master_key_file_path(sealing_path: String) -> PathBuf {
    PathBuf::from(&sealing_path).join(MASTER_KEY_FILE)
}

/// Seal master key seeds with signature to ensure integrity
pub fn seal(
    sealing_path: String,
    master_key_history: Vec<&sr25519::Pair>,
    identity_key: &sr25519::Pair,
    sys: &impl Sealing,
) {
    let secrets: Vec<Sr25519SecretKey> = master_key_history
        .into_iter()
        .map(|master_key| master_key.dump_secret_key())
        .collect();
    let payload = MasterKeyHistory { secrets };
    let signature = identity_key.sign_data(&payload.encode());

    let data = MasterKeySeal::V2(PersistentMasterKeyHistory { payload, signature });
    let filepath = master_key_file_path(sealing_path);
    info!("Seal master key to {}", filepath.as_path().display());
    sys.seal_data(filepath, &data.encode())
        .expect("Seal master key failed");
}

/// Unseal local master key seeds and verify signature
///
/// This function could panic a lot.
pub fn try_unseal(
    sealing_path: String,
    identity_key: &sr25519::Pair,
    sys: &impl Sealing,
) -> Vec<sr25519::Pair> {
    let filepath = master_key_file_path(sealing_path);
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
            vec![data.secret]
        }
        MasterKeySeal::V2(data) => {
            assert!(
                identity_key.verify_data(&data.signature, &data.payload.encode()),
                "Broken sealed master key history"
            );
            data.payload.secrets
        }
    };

    secrets
        .into_iter()
        .map(|secret| sr25519::Pair::restore_from_secret_key(&secret))
        .collect()
}

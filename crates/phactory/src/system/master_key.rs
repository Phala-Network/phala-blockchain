use std::path::PathBuf;

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

#[derive(Debug, Encode, Decode)]
enum MasterKeySeal {
    V1(PersistentMasterKey),
}

fn master_key_file_path(sealing_path: String) -> PathBuf {
    PathBuf::from(&sealing_path).join(MASTER_KEY_FILE)
}

/// Seal master key seed with signature to ensure integrity
pub fn seal(
    sealing_path: String,
    master_key: &sr25519::Pair,
    identity_key: &sr25519::Pair,
    sys: &impl Sealing,
) {
    let secret = master_key.dump_secret_key();
    let signature = identity_key.sign_data(&secret);

    let data = MasterKeySeal::V1(PersistentMasterKey { secret, signature });
    let filepath = master_key_file_path(sealing_path);
    info!("Seal master key to {}", filepath.as_path().display());
    sys.seal_data(filepath, &data.encode())
        .expect("Seal master key failed");
}

/// Unseal local master key seed and verify signature
///
/// This function could panic a lot.
pub fn try_unseal(
    sealing_path: String,
    identity_key: &sr25519::Pair,
    sys: &impl Sealing,
) -> Option<sr25519::Pair> {
    let filepath = master_key_file_path(sealing_path);
    info!("Unseal master key from {}", filepath.as_path().display());
    let sealed_data = match sys
        .unseal_data(&filepath)
        .expect("Unseal master key failed")
    {
        Some(data) => data,
        None => {
            warn!("No sealed master key");
            return None;
        }
    };

    let versioned_data =
        MasterKeySeal::decode(&mut &sealed_data[..]).expect("Failed to decode sealed master key");

    let data = match versioned_data {
        MasterKeySeal::V1(data) => data,
    };

    assert!(
        identity_key.verify_data(&data.signature, &data.secret),
        "Broken sealed master key"
    );

    Some(sr25519::Pair::restore_from_secret_key(&data.secret))
}

use crate::std::convert::TryInto;
use crate::std::path::PathBuf;
use crate::std::prelude::v1::*;
use crate::std::sgxfs::{read as sgx_read, write as sgx_write};

use phala_crypto::sr25519::{Persistence, Signature, Signing, Sr25519SecretKey};
use sp_core::sr25519;

use parity_scale_codec::{Encode, Decode};

/// Master key filepath
pub const MASTER_KEY_FILE: &str = "master_key.seal";

#[derive(Debug, Encode, Decode, Clone)]
struct PersistentMasterKey {
    secret_vec: Vec<u8>,
    signature_vec: Vec<u8>,
}

#[derive(Debug, Encode, Decode)]
enum MasterKeySeal {
    V1(PersistentMasterKey)
}

fn master_key_file_path(sealing_path: String) -> PathBuf {
    PathBuf::from(&sealing_path).join(MASTER_KEY_FILE)
}

/// Seal master key seed with signature to ensure integrity
pub fn seal(sealing_path: String, master_key: &sr25519::Pair, identity_key: &sr25519::Pair) {
    let secret = master_key.dump_secret_key();
    let signature_vec = identity_key.sign_data(&secret).0.to_vec();
    let secret_vec = secret.to_vec();

    let data = MasterKeySeal::V1(PersistentMasterKey {
        secret_vec,
        signature_vec,
    });
    let filepath = master_key_file_path(sealing_path);
    info!("Seal master key to {}", filepath.as_path().display());
    sgx_write(&filepath, &data.encode())
        .unwrap_or_else(|e| panic!("Seal master key failed: {:?}", e));
}

/// Unseal local master key seed and verify signature
///
/// This function could panic a lot.
pub fn try_unseal(sealing_path: String, identity_key: &sr25519::Pair) -> Option<sr25519::Pair> {
    let filepath = master_key_file_path(sealing_path);
    info!("Unseal master key from {}", filepath.as_path().display());
    let sealed_data = match sgx_read(&filepath) {
        Ok(data) => data,
        Err(e) => {
            warn!("Failed to unseal saved master key: {:?}", e);
            return None;
        }
    };

    let versioned_data = MasterKeySeal::decode(&mut &sealed_data[..])
        .unwrap_or_else(|e| panic!("Failed to unseal saved master key: {:?}", e));

    let data = match versioned_data {
        MasterKeySeal::V1(data) => data,
    };

    let secret: Sr25519SecretKey = data
        .secret_vec
        .try_into()
        .unwrap_or_else(|e| panic!("Unseal master key failed: {:?}", e));
    let signature: [u8; 64] = data
        .signature_vec
        .try_into()
        .unwrap_or_else(|e| panic!("Unseal signature failed: {:?}", e));
    let signature = Signature::from_raw(signature);

    assert!(
        identity_key.verify_data(&signature, &secret),
        "Broken sealed master key"
    );

    Some(sr25519::Pair::restore_from_secret_key(&secret))
}

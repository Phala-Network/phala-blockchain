use crate::std::io::{Read, Write};
use crate::std::path::PathBuf;
use crate::std::prelude::v1::*;
use crate::std::sgxfs::SgxFile;

use phala_crypto::sr25519::{Persistence, Signing, SECRET_KEY_LENGTH, SIGNATURE_BYTES};
use sp_core::sr25519;

/// Master key filepath
pub const MASTER_KEY_FILE: &str = "master_key.seal";

fn master_key_file_path(sealing_path: String) -> PathBuf {
    PathBuf::from(&sealing_path).join(MASTER_KEY_FILE)
}

/// Seal master key seed with signature to ensure integrity
pub fn seal(sealing_path: String, master_key: &sr25519::Pair, identity_key: &sr25519::Pair) {
    let secret = master_key.dump_secret_key();
    let sig = identity_key.sign_data(&secret);

    // TODO(shelven): use serialization rather than manual concat.
    let mut buf = Vec::new();
    buf.extend_from_slice(&secret);
    buf.extend_from_slice(sig.as_ref());

    let filepath = master_key_file_path(sealing_path);
    info!("Seal master key to {}", filepath.as_path().display());
    let mut file = SgxFile::create(filepath)
        .unwrap_or_else(|e| panic!("Create master key file failed: {:?}", e));
    file.write_all(&buf)
        .unwrap_or_else(|e| panic!("Seal master key failed: {:?}", e));
}

/// Unseal local master key seed and verify signature
///
/// This function could panic a lot.
pub fn try_unseal(sealing_path: String, identity_key: &sr25519::Pair) -> Option<sr25519::Pair> {
    let filepath = master_key_file_path(sealing_path);
    info!("Unseal master key from {}", filepath.as_path().display());
    let mut file = match SgxFile::open(filepath) {
        Ok(file) => file,
        Err(e) => {
            warn!("Failed to unseal saved master key: {:?}", e);
            return None;
        }
    };

    let mut secret = [0_u8; SECRET_KEY_LENGTH];
    let mut sig = [0_u8; SIGNATURE_BYTES];

    let n = file
        .read(secret.as_mut())
        .unwrap_or_else(|e| panic!("Read master key failed: {:?}", e));
    if n < SECRET_KEY_LENGTH {
        panic!(
            "Unexpected sealed secret key length {}, expected {}",
            n, SECRET_KEY_LENGTH
        );
    }

    let n = file
        .read(sig.as_mut())
        .unwrap_or_else(|e| panic!("Read master key sig failed: {:?}", e));
    if n < SIGNATURE_BYTES {
        panic!(
            "Unexpected sealed seed sig length {}, expected {}",
            n, SIGNATURE_BYTES
        );
    }

    assert!(
        identity_key.verify_data(&phala_crypto::sr25519::Signature::from_raw(sig), &secret),
        "Broken sealed master key"
    );

    Some(sr25519::Pair::restore_from_secret_key(&secret))
}

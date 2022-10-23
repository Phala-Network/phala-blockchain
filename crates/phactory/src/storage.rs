use crate::light_validation::{storage_proof::StorageProof, LightValidation};
use phactory_api::storage_sync::{BlockValidator, Error as SyncError, Result};
use std::string::ToString;

pub use storage_ext::{Storage, StorageExt};

impl BlockValidator for LightValidation<chain::Runtime> {
    fn submit_finalized_headers(
        &mut self,
        bridge_id: u64,
        header: chain::Header,
        ancestry_proof: Vec<chain::Header>,
        grandpa_proof: Vec<u8>,
        authority_set_change: Option<phactory_api::blocks::AuthoritySetChange>,
    ) -> Result<()> {
        self.submit_finalized_headers(
            bridge_id,
            header,
            ancestry_proof,
            grandpa_proof,
            authority_set_change,
        )
        .map_err(|e| SyncError::HeaderValidateFailed(e.to_string()))
    }

    fn validate_storage_proof(
        &self,
        state_root: chain::Hash,
        proof: StorageProof,
        items: &[(&[u8], &[u8])],
    ) -> Result<()> {
        self.validate_storage_proof(state_root, proof, items)
            .map_err(|e| SyncError::StorageProofFailed(e.to_string()))
    }
}

mod storage_ext {
    use crate::chain;
    use crate::light_validation::utils::storage_prefix;
    use phactory_api::blocks::ParaId;
    use log::error;
    use parity_scale_codec::{Decode, Error};
    use phala_mq::Message;
    use phala_trie_storage::TrieStorage;

    pub type Storage = TrieStorage<crate::RuntimeHasher>;

    pub trait StorageExt {
        fn get_raw(&self, key: impl AsRef<[u8]>) -> Option<Vec<u8>>;
        fn get_decoded_result<T: Decode>(&self, key: impl AsRef<[u8]>) -> Result<Option<T>, Error> {
            self.get_raw(key)
                .map(|v| match Decode::decode(&mut &v[..]) {
                    Ok(decoded) => Ok(decoded),
                    Err(e) => {
                        error!("Decode storage value failed: {}", e);
                        Err(e)
                    }
                })
                .transpose()
        }
        fn get_decoded<T: Decode>(&self, key: impl AsRef<[u8]>) -> Option<T> {
            self.get_decoded_result(key).ok().flatten()
        }
        fn get_decoded_or_default<T: Decode + Default>(
            &self,
            key: impl AsRef<[u8]>,
        ) -> Result<T, Error> {
            self.get_decoded_result(key).map(|v| v.unwrap_or_default())
        }
        fn para_id(&self) -> Option<ParaId> {
            self.get_decoded(storage_prefix("ParachainInfo", "ParachainId"))
        }
        fn mq_messages(&self) -> Result<Vec<Message>, Error> {
            for key in ["OutboundMessagesV2", "OutboundMessages"] {
                let messages: Vec<Message> =
                    self.get_decoded_or_default(storage_prefix("PhalaMq", key))?;
                if !messages.is_empty() {
                    info!("Got {} messages from {key}", messages.len());
                    return Ok(messages);
                }
            }
            Ok(vec![])
        }
        fn timestamp_now(&self) -> Option<chain::Moment> {
            self.get_decoded(storage_prefix("Timestamp", "Now"))
        }
        fn pink_system_code(&self) -> Option<(u16, Vec<u8>)> {
            self.get_decoded(storage_prefix("PhalaFatContracts", "PinkSystemCode"))
        }
    }

    impl StorageExt for Storage {
        fn get_raw(&self, key: impl AsRef<[u8]>) -> Option<Vec<u8>> {
            self.get(key)
        }
    }
}

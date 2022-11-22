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
    use crate::{
        chain,
        light_validation::utils::{storage_map_prefix_twox_64_concat, storage_prefix},
    };
    use log::error;
    use parity_scale_codec::{Decode, Encode, Error};
    use phactory_api::blocks::ParaId;
    use phala_mq::Message;
    use phala_trie_storage::TrieStorage;

    pub type Storage = TrieStorage<crate::RuntimeHasher>;

    // Hide the trait StorageGet behind this private mod sealed
    mod sealed {
        use super::*;
        pub trait StorageGet {
            fn get_raw(&self, key: impl AsRef<[u8]>) -> Option<Vec<u8>>;
            fn get_decoded_result<T: Decode>(
                &self,
                key: impl AsRef<[u8]>,
            ) -> Result<Option<T>, Error> {
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
        }

        impl StorageGet for Storage {
            fn get_raw(&self, key: impl AsRef<[u8]>) -> Option<Vec<u8>> {
                self.get(key)
            }
        }
    }

    pub trait StorageExt: sealed::StorageGet {
        fn para_id(&self) -> Option<ParaId> {
            self.get_decoded(storage_prefix("ParachainInfo", "ParachainId"))
        }

        fn mq_messages(&self) -> Result<Vec<Message>, Error> {
            for key in ["OutboundMessagesV2", "OutboundMessages"] {
                let messages: Vec<Message> = self
                    .get_decoded_result(storage_prefix("PhalaMq", key))
                    .map(|v| v.unwrap_or_default())?;
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

        /// Get the next mq sequnce number for given sender. Default to 0 if no message sent.
        fn mq_sequence(&self, sender: &impl Encode) -> u64 {
            use phala_pallets::pallet_mq::StorageMapTrait as _;
            type OffchainIngress = phala_pallets::pallet_mq::OffchainIngress<chain::Runtime>;

            let module_prefix = OffchainIngress::module_prefix();
            let storage_prefix = OffchainIngress::storage_prefix();
            let key = storage_map_prefix_twox_64_concat(module_prefix, storage_prefix, sender);
            self.get_decoded(key).unwrap_or(0)
        }

        /// Return `None` if given pruntime hash is not allowed on-chain
        fn get_pruntime_added_at(&self, runtime_hash: &[u8]) -> Option<chain::BlockNumber> {
            let key = storage_map_prefix_twox_64_concat(
                b"PhalaRegistry",
                b"PRuntimeAddedAt",
                runtime_hash,
            );
            self.get_decoded(key)
        }

        fn gatekeepers(&self) -> Vec<phala_types::WorkerPublicKey> {
            let key = storage_prefix("PhalaRegistry", "Gatekeeper");
            self.get_decoded(key).unwrap_or_default()
        }
    }

    impl StorageExt for Storage {}
}

use super::storage_proof::StorageProof;
use parity_scale_codec::{Decode, Encode};
use sp_finality_grandpa::{AuthorityList, SetId};

#[derive(Encode, Decode, Clone, PartialEq)]
pub struct AuthoritySet {
    pub authority_set: AuthorityList,
    pub set_id: SetId,
}

#[derive(Encode, Decode, Clone, PartialEq)]
pub struct AuthoritySetChange {
    pub authority_set: AuthoritySet,
    pub authority_proof: StorageProof,
}

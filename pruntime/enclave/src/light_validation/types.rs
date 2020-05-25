use parity_scale_codec::{Encode, Decode};
use sp_finality_grandpa::{AuthorityList, SetId};
use super::storage_proof::StorageProof;

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

pub mod pink;
pub use support::*;
mod support;
pub use phala_types::contract::*;

pub fn contract_address_to_id(address: &chain::AccountId) -> ContractId {
    let inner: &[u8; 32] = address.as_ref();
    inner.into()
}

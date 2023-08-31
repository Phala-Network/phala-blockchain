use im::OrdMap as BTreeMap;
use pink::types::AccountId;
use serde::{Deserialize, Serialize};
use sidevm::service::Spawner;

use crate::{contracts::Contract, im_helpers::ordmap_for_each_mut};

type ContractMap = BTreeMap<AccountId, Contract>;

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct ContractsKeeper {
    contracts: ContractMap,
    #[serde(skip)]
    pub(crate) weight_changed: bool,
}

impl ContractsKeeper {
    pub fn insert(&mut self, contract: Contract) {
        self.contracts.insert(contract.address().clone(), contract);
    }

    pub fn keys(&self) -> impl Iterator<Item = &AccountId> {
        self.contracts.keys()
    }

    pub fn get_mut(&mut self, id: &AccountId) -> Option<&mut Contract> {
        self.contracts.get_mut(id)
    }

    pub fn get(&self, id: &AccountId) -> Option<&Contract> {
        self.contracts.get(id)
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.contracts.len()
    }

    pub fn try_restart_sidevms(&mut self, spawner: &Spawner) {
        ordmap_for_each_mut(&mut self.contracts, |(_k, contract)| {
            if let Err(err) = contract.restart_sidevm_if_needed(spawner) {
                error!("Failed to restart sidevm instance: {:?}", err);
            }
        });
    }

    pub fn drain(&mut self) -> impl Iterator<Item = Contract> {
        std::mem::take(&mut self.contracts)
            .into_iter()
            .map(|(_, v)| v)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&AccountId, &Contract)> {
        self.contracts.iter()
    }

    pub fn apply_local_cache_quotas(&self) {
        ::pink::local_cache::apply_quotas(calc_cache_quotas(&self.contracts));
    }
}

const TOTAL_MEMORY: u64 = 1024 * 1024 * 20;
pub(super) trait ToWeight {
    fn to_weight(&self) -> u32;
}

impl ToWeight for Contract {
    fn to_weight(&self) -> u32 {
        self.weight
    }
}

pub(super) fn calc_cache_quotas<K: AsRef<[u8]> + Ord, C: ToWeight>(
    contracts: &BTreeMap<K, C>,
) -> impl Iterator<Item = (&[u8], usize)> {
    let total_weight = contracts
        .values()
        .map(|c| c.to_weight() as u64)
        .sum::<u64>()
        .max(1);
    contracts.iter().map(move |(id, contract)| {
        let contract_quota = (TOTAL_MEMORY * contract.to_weight() as u64) / total_weight;
        (id.as_ref(), contract_quota as usize)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    impl ToWeight for u32 {
        fn to_weight(&self) -> u32 {
            *self
        }
    }

    #[test]
    fn zero_quotas_works() {
        let mut contracts = BTreeMap::new();
        contracts.insert(b"foo", 0_u32);
        contracts.insert(b"bar", 0_u32);

        let quotas: Vec<_> = calc_cache_quotas(&contracts).collect();
        assert_eq!(quotas, sorted(vec![(&b"foo"[..], 0), (b"bar", 0)]));
    }

    #[test]
    fn little_quotas_works() {
        let mut contracts = BTreeMap::new();
        contracts.insert(b"foo", 0_u32);
        contracts.insert(b"bar", 1_u32);

        let quotas: Vec<_> = calc_cache_quotas(&contracts).collect();
        assert_eq!(
            quotas,
            sorted(vec![(&b"foo"[..], 0), (b"bar", TOTAL_MEMORY as usize),])
        );
    }

    #[test]
    fn it_wont_overflow() {
        let mut contracts = BTreeMap::new();
        contracts.insert(b"foo", 0_u32);
        contracts.insert(b"bar", u32::MAX);
        contracts.insert(b"baz", u32::MAX);

        let quotas: Vec<_> = calc_cache_quotas(&contracts).collect();
        assert_eq!(
            quotas,
            sorted(vec![
                (&b"foo"[..], 0),
                (b"bar", TOTAL_MEMORY as usize / 2),
                (b"baz", TOTAL_MEMORY as usize / 2),
            ])
        );
    }

    #[test]
    fn fraction_works() {
        let mut contracts = BTreeMap::new();
        contracts.insert(b"foo", 0_u32);
        contracts.insert(b"bar", 1);
        contracts.insert(b"baz", u32::MAX);

        let quotas: Vec<_> = calc_cache_quotas(&contracts).collect();
        assert_eq!(
            quotas,
            sorted(vec![
                (&b"foo"[..], 0),
                (b"bar", 0),
                (b"baz", TOTAL_MEMORY as usize - 1),
            ])
        );
    }

    fn sorted<T: Ord>(mut v: Vec<T>) -> Vec<T> {
        v.sort();
        v
    }
}

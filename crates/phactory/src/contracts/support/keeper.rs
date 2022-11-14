use pink::runtime::ExecSideEffects;
use serde::{Deserialize, Serialize};
use sidevm::service::Spawner;
use std::collections::BTreeMap;

use crate::{
    contracts::{pink::Pink, FatContract, TransactionContext},
    system::{TransactionError, TransactionResult},
    types::{deopaque_query, OpaqueError, OpaqueQuery, OpaqueReply},
};
use parity_scale_codec::{Decode, Encode};
use phala_mq::{ContractId, MessageOrigin};

use super::QueryContext;

type ContractMap = BTreeMap<ContractId, FatContract>;

macro_rules! define_any_native_contract {
    (pub enum $name:ident { $($contract:ident ($contract_type: tt),)* }) => {
        #[derive(Encode, Decode)]
        pub enum $name {
            $($contract($contract_type),)*
        }

        impl $name {
            pub(crate) fn handle_command(
                &mut self,
                origin: MessageOrigin,
                cmd: Vec<u8>,
                context: &mut TransactionContext,
            ) -> TransactionResult {
                match self {
                    $(Self::$contract(me) => {
                        let cmd = Decode::decode(&mut &cmd[..]).or(Err(TransactionError::BadInput))?;
                        me.handle_command(origin, cmd, context)
                    })*
                }
            }

            pub(crate) fn on_block_end(&mut self, context: &mut TransactionContext) -> TransactionResult {
                match self {
                    $(Self::$contract(me) => {
                        me.on_block_end(context)
                    })*
                }
            }

            pub(crate) fn snapshot(&self) -> Self {
                match self {
                    $($name::$contract(me) => {
                        Self::$contract(me.snapshot())
                    })*
                }
            }

            pub(crate) async fn handle_query(
                &self,
                origin: Option<&runtime::AccountId>,
                req: OpaqueQuery,
                context: &mut QueryContext,
            ) -> Result<(OpaqueReply, ExecSideEffects), OpaqueError> {
                match self {
                    $($name::$contract(me) => {
                        let mut effects = ExecSideEffects::default();
                        let response = me.handle_query(origin, deopaque_query(&req)?, context, &mut effects).await;
                        if let Err(err) = &response {
                            warn!("Error handling query: {:?}", err);
                        }
                        Ok((response.encode(), effects))
                    })*
                }
            }
        }

        $(
            impl From<$contract_type> for $name {
                fn from(c: $contract_type) -> Self {
                    $name::$contract(c)
                }
            }
        )*
    };
}

define_any_native_contract!(
    pub enum AnyContract {
        Pink(Pink),
    }
);

#[derive(Default, Serialize, Deserialize)]
pub struct ContractsKeeper {
    contracts: ContractMap,
    #[serde(skip)]
    pub(crate) weight_changed: bool,
}

impl ContractsKeeper {
    pub fn insert(&mut self, contract: FatContract) {
        self.contracts.insert(contract.id(), contract);
    }

    pub fn keys(&self) -> impl Iterator<Item = &ContractId> {
        self.contracts.keys()
    }

    pub fn get_mut(&mut self, id: &ContractId) -> Option<&mut FatContract> {
        self.contracts.get_mut(id)
    }

    pub fn get(&self, id: &ContractId) -> Option<&FatContract> {
        self.contracts.get(id)
    }

    pub fn len(&self) -> usize {
        self.contracts.len()
    }

    pub fn try_restart_sidevms(&mut self, spawner: &Spawner) {
        for contract in self.contracts.values_mut() {
            if let Err(err) = contract.restart_sidevm_if_needed(spawner) {
                error!("Failed to restart sidevm instance: {:?}", err);
            }
        }
    }

    pub fn remove(&mut self, id: &ContractId) -> Option<FatContract> {
        self.contracts.remove(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ContractId, &FatContract)> {
        self.contracts.iter()
    }

    pub fn apply_local_cache_quotas(&self) {
        ::pink::local_cache::apply_quotas(quotas::calc_cache_quotas(&self.contracts));
    }
}

mod quotas {
    use super::*;

    const TOTAL_MEMORY: u64 = 1024 * 1024 * 20;

    pub(super) trait ToWeight {
        fn to_weight(&self) -> u32;
    }

    impl ToWeight for FatContract {
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

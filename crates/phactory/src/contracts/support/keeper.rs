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
        const TOTAL_MEMORY: u64 = 1024 * 1024 * 20;
        let total_weight = self
            .contracts
            .values()
            .map(|c| c.weight as u64)
            .sum::<u64>()
            .max(1);
        let quotas = self.iter().map(|(id, contract)| {
            let contract_quota = (TOTAL_MEMORY * contract.weight as u64) / total_weight;
            (id.as_bytes(), contract_quota as usize)
        });
        ::pink::local_cache::apply_quotas(quotas);
    }
}

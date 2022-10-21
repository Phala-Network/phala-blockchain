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
pub struct ContractsKeeper(ContractMap);

impl ContractsKeeper {
    pub fn insert(&mut self, contract: FatContract) {
        self.0.insert(contract.id(), contract);
    }

    pub fn keys(&self) -> impl Iterator<Item = &ContractId> {
        self.0.keys()
    }

    pub fn get_mut(&mut self, id: &ContractId) -> Option<&mut FatContract> {
        self.0.get_mut(id)
    }

    pub fn get(&self, id: &ContractId) -> Option<&FatContract> {
        self.0.get(id)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn try_restart_sidevms(&mut self, spawner: &Spawner) {
        for contract in self.0.values_mut() {
            if let Err(err) = contract.restart_sidevm_if_needed(spawner) {
                error!("Failed to restart sidevm instance: {:?}", err);
            }
        }
    }

    pub fn remove(&mut self, id: &ContractId) -> Option<FatContract> {
        self.0.remove(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ContractId, &FatContract)> {
        self.0.iter()
    }
}

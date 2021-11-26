use phala_mq::ContractId;
use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};

type ContractMap = BTreeMap<ContractId, Box<dyn super::Contract + Send>>;

#[derive(Default)]
pub struct ContractsKeeper(ContractMap);

impl Deref for ContractsKeeper {
    type Target = ContractMap;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ContractsKeeper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

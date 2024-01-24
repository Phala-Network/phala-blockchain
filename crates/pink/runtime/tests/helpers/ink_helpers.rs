use anyhow::{anyhow, Error, Result};
use ink::{
    env::{
        call::{
            utils::{ReturnType, Set, Unset},
            Call, CallBuilder, CreateBuilder, ExecutionInput, FromAccountId,
        },
        Environment,
    },
    primitives::Hash,
    MessageResult,
};
use pink::Balance;
use pink_capi::{
    types::{ExecSideEffects, ExecutionMode},
    v1::ecall::{ECalls, TransactionArguments},
};
use scale::{Decode, Encode};
use sp_runtime::AccountId32 as AccountId;

use super::{
    test_cluster::{ContractExecResult, ContractInstantiateResult},
    TestCluster,
};

const DEFAULT_QUERY_GAS_LIMIT: u64 = 50_000_000_000_000;
const DEFAULT_TX_GAS_LIMIT: u64 = 2_500_000_000_000;

pub trait SessionExt {
    fn actor(&mut self) -> AccountId;
    fn set_driver<A: Encode>(&mut self, name: &str, contract: &A) -> Result<()>;
}

impl SessionExt for TestCluster {
    fn actor(&mut self) -> AccountId {
        super::ALICE.clone()
    }
    fn set_driver<A: Encode>(&mut self, name: &str, contract: &A) -> Result<()> {
        let origin = self.actor();
        let system_contract = self
            .query()
            .system_contract()
            .expect("System contract not found");
        let selector = 0xaa1e2030u32.to_be_bytes();
        let input_data = (selector, name, contract).encode();
        let mode = ExecutionMode::Transaction;
        self.tx().contract_call(
            system_contract,
            input_data,
            mode,
            TransactionArguments {
                origin,
                transfer: 0,
                gas_limit: DEFAULT_TX_GAS_LIMIT,
                gas_free: true,
                storage_deposit_limit: None,
                deposit: 0,
            },
        );
        Ok(())
    }
}

pub trait BareDeployWasm {
    fn bare_deploy(
        self,
        wasm: &[u8],
        session: &mut TestCluster,
    ) -> Result<ContractInstantiateResult>;
}
pub trait DeployBundle: BareDeployWasm {
    type Contract;
    fn deploy_wasm(self, wasm: &[u8], session: &mut TestCluster) -> Result<Self::Contract>
    where
        Self: Sized;
}

pub trait BareDeployable {
    fn bare_deploy(self, session: &mut TestCluster) -> ContractInstantiateResult;
}

pub trait Deployable: BareDeployable {
    type Contract;
    fn deploy(self, session: &mut TestCluster) -> Result<Self::Contract>;
}

pub trait Callable {
    type Ret;
    fn submit_tx(self, session: &mut TestCluster) -> Result<Self::Ret>;
    fn bare_tx(self, session: &mut TestCluster) -> ContractExecResult;
    fn query(self, session: &mut TestCluster) -> Result<Self::Ret>;
    fn bare_query(self, session: &mut TestCluster) -> ContractExecResult;
}

impl<Env, Contract, Ret, Args, Salt> BareDeployWasm
    for CreateBuilder<
        Env,
        Contract,
        Unset<Hash>,
        Unset<u64>,
        Unset<Balance>,
        Set<ExecutionInput<Args>>,
        Set<Salt>,
        Set<ReturnType<Ret>>,
    >
where
    Env: Environment<Hash = Hash, Balance = Balance>,
    Args: Encode,
    Salt: AsRef<[u8]>,
{
    fn bare_deploy(
        self,
        wasm: &[u8],
        session: &mut TestCluster,
    ) -> Result<ContractInstantiateResult> {
        let caller = session.actor();
        let code_hash = session
            .tx()
            .upload_code(caller.clone(), wasm.to_vec(), true)
            .map_err(Error::msg)?;
        Ok(self.code_hash(code_hash.0.into()).bare_deploy(session))
    }
}

impl<Env, Contract, Args, Salt> DeployBundle
    for CreateBuilder<
        Env,
        Contract,
        Unset<Hash>,
        Unset<u64>,
        Unset<Balance>,
        Set<ExecutionInput<Args>>,
        Set<Salt>,
        Set<ReturnType<Contract>>,
    >
where
    Env: Environment<Hash = Hash, Balance = Balance>,
    Contract: FromAccountId<Env>,
    Args: Encode,
    Salt: AsRef<[u8]>,
{
    type Contract = Contract;

    fn deploy_wasm(self, wasm: &[u8], session: &mut TestCluster) -> Result<Self::Contract> {
        into_contract(self.bare_deploy(wasm, session)?)
    }
}

impl<Env, Contract, Ret, Args> BareDeployWasm
    for CreateBuilder<
        Env,
        Contract,
        Unset<Hash>,
        Unset<u64>,
        Unset<Balance>,
        Set<ExecutionInput<Args>>,
        Unset<ink::env::call::state::Salt>,
        Set<ReturnType<Ret>>,
    >
where
    Env: Environment<Hash = Hash, Balance = Balance>,
    Args: Encode,
{
    fn bare_deploy(
        self,
        wasm: &[u8],
        session: &mut TestCluster,
    ) -> Result<ContractInstantiateResult> {
        self.salt_bytes(Vec::new()).bare_deploy(wasm, session)
    }
}

impl<Env, Contract, Args> DeployBundle
    for CreateBuilder<
        Env,
        Contract,
        Unset<Hash>,
        Unset<u64>,
        Unset<Balance>,
        Set<ExecutionInput<Args>>,
        Unset<ink::env::call::state::Salt>,
        Set<ReturnType<Contract>>,
    >
where
    Env: Environment<Hash = Hash, Balance = Balance>,
    Contract: FromAccountId<Env>,
    Args: Encode,
{
    type Contract = Contract;
    fn deploy_wasm(self, wasm: &[u8], session: &mut TestCluster) -> Result<Self::Contract> {
        self.salt_bytes(Vec::new()).deploy_wasm(wasm, session)
    }
}

impl<Env, Contract, Ret, Args, Salt> BareDeployable
    for CreateBuilder<
        Env,
        Contract,
        Set<Hash>,
        Unset<u64>,
        Unset<Balance>,
        Set<ExecutionInput<Args>>,
        Set<Salt>,
        Set<ReturnType<Ret>>,
    >
where
    Env: Environment<Hash = Hash, Balance = Balance>,
    Args: Encode,
    Salt: AsRef<[u8]>,
{
    fn bare_deploy(self, session: &mut TestCluster) -> ContractInstantiateResult {
        let origin = session.actor();
        let constructor = self.endowment(0).gas_limit(DEFAULT_TX_GAS_LIMIT);
        let params = constructor.params();
        let code_hash: &[u8] = params.code_hash().as_ref();
        let code_hash = sp_core::H256(code_hash.try_into().expect("Hash convert failed"));
        let input_data = params.exec_input().encode();
        let salt = params.salt_bytes().as_ref().to_vec();
        let exec = session.tx();
        let mode = exec.mode();
        let transfer = *params.endowment();
        let gas_limit = params.gas_limit();

        let result = session.tx().contract_instantiate(
            code_hash,
            input_data,
            salt,
            mode,
            TransactionArguments {
                origin,
                transfer,
                gas_limit,
                gas_free: false,
                storage_deposit_limit: None,
                deposit: 0,
            },
        );
        ContractInstantiateResult::decode(&mut &result[..])
            .expect("Failed to decode contract instantiate result")
    }
}

impl<Env, Contract, Args, Salt> Deployable
    for CreateBuilder<
        Env,
        Contract,
        Set<Hash>,
        Unset<u64>,
        Unset<Balance>,
        Set<ExecutionInput<Args>>,
        Set<Salt>,
        Set<ReturnType<Contract>>,
    >
where
    Env: Environment<Hash = Hash, Balance = Balance>,
    Contract: FromAccountId<Env>,
    Args: Encode,
    Salt: AsRef<[u8]>,
{
    type Contract = Contract;

    fn deploy(self, session: &mut TestCluster) -> Result<Self::Contract> {
        into_contract(self.bare_deploy(session))
    }
}

fn into_contract<Contract, Env>(result: ContractInstantiateResult) -> Result<Contract>
where
    Contract: FromAccountId<Env>,
    Env: Environment,
{
    let result = result.result.map_err(|err| anyhow::anyhow!("{err:?}"))?;
    if result.result.did_revert() {
        anyhow::bail!("Contract instantiation reverted");
    }
    let account_id =
        Decode::decode(&mut &result.account_id.encode()[..]).expect("Failed to decode account id");
    Ok(Contract::from_account_id(account_id))
}

impl<Env, Args: Encode, Ret: Decode> Callable
    for CallBuilder<Env, Set<Call<Env>>, Set<ExecutionInput<Args>>, Set<ReturnType<Ret>>>
where
    Env: Environment<Balance = Balance>,
    Ret: Decode,
    Args: Encode,
{
    type Ret = Ret;

    fn submit_tx(self, session: &mut TestCluster) -> Result<Self::Ret> {
        call(self, true, session)
    }
    fn bare_tx(self, session: &mut TestCluster) -> ContractExecResult {
        bare_call(self, false, session)
    }
    fn query(self, session: &mut TestCluster) -> Result<Self::Ret> {
        call(self, false, session)
    }
    fn bare_query(self, session: &mut TestCluster) -> ContractExecResult {
        bare_call(self, false, session)
    }
}

type CB<Env, Args, Ret> =
    CallBuilder<Env, Set<Call<Env>>, Set<ExecutionInput<Args>>, Set<ReturnType<Ret>>>;

fn call<Env, Args, Ret>(
    call_builder: CB<Env, Args, Ret>,
    deterministic: bool,
    session: &mut TestCluster,
) -> Result<Ret>
where
    Env: Environment<Balance = Balance>,
    Args: Encode,
    Ret: Decode,
{
    let result = bare_call(call_builder, deterministic, session);
    let result = result
        .result
        .map_err(|e| anyhow!("Failed to execute call: {e:?}"))?;
    let ret = MessageResult::<Ret>::decode(&mut &result.data[..])
        .map_err(|e| anyhow!("Failed to decode result: {e:?}"))?
        .map_err(|e| anyhow!("Failed to execute call: {e:?}"))?;
    Ok(ret)
}

fn bare_call<Env, Args, Ret>(
    call_builder: CB<Env, Args, Ret>,
    deterministic: bool,
    session: &mut TestCluster,
) -> ContractExecResult
where
    Env: Environment<Balance = Balance>,
    Args: Encode,
{
    let origin = session.actor();
    let params = call_builder.params();
    let data = params.exec_input().encode();
    let callee = params.callee();
    let address: [u8; 32] = callee.as_ref().try_into().expect("Invalid callee");
    let gas_limit = if params.gas_limit() > 0 {
        params.gas_limit()
    } else if deterministic {
        DEFAULT_TX_GAS_LIMIT
    } else {
        DEFAULT_QUERY_GAS_LIMIT
    };

    let mut call = if deterministic {
        session.tx()
    } else {
        session.query()
    };
    let mode = call.mode();
    let result = call.contract_call(
        address.into(),
        data,
        mode,
        TransactionArguments {
            origin,
            transfer: *params.transferred_value(),
            gas_limit,
            gas_free: true,
            storage_deposit_limit: None,
            deposit: 0,
        },
    );
    let effects = session.effects.take();
    if let Some(effects) = effects {
        let effects = if mode.is_query() {
            effects.into_query_only_effects()
        } else {
            effects
        };
        if !mode.is_estimating() && !effects.is_empty() {
            match effects {
                ExecSideEffects::V1 {
                    pink_events,
                    ink_events,
                    ..
                } => {
                    session.apply_pink_events(pink_events);
                    session.stat_ink_events(ink_events.len());
                }
            }
        }
    }
    ContractExecResult::decode(&mut &result[..]).expect("Failed to decode contract exec result")
}

mod system {
    use ink::{
        codegen::TraitCallForwarder, env::call::FromAccountId, primitives::AccountId,
        reflect::TraitDefinitionRegistry,
    };
    use pink::{system::System, ConvertTo, PinkEnvironment};
    use pink_capi::v1::ecall::ECalls;

    use crate::helpers::TestCluster;

    type TraitInfo = <TraitDefinitionRegistry<PinkEnvironment> as System>::__ink_TraitInfo;
    type SystemForwarder = <TraitInfo as TraitCallForwarder>::Forwarder;

    fn system_contract(address: AccountId) -> SystemForwarder {
        SystemForwarder::from_account_id(address)
    }

    impl TestCluster {
        pub fn system(&mut self) -> SystemForwarder {
            self.query()
                .system_contract()
                .map(|contract| system_contract(contract.convert_to()))
                .expect("System contract not found")
        }
    }
}

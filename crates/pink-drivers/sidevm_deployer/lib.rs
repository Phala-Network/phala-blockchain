#![cfg_attr(not(feature = "std"), no_std, no_main)]

extern crate alloc;

#[ink::contract(env = PinkEnvironment)]
mod sidevm_deployer {
    use alloc::collections::BTreeMap;
    use alloc::string::ToString;
    use alloc::vec::Vec;
    #[cfg(feature = "std")]
    use ink::storage::traits::StorageLayout;
    use ink::storage::Mapping;
    use pink::system::DriverError as Error;
    use pink::{PinkEnvironment, WorkerId};
    use scale::{Decode, Encode};

    type Result<T> = core::result::Result<T, Error>;

    #[ink(storage)]
    pub struct SidevmOp {
        /// Owner of the contract.
        owner: AccountId,
        /// Contracts that are allowed to deploy sidevm.
        whitelist: Mapping<AccountId, ()>,
        /// Price of sidevm instance per block per worker.
        vm_price: Balance,
        /// Price of memory per byte per block per worker.
        mem_price: Balance,
        /// Deadlines of paid instances indexed by workers.
        contract_deadline_by_workers: Mapping<WorkerId, Vec<BlockNumber>>,
        /// Paid instances indexed by contracts.
        paid_instances_by_contracts: Mapping<AccountId, ContractInstances>,
        /// Contracts that are currently running sidevm. Adding this field because the Mapping
        /// doesn't support iterating.
        contracts_running_sidevm: Vec<AccountId>,
        /// Max paid instances per worker.
        max_paid_instances_per_worker: u32,
    }

    #[derive(Encode, Decode, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Info {
        block_number: BlockNumber,
        vm_price: Balance,
        mem_price: Balance,
        max_paid_instances_per_worker: u32,
        instances: BTreeMap<AccountId, ContractInstances>,
        workers: BTreeMap<WorkerId, Vec<BlockNumber>>,
    }

    #[derive(Encode, Decode, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
    struct ContractInstances {
        workers: Vec<WorkerId>,
        deadline: BlockNumber,
        price: Balance,
    }

    impl SidevmOp {
        fn ensure_owner(&self) -> Result<()> {
            use ink::codegen::StaticEnv;
            if Self::env().caller() != self.owner {
                return Err(Error::BadOrigin);
            }
            Ok(())
        }
    }

    fn ensure_tx() -> Result<()> {
        if !pink::ext().is_in_transaction() {
            return Err(Error::Other("Transaction required".to_string()));
        }
        Ok(())
    }

    impl SidevmOp {
        #[ink(constructor)]
        #[allow(clippy::should_implement_trait)]
        pub fn default() -> Self {
            Self::new(1, 1, 5)
        }

        #[ink(constructor)]
        pub fn new(
            vm_price: Balance,
            mem_price: Balance,
            max_paid_instances_per_worker: u32,
        ) -> Self {
            Self {
                owner: Self::env().caller(),
                whitelist: Default::default(),
                vm_price,
                mem_price,
                contract_deadline_by_workers: Default::default(),
                paid_instances_by_contracts: Default::default(),
                contracts_running_sidevm: Default::default(),
                max_paid_instances_per_worker,
            }
        }

        #[ink(message)]
        pub fn owner(&self) -> AccountId {
            self.owner
        }

        #[ink(message)]
        pub fn allow(&mut self, contract: AccountId) -> Result<()> {
            self.ensure_owner()?;
            self.whitelist.insert(contract, &());
            Ok(())
        }

        #[ink(message)]
        pub fn version(&self) -> this_crate::VersionTuple {
            this_crate::version_tuple!()
        }

        #[ink(message)]
        pub fn info(&self) -> Info {
            let block_number = self.env().block_number();
            let mut workers = BTreeMap::new();
            let mut instances = BTreeMap::new();
            for contract in self.contracts_running_sidevm.iter() {
                let info = self
                    .paid_instances_by_contracts
                    .get(contract)
                    .expect("Failed to get instances");
                for worker in info.workers.iter() {
                    let deadlines = self
                        .contract_deadline_by_workers
                        .get(worker)
                        .expect("Failed to get instances");
                    workers.insert(*worker, deadlines);
                }
                instances.insert(*contract, info);
            }
            Info {
                block_number,
                vm_price: self.vm_price,
                mem_price: self.mem_price,
                max_paid_instances_per_worker: self.max_paid_instances_per_worker,
                instances,
                workers,
            }
        }

        /// For self upgrade.
        #[ink(message)]
        pub fn set_code(&mut self, code_hash: pink::Hash) -> Result<()> {
            self.ensure_owner()?;
            ink::env::set_code_hash(&code_hash).expect("Failed to set code hash");
            pink::info!("Switched code hash to {:?}.", code_hash);
            Ok(())
        }

        /// Set the price of sidevm instance per block per worker.
        #[ink(message)]
        pub fn set_vm_price(&mut self, price: Balance) -> Result<()> {
            self.ensure_owner()?;
            self.vm_price = price;
            Ok(())
        }

        /// Set the price of memory per byte per block per worker.
        #[ink(message)]
        pub fn set_mem_price(&mut self, price: Balance) -> Result<()> {
            self.ensure_owner()?;
            self.mem_price = price;
            Ok(())
        }

        /// Set max paid instances per worker
        #[ink(message)]
        pub fn set_max_paid_instances_per_worker(&mut self, n: u32) -> Result<()> {
            self.ensure_owner()?;
            self.max_paid_instances_per_worker = n;
            Ok(())
        }

        fn clear_contract(&mut self, contract: &AccountId) -> Result<()> {
            self.contracts_running_sidevm.retain(|x| x != contract);
            let instances = self
                .paid_instances_by_contracts
                .get(contract)
                .ok_or_else(|| Error::Other("No instances".to_string()))?;
            self.paid_instances_by_contracts.remove(contract);
            for worker in instances.workers.iter() {
                let mut deadlines = self
                    .contract_deadline_by_workers
                    .get(worker)
                    .ok_or_else(|| Error::Other("No instances".to_string()))?;
                if let Some(ind) = deadlines.iter().position(|&x| x == instances.deadline) {
                    deadlines.remove(ind);
                }
                if deadlines.is_empty() {
                    self.contract_deadline_by_workers.remove(worker);
                } else {
                    self.contract_deadline_by_workers.insert(worker, &deadlines);
                }
            }
            Ok(())
        }

        fn recycle(&mut self) {
            let now = self.env().block_number();
            let mut to_remove = Vec::new();
            for contract in self.contracts_running_sidevm.iter() {
                let instances = self
                    .paid_instances_by_contracts
                    .get(contract)
                    .expect("Failed to get instances");
                if instances.deadline < now {
                    to_remove.push(*contract);
                }
            }
            for contract in to_remove {
                self.clear_contract(&contract).expect("Failed to clear");
            }
        }

        fn ensure_available(&self, workers: &[WorkerId]) -> Result<()> {
            for worker in workers.iter() {
                let deadlines = self
                    .contract_deadline_by_workers
                    .get(worker)
                    .unwrap_or_default();
                if deadlines.len() >= self.max_paid_instances_per_worker as usize {
                    return Err(Error::Other("No available workers".to_string()));
                }
            }
            Ok(())
        }

        fn add_deploy_info(
            &mut self,
            contract: &AccountId,
            workers: &[WorkerId],
            price: Balance,
            deadline: u32,
        ) -> Result<()> {
            self.recycle();
            let _ = self.clear_contract(contract);
            self.ensure_available(workers)?;
            let instances = ContractInstances {
                workers: workers.to_vec(),
                deadline,
                price,
            };
            self.paid_instances_by_contracts
                .insert(contract, &instances);
            for worker in workers.iter() {
                let mut deadlines = self
                    .contract_deadline_by_workers
                    .get(worker)
                    .unwrap_or_default();
                deadlines.push(instances.deadline);
                self.contract_deadline_by_workers.insert(worker, &deadlines);
            }
            self.contracts_running_sidevm.push(*contract);
            Ok(())
        }

        fn remaining_time_to_value(&self, contract: &AccountId) -> Balance {
            let Some(info) = self.paid_instances_by_contracts.get(contract) else {
                return 0;
            };
            let current = self.env().block_number().saturating_add(1);
            if info.deadline <= current {
                return 0;
            }
            let remaining_time = info.deadline.saturating_sub(current);
            info.price.saturating_mul(remaining_time as Balance)
        }

        fn terminate(&mut self, contract: &AccountId) -> Result<()> {
            self.clear_contract(contract)?;
            pink::system::SystemRef::instance().stop_sidevm_at(contract.clone())?;
            Ok(())
        }
    }

    impl pink::system::SidevmOperation for SidevmOp {
        #[ink(message)]
        fn deploy(&self, code_hash: pink::Hash) -> Result<()> {
            let caller = self.env().caller();
            if !self.whitelist.contains(caller) {
                return Err(Error::BadOrigin);
            }
            let system = pink::system::SystemRef::instance();
            system.deploy_sidevm_to(caller, code_hash)?;
            Ok(())
        }

        #[ink(message)]
        fn can_deploy(&self, contract: AccountId) -> bool {
            self.whitelist.contains(contract)
        }

        #[ink(message, payable)]
        fn deploy_to_workers(
            &mut self,
            code_hash: pink::Hash,
            code_size: u32,
            workers: Vec<WorkerId>,
            max_memory_pages: u32,
            blocks_to_live: u32,
        ) -> Result<()> {
            ensure_tx()?;
            let mut workers = workers;
            workers.dedup();
            if workers.is_empty() {
                return Err(Error::Other("No workers".to_string()));
            }
            if max_memory_pages == 0 {
                return Err(Error::Other("No memory".to_string()));
            }
            if blocks_to_live == 0 {
                return Err(Error::Other("Live too short".to_string()));
            }
            let caller = self.env().caller();
            let paid_value = self.env().transferred_value();
            let remaining_value = self.remaining_time_to_value(&caller);
            let available_value = paid_value.saturating_add(remaining_value);
            let price = self.calc_price(code_size, max_memory_pages, workers.len() as u32)?;
            let n_workers = workers.len();
            let hex_caller = hex_fmt::HexFmt(&caller);
            pink::info!(
                "Deploying sidevm to workers: \
                            caller={hex_caller:?}, \
                            value={paid_value}, \
                            remaining={remaining_value}, \
                            code_size={code_size}, \
                            pages={max_memory_pages}, \
                            price={price}, \
                            n_workers={n_workers}, \
                            ttl={blocks_to_live}"
            );
            let required_value = price
                .checked_mul(blocks_to_live as Balance)
                .ok_or_else(|| Error::Other("Overflow".to_string()))?;
            if available_value < required_value {
                return Err(Error::Other("Not enough value".to_string()));
            }
            if available_value > required_value {
                self.env()
                    .transfer(caller, available_value.saturating_sub(required_value))
                    .expect("Failed to refund");
            }
            let code_size = code_size.min(1024 * 1024 * 16);
            let max_memory_pages = max_memory_pages.min(1024);
            let deadline = self.env().block_number().saturating_add(blocks_to_live);
            self.add_deploy_info(&caller, &workers, price, deadline)?;
            let system = pink::system::SystemRef::instance();
            let config = pink::SidevmConfig {
                max_code_size: code_size,
                max_memory_pages,
                deadline,
                ..Default::default()
            };
            system.deploy_sidevm_to_workers(caller, code_hash, workers, config)?;
            Ok(())
        }

        #[ink(message)]
        fn calc_price(
            &self,
            code_size: u32,
            max_memory_pages: u32,
            n_workers: u32,
        ) -> Result<Balance> {
            calc_price_per_block(
                self.vm_price,
                self.mem_price,
                code_size,
                max_memory_pages,
                n_workers,
            )
            .ok_or_else(|| Error::Other("Overflow".to_string()))
        }

        #[ink(message, payable)]
        fn update_deadline(&mut self, deadline: u32) -> Result<()> {
            ensure_tx()?;
            let caller = self.env().caller();
            let paid_value = self.env().transferred_value();
            let mut current = self
                .paid_instances_by_contracts
                .get(caller)
                .ok_or_else(|| Error::Other("No instances".to_string()))?;
            let now = self.env().block_number();
            let deadline = u32::max(deadline, now);
            let current_deadline = current.deadline;
            let price = current.price;
            let hex_caller = hex_fmt::HexFmt(&caller);
            pink::info!(
                "Updating deadline: \
                            caller={hex_caller:?}, \
                            value={paid_value}, \
                            now={now}, \
                            price={price}, \
                            deadline={deadline}, \
                            current_deadline={current_deadline}"
            );
            if deadline == current.deadline {
                if paid_value > 0 {
                    self.env()
                        .transfer(caller, paid_value)
                        .expect("Failed to refund");
                }
                return Ok(());
            }
            let refund = if deadline > current.deadline {
                let blocks = deadline.saturating_sub(current.deadline);
                let price = current
                    .price
                    .checked_mul(blocks as Balance)
                    .ok_or_else(|| Error::Other("Overflow".to_string()))?;
                if paid_value < price {
                    return Err(Error::Other("Not enough value".to_string()));
                }
                paid_value.saturating_sub(price)
            } else {
                let blocks = current.deadline.saturating_sub(deadline);
                let refund = current
                    .price
                    .checked_mul(blocks as Balance)
                    .ok_or_else(|| Error::Other("Overflow".to_string()))?;
                paid_value
                    .checked_add(refund)
                    .ok_or_else(|| Error::Other("Overflow".to_string()))?
            };
            if refund > 0 {
                self.env()
                    .transfer(caller, refund)
                    .expect("Failed to refund");
            }
            if deadline == now {
                let _ = self.terminate(&caller);
                return Ok(());
            }
            for worker in current.workers.iter() {
                let mut deadlines = self
                    .contract_deadline_by_workers
                    .get(worker)
                    .ok_or_else(|| Error::Other("No instances".to_string()))?;
                let ind = deadlines
                    .iter()
                    .position(|&x| x == current.deadline)
                    .ok_or_else(|| Error::Other("No deadline for contract".to_string()))?;
                deadlines.remove(ind);
                deadlines.push(deadline);
                self.contract_deadline_by_workers.insert(worker, &deadlines);
            }
            current.deadline = deadline;
            self.paid_instances_by_contracts.insert(caller, &current);
            pink::system::SystemRef::instance().set_sidevm_deadline(caller, deadline)?;
            Ok(())
        }

        #[ink(message)]
        fn deadline_of(&self, account: AccountId) -> Option<u32> {
            self.paid_instances_by_contracts
                .get(&account)
                .map(|x| x.deadline)
        }
    }

    fn calc_price_per_block(
        vm_price: Balance,
        mem_price: Balance,
        code_size: u32,
        max_memory_pages: u32,
        n_workers: u32,
    ) -> Option<Balance> {
        let code_size: Balance = code_size.try_into().ok()?;
        let max_memory_pages: Balance = max_memory_pages.try_into().ok()?;
        let n_workers: Balance = n_workers.try_into().ok()?;
        let page_size = 64 * 1024;
        let mem_size = max_memory_pages
            .checked_mul(page_size)?
            .checked_add(code_size)?;
        let mem_price = mem_price.checked_mul(mem_size)?;
        let price = vm_price.checked_add(mem_price)?;
        price.checked_mul(n_workers)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        use system::System;

        const SYSTEM_ADDR: [u8; 32] = [42u8; 32];
        const SIDEVMOP_ADDR: [u8; 32] = [24u8; 32];

        fn with_callee<T>(callee: [u8; 32], f: impl FnOnce() -> T) -> T {
            let prev = ink::env::test::callee::<PinkEnvironment>();
            ink::env::test::set_callee::<PinkEnvironment>(callee.into());
            let ret = f();
            ink::env::test::set_callee::<PinkEnvironment>(prev);
            ret
        }

        fn assume_inside<T>(contract: [u8; 32], f: impl FnOnce() -> T) -> T {
            let prev_caller = ink::env::caller::<PinkEnvironment>();
            ink::env::test::set_caller::<PinkEnvironment>(contract.into());
            let ret = with_callee(contract, f);
            ink::env::test::set_caller::<PinkEnvironment>(prev_caller);
            ret
        }

        fn set_caller(caller: [u8; 32]) {
            ink::env::test::set_caller::<PinkEnvironment>(caller.into());
        }

        fn set_value_transferred(value: Balance) {
            ink::env::test::set_value_transferred::<PinkEnvironment>(value);
        }

        fn set_balance(account: [u8; 32], value: Balance) {
            ink::env::test::set_account_balance::<PinkEnvironment>(account.into(), value);
        }

        #[ink::test]
        fn should_forbid_non_admin_contract_to_deploy_sidevm() {
            use pink::system::{Error as SystemError, SidevmOperationRef, SystemRef};

            with_callee(SYSTEM_ADDR, || {
                SystemRef::mock_with(System::default());
            });

            with_callee(SIDEVMOP_ADDR, || {
                let mut sideman = SidevmOp::default();
                sideman
                    .allow([1u8; 32].into())
                    .expect("Failed to allow contract");
                SidevmOperationRef::mock_with(sideman);
            });
            let driver = SidevmOperationRef::instance().expect("Failed to get driver instance");

            let result = driver.deploy(Default::default());
            assert_eq!(
                result,
                Err(Error::SystemError(SystemError::PermisionDenied))
            );
        }

        #[ink::test]
        fn should_forbid_contract_not_in_whitelist() {
            use pink::system::{SidevmOperationRef, System as _, SystemRef};
            with_callee(SYSTEM_ADDR, || {
                let mut system = System::default();
                system.grant_admin(SIDEVMOP_ADDR.into()).ok();
                SystemRef::mock_with(system);
            });

            with_callee(SIDEVMOP_ADDR, || {
                SidevmOperationRef::mock_with(SidevmOp::default());
            });
            let driver = SidevmOperationRef::instance().expect("Failed to get driver instance");
            let result = driver.deploy(Default::default());
            assert_eq!(result, Err(Error::BadOrigin));
        }

        #[ink::test]
        fn should_allow_contract_in_whitelist() {
            use pink::system::{SidevmOperationRef, System as _, SystemRef};

            with_callee(SYSTEM_ADDR, || {
                let mut system = System::default();
                system.grant_admin(SIDEVMOP_ADDR.into()).ok();
                SystemRef::mock_with(system);
            });

            with_callee(SIDEVMOP_ADDR, || {
                let mut sideman = SidevmOp::default();
                sideman
                    .allow([1u8; 32].into())
                    .expect("Failed to allow contract");
                SidevmOperationRef::mock_with(sideman);
            });
            let driver = SidevmOperationRef::instance().expect("Failed to get driver instance");
            let result = driver.deploy(Default::default());
            assert_eq!(result, Ok(()));
        }

        #[ink::test]
        fn paid_vm_works() {
            tracing_subscriber::fmt::init();
            pink_chain_extension::mock_ext::mock_all_ext();
            pink_chain_extension::mock_ext::set_mode(true);

            use pink::system::{SidevmOperationRef, System as _, SystemRef};

            let code_hash = Default::default();
            let code_size = 1024;
            let workers0 = vec![[1; 32], [2; 32], [3; 32]];
            let workers1 = vec![[11; 32], [12; 32], [13; 32]];
            let max_memory_pages = 1;
            let blocks_to_live = 5;
            let vm_price = 1024;
            let mem_price = 2048;
            let max_paid_instances_per_worker = 2;

            let contract0 = [50u8; 32];
            let contract1 = [51u8; 32];
            let contract2 = [52u8; 32];

            set_balance(SIDEVMOP_ADDR, 1000000000000000000);

            set_caller(contract0);

            // Setup the drivers
            with_callee(SYSTEM_ADDR, || {
                let mut system = System::default();
                system.grant_admin(SIDEVMOP_ADDR.into()).ok();
                SystemRef::mock_with(system);
            });

            let mut driver = with_callee(SIDEVMOP_ADDR, || {
                let mut driver = SidevmOp::default();
                driver
                    .allow(contract0.into())
                    .expect("Failed to allow contract");
                driver
                    .set_vm_price(vm_price)
                    .expect("Failed to set vm price");
                driver
                    .set_mem_price(mem_price)
                    .expect("Failed to set memory price");
                driver
                    .set_max_paid_instances_per_worker(max_paid_instances_per_worker)
                    .expect("Failed to set max paid instances per worker");
                driver
            });

            with_callee(SIDEVMOP_ADDR, || unsafe {
                SidevmOperationRef::unsafe_mock_with(&mut driver)
            });

            let mut driver_ref =
                SidevmOperationRef::instance().expect("Failed to get driver instance");

            let mut price = 0;
            // Should be able to deploy
            let required = assume_inside(contract0, || {
                price = driver_ref
                    .calc_price(code_size, max_memory_pages, workers0.len() as u32)
                    .unwrap();
                {
                    let page_size = 64 * 1024;
                    let expected_price = (vm_price
                        + mem_price * (max_memory_pages * page_size + code_size) as Balance)
                        * workers0.len() as Balance;
                    assert_eq!(price, expected_price);
                }
                let required = price * blocks_to_live as Balance;

                set_value_transferred(required - 1);
                let result = driver_ref.deploy_to_workers(
                    code_hash,
                    code_size,
                    workers0.clone(),
                    max_memory_pages,
                    blocks_to_live,
                );
                assert_eq!(
                    result,
                    Err(Error::Other("Not enough value".into())),
                    "Should fail if not enough value transferred"
                );

                set_value_transferred(required);
                let result = driver_ref.deploy_to_workers(
                    code_hash,
                    code_size,
                    workers0.clone(),
                    max_memory_pages,
                    blocks_to_live,
                );

                assert_eq!(result, Ok(()), "Should succeed if enough value transferred");
                let info = with_callee(SIDEVMOP_ADDR, || driver.info());
                let mut deployed_workers = info.workers.keys().cloned().collect::<Vec<_>>();
                deployed_workers.sort();
                assert_eq!(deployed_workers, workers0, "Should deploy to workers");
                let contracts = info.instances.keys().cloned().collect::<Vec<_>>();
                assert_eq!(contracts, vec![contract0.into()]);
                insta::assert_debug_snapshot!(info);

                required
            });

            // Shoulde be able to deploy to another contract
            assume_inside(contract1, || {
                set_value_transferred(required);
                let result = driver_ref.deploy_to_workers(
                    code_hash,
                    code_size,
                    workers0.clone(),
                    max_memory_pages,
                    blocks_to_live,
                );
                assert_eq!(result, Ok(()), "The second contract should deploy succeed");
                let info = with_callee(SIDEVMOP_ADDR, || driver.info());
                insta::assert_debug_snapshot!(info);
            });

            // Should no longer be able to deploy to worker group 0 using another contract
            assume_inside(contract2, || {
                set_value_transferred(required);
                let result = driver_ref.deploy_to_workers(
                    code_hash,
                    code_size,
                    workers0.clone(),
                    max_memory_pages,
                    blocks_to_live,
                );
                assert_eq!(
                    result,
                    Err(Error::Other("No available workers".into())),
                    "Should fail with no available workers"
                );
            });

            // Should be able to re-deploy to worker group 0 using contract0
            assume_inside(contract0, || {
                set_value_transferred(required);
                let result = driver_ref.deploy_to_workers(
                    code_hash,
                    code_size,
                    workers0.clone(),
                    max_memory_pages,
                    blocks_to_live,
                );
                assert_eq!(result, Ok(()), "Should succeed if enough value transferred");
            });

            // However, it should be able to deploy to worker group 1 using contract2
            assume_inside(contract2, || {
                set_value_transferred(required);
                let result = driver_ref.deploy_to_workers(
                    code_hash,
                    code_size,
                    workers1.clone(),
                    max_memory_pages,
                    blocks_to_live,
                );
                assert_eq!(result, Ok(()), "Should succeed if enough value transferred");
            });

            for _ in 0..6 {
                ink::env::test::advance_block::<PinkEnvironment>();
            }

            // Should be able to deploy to worker group 0 using contract2 after the instance of contract0 terminated
            assume_inside(contract2, || {
                set_value_transferred(required);
                let result = driver_ref.deploy_to_workers(
                    code_hash,
                    code_size,
                    workers0.clone(),
                    max_memory_pages,
                    blocks_to_live,
                );
                assert_eq!(result, Ok(()), "Should succeed if enough value transferred");
                let info = with_callee(SIDEVMOP_ADDR, || driver.info());
                insta::assert_debug_snapshot!(info);
            });

            assume_inside(contract2, || {
                set_value_transferred(0);
                let result = driver_ref.update_deadline(11);
                assert_eq!(
                    result,
                    Ok(()),
                    "Should succeed if deadline is equal to current block number"
                );
            });

            assume_inside(contract2, || {
                set_value_transferred(0);
                let result = driver_ref.update_deadline(10);
                assert_eq!(
                    result,
                    Ok(()),
                    "Should succeed decrease deadline if no value transferred"
                );
            });

            assume_inside(contract2, || {
                set_value_transferred(0);
                let result = driver_ref.update_deadline(11);
                assert_eq!(
                    result,
                    Err(Error::Other("Not enough value".into())),
                    "Should fail if not enough value transferred"
                );
            });

            assume_inside(contract2, || {
                set_value_transferred(price * 1);
                let result = driver_ref.update_deadline(11);
                assert_eq!(
                    result,
                    Ok(()),
                    "Should succeed increase deadline if enough value transferred"
                );
            });
        }
    }
}

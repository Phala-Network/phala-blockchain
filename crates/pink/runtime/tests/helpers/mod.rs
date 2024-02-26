use pink_capi::types::Balance;
use sp_runtime::AccountId32;

pub use check_system::CheckSystemRef;
pub use test_cluster::TestCluster;

pub const ALICE: AccountId32 = AccountId32::new([1u8; 32]);
pub const TREASURY: AccountId32 = AccountId32::new([2u8; 32]);
pub const ENOUGH: Balance = u128::MAX / 2;

pub mod ink_helpers;
mod storage;
mod test_cluster;
mod xcalls;

pub fn checker_wasm() -> &'static [u8] {
    include_bytes!("../fixtures/check_system/check_system.wasm")
}

pub fn deploy_checker(cluster: &mut TestCluster, deposit: u128) -> CheckSystemRef {
    use ink_helpers::DeployBundle;
    use pink_capi::v1::ecall::ECalls;

    cluster.tx().deposit(ALICE.clone(), deposit);

    CheckSystemRef::default()
        .deploy_wasm(checker_wasm(), cluster)
        .expect("Failed to deploy checker")
}

pub fn create_cluster() -> (TestCluster, CheckSystemRef) {
    let mut cluster = TestCluster::for_test();
    let checker = deploy_checker(&mut cluster, 0);
    (cluster, checker)
}

use scale_info::TypeInfo;
use frame_support::pallet_prelude::*;
use sp_runtime::Permill;
use phala_types::WorkerPublicKey;
use crate::balance_convert::FixedPointConvert;
use sp_std::fmt::Display;
use crate::basepool;

#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct StakePool<AccountId, Balance> {
    pub basepool: basepool::BasePool<AccountId, Balance>,

    /// The commission the pool owner takes
    ///
    /// For example, 10% commission means 10% of the miner reward goes to the pool owner, and
    /// the remaining 90% is distributed to the contributors. Setting to `None` means a
    /// commission of 0%.
    pub payout_commission: Option<Permill>,
    /// Claimable owner reward
    ///
    /// Whenver a miner gets some reward, the commission the pool taken goes to here. The owner
    /// can claim their reward at any time.
    pub owner_reward: Balance,
    /// The hard capacity of the pool
    ///
    /// When it's set, the totals stake a pool can receive will not exceed this capacity.
    pub cap: Option<Balance>,
    /// Bound workers
    pub workers: Vec<WorkerPublicKey>,
    /// The workers in cd in the pool
    pub cd_workers: Vec<WorkerPublicKey>,
}

#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum PoolProxy<AccountId, Balance> {
    StakePool(StakePool<AccountId, Balance>),
}

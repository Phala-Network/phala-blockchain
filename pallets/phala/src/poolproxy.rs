use crate::basepool;
use frame_support::pallet_prelude::*;
use phala_types::WorkerPublicKey;
use scale_info::TypeInfo;
use sp_runtime::Permill;
use sp_std::collections::vec_deque::VecDeque;

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
	pub workers: VecDeque<WorkerPublicKey>,
	/// The workers in cd in the pool
	pub cd_workers: VecDeque<WorkerPublicKey>,
}

#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Vault<AccountId, Balance> {
	pub basepool: basepool::BasePool<AccountId, Balance>,

	pub pool_account_id: AccountId,

	pub last_share_price_checkpoint: Balance,

	pub delta_price_ratio: Option<Permill>,

	pub owner_shares: Balance,

	pub invest_pools: VecDeque<u64>,
}

impl<AccountId, Balance> StakePool<AccountId, Balance> {
	/// Removes a worker from the pool's worker list
	pub fn remove_worker(&mut self, worker: &WorkerPublicKey) {
		self.workers.retain(|w| w != worker);
	}

	/// Removes a worker from the pool's cd_worker list
	pub fn remove_cd_worker(&mut self, worker: &WorkerPublicKey) {
		self.cd_workers.retain(|w| w != worker);
	}
}

#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum PoolProxy<AccountId, Balance> {
	StakePool(StakePool<AccountId, Balance>),
	Vault(Vault<AccountId, Balance>),
}

pub fn ensure_stake_pool<T: basepool::Config>(
	pid: u64,
) -> Result<StakePool<T::AccountId, basepool::BalanceOf<T>>, basepool::Error<T>> {
	let pool_proxy = basepool::Pallet::<T>::pool_collection(pid)
		.ok_or(basepool::Error::<T>::PoolDoesNotExist)?;
	match pool_proxy {
		PoolProxy::StakePool(res) => Ok(res),
		_other => Err(basepool::Error::<T>::PoolTypeNotMatch),
	}
}

pub fn ensure_vault<T: basepool::Config>(
	pid: u64,
) -> Result<Vault<T::AccountId, basepool::BalanceOf<T>>, basepool::Error<T>> {
	let pool_proxy = basepool::Pallet::<T>::pool_collection(pid)
		.ok_or(basepool::Error::<T>::PoolDoesNotExist)?;
	match pool_proxy {
		PoolProxy::Vault(res) => Ok(res),
		_other => Err(basepool::Error::<T>::PoolTypeNotMatch),
	}
}

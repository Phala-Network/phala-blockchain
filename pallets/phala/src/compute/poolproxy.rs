use crate::basepool;
use crate::pawnshop;
use crate::vault;
use phala_types::WorkerPublicKey;
use scale_info::TypeInfo;
use sp_runtime::Permill;
use sp_std::collections::vec_deque::VecDeque;

use crate::BalanceOf;

use frame_support::pallet_prelude::*;
#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct StakePool<AccountId, Balance> {
	/// General fields of a pool
	pub basepool: basepool::BasePool<AccountId, Balance>,
	/// The commission the pool owner takes
	///
	/// For example, 10% commission means 10% of the miner reward goes to the pool owner, and
	/// the remaining 90% is distributed to the contributors. Setting to `None` means a
	/// commission of 0%.
	pub payout_commission: Option<Permill>,
	/// The hard capacity of the pool
	///
	/// When it's set, the totals stake a pool can receive will not exceed this capacity.
	pub cap: Option<Balance>,
	/// Bound workers
	pub workers: VecDeque<WorkerPublicKey>,
	/// The workers in cd in the pool
	pub cd_workers: VecDeque<WorkerPublicKey>,
	/// Generated account to store P-PHA locked in mining workers, controlled by the pallet
	pub lock_account: AccountId,
	/// Generated account to maintain owner rewards, controlled by the pallet
	pub owner_reward_account: AccountId,
}

#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Vault<AccountId, Balance> {
	/// General fields of a pool
	pub basepool: basepool::BasePool<AccountId, Balance>,
	/// Record the share price when function: `maybe_gain_owner_shares` is called last time
	pub last_share_price_checkpoint: Balance,
	/// The commission the owner takes
	///
	/// The calculated formula of the additional shares minted for pool owner is present below:
	///
	/// ```ignore
	/// additional_shares = total_value / (current_price - commission * min(current_price - last_share_price_checkpoint, 0)) - total_shares
	/// ```
	///
	/// When the pool's share price is lower than `last_share_price_checkpoint`, the pool made no profit  since the last checkpoint. So it won't gain any additional owner shares.
	///
	/// The `commission` indicates the percentage of the equivalent profits earned by the pool should be redistributed to the pool owner in the form of shares.
	pub commission: Option<Permill>,
	/// Claimable owner reward shares
	///
	/// Whenver the pool profit and function: `maybe_gain_owner_shares` is settled successfully, the commission the pool taken goes to here. The owner
	/// can claim their reward shares at any time.
	pub owner_shares: Balance,
	/// The upstream stake pools the vault has delegated to
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

	/// Get the value of owner rewards
	pub fn get_owner_stakes<T>(&self) -> Balance
	where
		T: pallet_assets::Config<AssetId = u32, Balance = Balance>,
		T: basepool::Config<AccountId = AccountId>,
		T: basepool::Config + pawnshop::Config + vault::Config,
	{
		pallet_assets::Pallet::<T>::balance(
			<T as pawnshop::Config>::PPhaAssetId::get(),
			&self.owner_reward_account,
		)
	}
}

/// The enumerate proxy caontains all kinds of pools (stake pool and vault is included currently)
#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum PoolProxy<AccountId, Balance> {
	StakePool(StakePool<AccountId, Balance>),
	Vault(Vault<AccountId, Balance>),
}

/// Returns a stakepool object by pid directly
///
/// Returns error when the mapping pool type of the pid mismatch a stake pool
pub fn ensure_stake_pool<T: basepool::Config>(
	pid: u64,
) -> Result<StakePool<T::AccountId, BalanceOf<T>>, basepool::Error<T>> {
	let pool_proxy = basepool::Pallet::<T>::pool_collection(pid)
		.ok_or(basepool::Error::<T>::PoolDoesNotExist)?;
	match pool_proxy {
		PoolProxy::StakePool(res) => Ok(res),
		_other => Err(basepool::Error::<T>::PoolTypeNotMatch),
	}
}

/// Returns a vault object by pid directly
///
/// Returns error when the mapping pool type of the pid mismatch a vault
pub fn ensure_vault<T: basepool::Config>(
	pid: u64,
) -> Result<Vault<T::AccountId, BalanceOf<T>>, basepool::Error<T>> {
	let pool_proxy = basepool::Pallet::<T>::pool_collection(pid)
		.ok_or(basepool::Error::<T>::PoolDoesNotExist)?;
	match pool_proxy {
		PoolProxy::Vault(res) => Ok(res),
		_other => Err(basepool::Error::<T>::PoolTypeNotMatch),
	}
}

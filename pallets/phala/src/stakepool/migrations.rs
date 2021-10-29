use super::*;

use crate::balance_convert::FixedPointConvert;
use frame_support::pallet_prelude::Weight;
use frame_support::traits::Get;
use log::info;
use sp_std::collections::btree_map::BTreeMap;
use sp_std::fmt::Display;
use sp_std::marker::PhantomData;
use sp_std::vec::Vec;

#[cfg(not(feature = "std"))]
use alloc::string::ToString;

pub(super) fn migrate_to_v1<T: Config>() -> Weight
where
	T: crate::mining::Config<Currency = <T as Config>::Currency>,
	BalanceOf<T>: FixedPointConvert + Display,
{
	Migration::<T>::migrate_fix487_490()
}

/// Indicating now it's pre or post migration
enum Stage {
	PreMigration,
	PostMigration,
}

/// Migration for bug 487 fix
struct Migration<T: Config>(PhantomData<T>);

impl<T: Config> Migration<T>
where
	T: crate::mining::Config<Currency = <T as Config>::Currency>,
	BalanceOf<T>: FixedPointConvert + Display,
{
	/// Fix bug #487
	pub fn migrate_fix487_490() -> Weight {
		info!("== migrate_fix487: Pre-Migration ==");
		Self::print_pool_details(Stage::PreMigration);
		let result = Self::backfill_487_490_maybe_remove_pool_dust();
		info!("== migrate_fix487: Post-Migration ==");
		Self::print_pool_details(Stage::PostMigration);
		info!("== migrate_fix487: {:?} ==", result);
		result.unwrap_or_default()
	}

	fn print_pool_details(stage: Stage) {
		#[cfg(feature = "std")]
		let _ = Self::maybe_dump_pool_details(stage);
	}

	fn log_pool_details() -> Result<(), ()> {
		info!("[PoolStakers]");
		info!("pid\tuser\tlocked\tshares");
		for ((pid, _account), user) in PoolStakers::<T>::iter() {
			info!("{}\t{:?}\t{}\t{}", pid, user.user, user.locked, user.shares);
		}
		info!("[StakePools]");
		info!("pid\ttotal_shares\ttotal_stake\tfree_stake\treleasing_stake\twithdraw_queue");
		for (pid, pool) in StakePools::<T>::iter() {
			let withdraw_queue_display = pool
				.withdraw_queue
				.iter()
				.map(|w| w.shares.to_string())
				.collect::<Vec<_>>()
				.join(",");

			info!(
				"{}\t{}\t{}\t{}\t{}\t{}",
				pid,
				pool.total_shares,
				pool.total_stake,
				pool.free_stake,
				pool.releasing_stake,
				withdraw_queue_display
			);
		}
		Ok(())
	}

	#[cfg(feature = "std")]
	fn maybe_dump_pool_details(stage: Stage) -> Result<(), ()> {
		use std::fs::File;
		use std::io::prelude::*;

		let path = match std::env::var("DUMP_MIGRATION_PATH") {
			Ok(path) => path,
			_ => return Ok(()),
		};
		let prefix = match stage {
			Stage::PreMigration => "pre",
			Stage::PostMigration => "post",
		};

		let mut file_users = File::create(format!("{}/{}-users.tsv", path, prefix))
			.expect("failed to open users output file");
		let mut file_pools = File::create(format!("{}/{}-pools.tsv", path, prefix))
			.expect("failed to open pools output file");

		write!(file_users, "pid\tuser\tlocked\tshares\n").or(Err(()))?;
		for ((pid, _account), user) in PoolStakers::<T>::iter() {
			write!(
				file_users,
				"{}\t{:?}\t{}\t{}\n",
				pid, user.user, user.locked, user.shares
			)
			.or(Err(()))?;
		}
		write!(
			file_pools,
			"pid\ttotal_shares\ttotal_stake\tfree_stake\treleasing_stake\twithdraw_queue\n"
		)
		.or(Err(()))?;
		for (pid, pool) in StakePools::<T>::iter() {
			let withdraw_queue_display = pool
				.withdraw_queue
				.iter()
				.map(|w| w.shares.to_string())
				.collect::<Vec<_>>()
				.join(",");

			write!(
				file_pools,
				"{}\t{}\t{}\t{}\t{}\t{}\n",
				pid,
				pool.total_shares,
				pool.total_stake,
				pool.free_stake,
				pool.releasing_stake,
				withdraw_queue_display
			)
			.or(Err(()))?;
		}
		Ok(())
	}

	// Removes the dust shares and stake in a pool.
	fn backfill_487_490_maybe_remove_pool_dust() -> Result<Weight, ()> {
		// BTreeMap is chosen for its slightly lower memory footprint
		let mut share_dust_removed = BTreeMap::<u64, BalanceOf<T>>::new();
		let mut num_reads = 0u64;
		let mut num_writes = 0u64;

		// Remove dust in stakers and collect dust shares that will be removed in stake pools
		// later.
		PoolStakers::<T>::translate(
			|key: (u64, T::AccountId), mut user: UserStakeInfo<T::AccountId, BalanceOf<T>>| {
				let (pid, account) = key;
				// Shares
				let (shares, shares_dust) = extract_dust(user.shares);
				user.shares = shares;
				// Accumulate the dust to the table to ensure shares invariant
				if shares_dust > Zero::zero() {
					let v = share_dust_removed
						.get(&pid)
						.cloned()
						.unwrap_or(Zero::zero());
					share_dust_removed.insert(pid, v + shares_dust);
				}
				// Locked
				let (locked, stake_dust) = extract_dust(user.locked);
				user.locked = locked;
				// Remove the dust in `locked` by slashing to treasury
				if stake_dust > Zero::zero() {
					info!("dust stake removed: {:?} {}", account, stake_dust);
					Pallet::<T>::ledger_reduce(&account, Zero::zero(), stake_dust);
				}
				num_reads += 1;
				num_writes += 1;
				Some(user)
			},
		);

		info!("share_dust_removed: {:#?}", share_dust_removed);

		// Remove dust in stake pools.
		StakePools::<T>::translate_values(|mut pool: PoolInfo<T::AccountId, BalanceOf<T>>| {
			let pid = pool.pid;
			// Maintain total_shares invarant
			if let Some(dust) = share_dust_removed.get(&pid) {
				pool.total_shares -= *dust;
			}
			// Remove dust from total_stake by NOP
			let (total_stake, _) = extract_dust(pool.total_stake);
			if total_stake == Zero::zero() {
				pool.total_stake = Zero::zero();
				pool.free_stake = Zero::zero();
			}
			// Remove dust in withdraw request
			pool.withdraw_queue
				.retain(|withdraw| is_nondust_balance(withdraw.shares));

			// Remove duplicated requests (we only allow one request per user)
			let mut visited = Vec::<T::AccountId>::new();
			pool.withdraw_queue.retain(|withdraw| {
				if !visited.contains(&withdraw.user) {
					visited.push(withdraw.user.clone());
					true
				} else {
					false
				}
			});

			num_reads += 1;
			num_writes += 1;
			Some(pool)
		});

		Ok(T::DbWeight::get().reads_writes(num_reads, num_writes))
	}
}

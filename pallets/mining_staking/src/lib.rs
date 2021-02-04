#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;
use sp_std::prelude::*;
use sp_std::cmp;
use codec::FullCodec;

use frame_support::{ensure, decl_module, decl_storage, decl_event, decl_error, dispatch};
use frame_system::{ensure_signed, ensure_root};

use alloc::vec::Vec;
use sp_runtime::{
	traits:: {
		AccountIdConversion, Zero
	},
	ModuleId,
};
use frame_support::{
	traits::{Currency, ExistenceRequirement},
};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

const PALLET_ID: ModuleId = ModuleId(*b"PHAPoWS.");

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type Currency: Currency<Self::AccountId>;
}

decl_storage! {
	trait Store for Module<T: Trait> as MiningStaking {
		Wallet get(fn wallet): map hasher(twox_64_concat) T::AccountId => BalanceOf<T>;
		PendingStaking get(fn pending_staking):
			double_map
				hasher(twox_64_concat) T::AccountId,
				hasher(twox_64_concat) T::AccountId
			=> BalanceOf<T>;
		PendingUnstaking get(fn pending_unstaking):
			double_map
				hasher(twox_64_concat) T::AccountId,
				hasher(twox_64_concat) T::AccountId
			=> BalanceOf<T>;
		Staked get(fn staked):
			double_map
				hasher(twox_64_concat) T::AccountId,
				hasher(twox_64_concat) T::AccountId
			=> BalanceOf<T>;

		// Indices
		/// WalletLocked
		WalletLocked get(fn wallet_locked):
			map hasher(twox_64_concat) T::AccountId => BalanceOf<T>;
		StakeReceived get(fn stake_received):
			map hasher(twox_64_concat) T::AccountId => BalanceOf<T>;
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId, Balance = BalanceOf<T> {
		/// Applied the pending stake
		PendingStakeApplied,
		PendingUnstakeAdded(AccountId, AccountId, Balance),
		PendingStakeAdded(AccountId, AccountId, Balance),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Trait> {
		InsufficientFunds,
		InsufficientStake,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;
		fn deposit_event() = default;

		/// Deposits to the stash account wallet.
		#[weight = 0]
		fn deposit(origin, value: BalanceOf<T>) -> dispatch::DispatchResult {
			let sender = ensure_signed(origin)?;
			T::Currency::transfer(
				&sender, &Self::account_id(), value, ExistenceRequirement::KeepAlive)?;
			Wallet::<T>::mutate(sender, |balance| *balance += value);
			Ok(())
		}

		/// Withdraws some available token from the stash account.
		#[weight = 0]
		fn withdraw(origin, value: BalanceOf<T>) -> dispatch::DispatchResult {
			let sender = ensure_signed(origin)?;
			let available = Self::available(&sender);
			ensure!(value <= available, Error::<T>::InsufficientFunds);
			T::Currency::transfer(
				&Self::account_id(), &sender, value, ExistenceRequirement::AllowDeath)?;
			Wallet::<T>::mutate(sender, |balance| *balance -= value);
			Ok(())
		}

		/// Adds some stake to a target
		#[weight = 0]
		fn stake(origin, to: T::AccountId, value: BalanceOf<T>) -> dispatch::DispatchResult {
			let sender = ensure_signed(origin)?;
			let zero: BalanceOf<T> = Zero::zero();
			// Check there are enough funds
			let pending_unstaking = PendingUnstaking::<T>::get(&sender, &to);
			let free = Self::available(&sender);
			ensure!(value <= pending_unstaking + free, Error::<T>::InsufficientFunds);
			// Cancel some unstaking operations first
			let mut to_stake = value;
			let to_cancel = cmp::min(pending_unstaking, value);
			if to_cancel > zero {
				PendingUnstaking::<T>::mutate(&sender, &to, |v| *v -= to_cancel);
				to_stake -= to_cancel;
			}
			// Then move the free tokens to cover the rest
			if to_stake > zero {
				Self::lock(&sender, &to, to_stake);
			}
			Ok(())
		}

		/// Remove some stack from a target
		#[weight = 0]
		fn unstake(origin, to: T::AccountId, value: BalanceOf<T>) -> dispatch::DispatchResult {
			let sender = ensure_signed(origin)?;
			let zero: BalanceOf<T> = Zero::zero();
			// Check there are enough funds
			let staked = Staked::<T>::get(&sender, &to);
			let pending = PendingStaking::<T>::get(&sender, &to);
			let unstaking = PendingUnstaking::<T>::get(&sender, &to);
			let to_cancel = cmp::min(value, pending);
			let to_unstake = value - to_cancel;
			ensure!(to_unstake + unstaking <= staked, Error::<T>::InsufficientStake);
			// Cancel some new stake first
			if to_cancel > zero {
				Self::cancel_lock(&sender, &to, to_cancel);
			}
			// Then add more pending unstaking
			if to_unstake > zero {
				PendingUnstaking::<T>::mutate(&sender, &to, |v| *v += to_unstake);
			}
			Ok(())
		}

		#[weight = 0]
		fn force_trigger_round_end(origin) -> dispatch::DispatchResult {
			ensure_root(origin)?;
			Self::handle_round_end();
			Ok(())
		}
	}
}

impl<T: Trait> Module<T> {
	pub fn account_id() -> T::AccountId {
		PALLET_ID.into_account()
	}

	/// Gets the availabe funds (wallet minus the pending staking tokens)
	fn available(who: &T::AccountId) -> BalanceOf<T> {
		Wallet::<T>::get(who) - WalletLocked::<T>::get(who)
	}

	fn lock(from: &T::AccountId, to: &T::AccountId, value: BalanceOf<T>) {
		PendingStaking::<T>::mutate(&from, &to, |v| *v += value);
		WalletLocked::<T>::mutate(&from, |v| *v += value);
	}

	fn cancel_lock(from: &T::AccountId, to: &T::AccountId, value: BalanceOf<T>) {
		PendingStaking::<T>::mutate(&from, &to, |v| *v -= value);
		WalletLocked::<T>::mutate(&from, |v| *v -= value);
	}

	fn inc_stake(from: &T::AccountId, to: &T::AccountId, value: BalanceOf<T>) {
		Staked::<T>::mutate(&from, &to, |v| *v += value);
		StakeReceived::<T>::mutate(&to, |v| *v += value);
	}

	fn dec_stake(from: &T::AccountId, to: &T::AccountId, value: BalanceOf<T>) {
		Staked::<T>::mutate(&from, &to, |v| *v -= value);
		StakeReceived::<T>::mutate(&to, |v| *v -= value);
	}

	/// Applies the pending staking and unstaking tokens at the end of a round.
	pub fn handle_round_end() {
		// Apply staking
		group_by_key(PendingStaking::<T>::drain(), |from, group| {
			let mut staked: BalanceOf<T> = Zero::zero();
			for (to, value) in group.iter() {
				staked += *value;
				Self::inc_stake(&from, &to, *value);
			}
			Wallet::<T>::mutate(&from, |v| *v -= staked);
		});
		// Apply unstaking
		group_by_key(PendingUnstaking::<T>::drain(), |from, group| {
			let mut unstaked: BalanceOf<T> = Zero::zero();
			for (to, value) in group {
				unstaked += *value;
				Self::dec_stake(&from, &to, *value);
			}
			Wallet::<T>::mutate(&from, |v| *v += unstaked);
		});
		// Clear the pending staking
		WalletLocked::<T>::drain().for_each(drop);
		Self::deposit_event(RawEvent::PendingStakeApplied)
	}
}

impl<T: Trait> pallet_phala::OnRoundEnd for Module<T> {
	fn on_round_end(_round: u32) {
		Self::handle_round_end();
	}
}

fn group_by_key<I, Op, AccountId, Balance>(iter: I, mut op: Op)
where
	Balance: FullCodec + Copy,
	AccountId: FullCodec + PartialEq + Clone,
	I: Iterator<Item = (AccountId, AccountId, Balance)>,
	Op: FnMut(&AccountId, &Vec<(AccountId, Balance)>),
{
	let mut last_key: Option<AccountId> = None;
	let mut group = Vec::<(AccountId, Balance)>::new();
	for (from, to, value) in iter {
		if let Some(ref last) = last_key {
			if last != &from {
				op(&last, &group);
				group.clear();
				last_key = Some(from.clone());
			}
		} else if last_key == None {
			last_key = Some(from.clone());
		}
		group.push((to.clone(), value));
	}
	if let Some(last) = last_key {
		op(&last, &group);
	}
}

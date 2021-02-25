#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;
use sp_std::{cmp};
use codec::{FullCodec};
use sp_std::prelude::*;

use alloc::vec::Vec;
use frame_support::{
  traits::{Currency, ExistenceRequirement::KeepAlive,
    ExistenceRequirement::AllowDeath},
  
};
use sp_runtime::{
  ModuleId,
  traits::{
    AccountIdConversion, Zero
  },
};
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

const PALLET_ID: ModuleId = ModuleId(*b"PHAPoWS.");

#[frame_support::pallet]
pub mod pallet {
  use frame_support::pallet_prelude::*;
  use frame_system::pallet_prelude::*;
  use super::*;

  /// Configure the pallet by specifying the parameters and types on which it depends.
  #[pallet::config]
  pub trait Config: frame_system::Config {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    type Currency: Currency<Self::AccountId>;
  }

  #[pallet::pallet]
  #[pallet::generate_store(pub(super) trait Store)]
  pub struct Pallet<T>(_);

  #[pallet::storage]
  #[pallet::getter(fn wallet)]
  pub type Wallet<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, BalanceOf<T>>;

  #[pallet::storage]
  #[pallet::getter(fn pending_staking)]
  pub type PendingStaking<T: Config> = StorageDoubleMap<
    _, Twox64Concat, T::AccountId, Twox64Concat, T::AccountId, BalanceOf<T>>;
  
  #[pallet::storage]
  #[pallet::getter(fn pending_unstaking)]
  pub type PendingUnstaking<T: Config> = StorageDoubleMap<
    _, Twox64Concat, T::AccountId, Twox64Concat, T::AccountId, BalanceOf<T>>;

  #[pallet::storage]
  #[pallet::getter(fn staked)]
  pub type Staked<T: Config> = StorageDoubleMap<
    _, Twox64Concat, T::AccountId, Twox64Concat, T::AccountId, BalanceOf<T>>;

  // Indices
  /// WalletLocked
  #[pallet::storage]
  #[pallet::getter(fn wallet_locked)]
  pub type WalletLocked<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, BalanceOf<T>>;

  #[pallet::storage]
  #[pallet::getter(fn stake_received)]
  pub type StakeReceived<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, BalanceOf<T>>;


  #[pallet::event]
  #[pallet::metadata(T::AccountId = "AccountId", BalanceOf<T> = "Balance")]
  #[pallet::generate_deposit(pub(super) fn deposit_event)]
  pub enum Event<T: Config> {
    /// Event documentation should end with an array that provides descriptive names for event
    /// parameters. [something, who]
    /// Applied the pending stake
    PendingStakeApplied,
    PendingUnstakeAdded(T::AccountId, T::AccountId, BalanceOf<T>),
    PendingStakeAdded(T::AccountId, T::AccountId, BalanceOf<T>),
  }
  
  #[pallet::error]
  pub enum Error<T> {
    /// Error names should be descriptive.
    InsufficientFunds,
    /// Errors should have helpful documentation associated with them.
    InsufficientStake,
  }



  #[pallet::hooks]
  impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

  // Dispatchable functions allows users to interact with the pallet and invoke state changes.
  // These functions materialize as "extrinsics", which are often compared to transactions.
  // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
  #[pallet::call]
  impl<T:Config> Pallet<T> {
    /// An example dispatchable that takes a singles value as a parameter, writes the value to
    /// storage and emits an event. This function must be dispatched by a signed extrinsic.
    /// Deposits to the stash account wallet.
    #[pallet::weight(0 + T::DbWeight::get().writes(1))]
    pub fn deposit(origin: OriginFor<T>, value: BalanceOf<T>) -> DispatchResultWithPostInfo  {
      let sender = ensure_signed(origin)?;
      T::Currency::transfer(
        &sender, &Self::account_id(), value, KeepAlive)?;
      Wallet::<T>::mutate(sender, |balance| *balance = Some(balance.unwrap_or_default() + value) );
      Ok(().into())
    }

    /// Withdraws some available token from the stash account.
    #[pallet::weight(0 + T::DbWeight::get().reads_writes(1,1))]
    pub fn withdraw(origin: OriginFor<T>, value: BalanceOf<T>) -> DispatchResultWithPostInfo  {
      let sender = ensure_signed(origin)?;
      let available = Self::available(&sender);
      ensure!(value <= available, Error::<T>::InsufficientFunds);
      T::Currency::transfer(
        &Self::account_id(), &sender, value, AllowDeath)?;
      Wallet::<T>::mutate(sender, |balance| *balance = Some(balance.unwrap_or_default() - value));
      Ok(().into())
    }

    /// Adds some stake to a target
    #[pallet::weight(0 + T::DbWeight::get().reads_writes(1,1))]
    pub fn stake(origin: OriginFor<T>, to: T::AccountId, value: BalanceOf<T>) -> DispatchResultWithPostInfo  {
      let sender = ensure_signed(origin)?;
      let zero: BalanceOf<T> = Zero::zero();
      // Check there are enough funds
      let pending_unstaking = PendingUnstaking::<T>::get(&sender, &to).unwrap_or_default();
      let free = Self::available(&sender);
      ensure!(value <= pending_unstaking + free, Error::<T>::InsufficientFunds);
      // Cancel some unstaking operations first
      let mut to_stake = value;
      let to_cancel = cmp::min(pending_unstaking, value);
      if to_cancel > zero {
        PendingUnstaking::<T>::mutate(&sender, &to, |v| *v = Some(v.unwrap_or_default() - to_cancel));
        to_stake -= to_cancel;
      }
      // Then move the free tokens to cover the rest
      if to_stake > zero {
        Self::lock(&sender, &to, to_stake);
      }
      Ok(().into())
    }

    /// Remove some stack from a target
    #[pallet::weight(0 + T::DbWeight::get().reads_writes(1,1))]
    pub fn unstake(origin: OriginFor<T>, to: T::AccountId, value: BalanceOf<T>) -> DispatchResultWithPostInfo  {
      let sender = ensure_signed(origin)?;
      let zero: BalanceOf<T> = Zero::zero();
      // Check there are enough funds
      let staked = Staked::<T>::get(&sender, &to).unwrap_or_default();
      let pending = PendingStaking::<T>::get(&sender, &to).unwrap_or_default();
      let unstaking = PendingUnstaking::<T>::get(&sender, &to).unwrap_or_default();
      let to_cancel = cmp::min(value, pending);
      let to_unstake = value - to_cancel;
      ensure!(to_unstake + unstaking <= staked, Error::<T>::InsufficientStake);
      // Cancel some new stake first
      if to_cancel > zero {
        Self::cancel_lock(&sender, &to, to_cancel);
      }
      // Then add more pending unstaking
      if to_unstake > zero {
        PendingUnstaking::<T>::mutate(&sender, &to, |v| *v = Some(v.unwrap_or_default() + to_unstake));
      }
      Ok(().into())
    }

    #[pallet::weight(0 + T::DbWeight::get().reads_writes(1,1))]
    pub fn force_trigger_round_end(origin: OriginFor<T>) -> DispatchResultWithPostInfo  {
      ensure_root(origin)?;
      Self::handle_round_end();
      Ok(().into())
    }

  }


  impl<T: Config> Pallet<T> {
    pub fn account_id() -> T::AccountId {
      PALLET_ID.into_account()
    }
  
    /// Gets the availabe funds (wallet minus the pending staking tokens)
    pub fn available(who: &T::AccountId) -> BalanceOf<T> {
      Wallet::<T>::get(who).unwrap_or_default() - WalletLocked::<T>::get(who).unwrap_or_default()
    }
  
    fn lock(from: &T::AccountId, to: &T::AccountId, value: BalanceOf<T>) {
      PendingStaking::<T>::mutate(&from, &to, |v| *v  = Some(v.unwrap_or_default() + value));
      WalletLocked::<T>::mutate(&from, |v| *v = Some(v.unwrap_or_default() + value));
    }
  
    fn cancel_lock(from: &T::AccountId, to: &T::AccountId, value: BalanceOf<T>) {
      PendingStaking::<T>::mutate(&from, &to, |v| *v = Some(v.unwrap_or_default() - value));
      WalletLocked::<T>::mutate(&from, |v| *v = Some(v.unwrap_or_default() - value));
    }
  
    fn inc_stake(from: &T::AccountId, to: &T::AccountId, value: BalanceOf<T>) {
      Staked::<T>::mutate(&from, &to, |v| *v = Some(v.unwrap_or_default() + value));
      StakeReceived::<T>::mutate(&to, |v| *v = Some(v.unwrap_or_default() + value));
    }
  
    fn dec_stake(from: &T::AccountId, to: &T::AccountId, value: BalanceOf<T>) {
      Staked::<T>::mutate(&from, &to, |v| *v = Some(v.unwrap_or_default() - value));
      StakeReceived::<T>::mutate(&to, |v| *v = Some(v.unwrap_or_default() - value));
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
        Wallet::<T>::mutate(&from, |v| *v = Some(v.unwrap_or_default() - staked));
      });
      // Apply unstaking
      group_by_key(PendingUnstaking::<T>::drain(), |from, group| {
        let mut unstaked: BalanceOf<T> = Zero::zero();
        for (to, value) in group {
          unstaked += *value;
          Self::dec_stake(&from, &to, *value);
        }
        Wallet::<T>::mutate(&from, |v| *v = Some(v.unwrap_or_default() + unstaked));
      });
      // Clear the pending staking
      WalletLocked::<T>::drain().for_each(drop);
      Self::deposit_event(Event::PendingStakeApplied)
    }
  }

  impl<T: Config> pallet_phala::OnRoundEnd for Pallet<T> {
    fn on_round_end(_round: u32) {
      Self::handle_round_end();
    }
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
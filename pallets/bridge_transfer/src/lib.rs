// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub use pallet::*;
#[frame_support::pallet]
pub mod pallet {
	use codec::{Decode, Encode};
	use frame_support::{
		fail,
		pallet_prelude::*,
		traits::{Currency, ExistenceRequirement, OnUnbalanced, StorageVersion, WithdrawReasons},
	};
	use frame_system::pallet_prelude::*;
	pub use pallet_bridge as bridge;
	use sp_arithmetic::traits::SaturatedConversion;
	use sp_core::U256;
	use sp_runtime::traits::{Saturating, Zero};
	use sp_std::convert::TryFrom;
	use sp_std::prelude::*;

	use pallet_mq::MessageOriginInfo;
	use phala_pallets::pallet_mq;

	type ResourceId = bridge::ResourceId;

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
		<T as frame_system::Config>::AccountId,
	>>::NegativeImbalance;

	#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
	pub struct AssetInfo {
		pub dest_id: bridge::BridgeChainId,
		pub asset_identity: Vec<u8>,
	}

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + bridge::Config + pallet_mq::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Specifies the origin check provided by the bridge for calls that can only be called by the bridge pallet
		type BridgeOrigin: EnsureOrigin<Self::Origin, Success = Self::AccountId>;

		/// The currency mechanism.
		type Currency: Currency<Self::AccountId>;

		#[pallet::constant]
		type ReserveTokenId: Get<ResourceId>;

		/// The handler to absorb the fee.
		type OnFeePay: OnUnbalanced<NegativeImbalanceOf<Self>>;
	}

	#[pallet::event]
	#[pallet::metadata(BalanceOf<T> = "Balance")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// [chainId, min_fee, fee_scale]
		FeeUpdated(bridge::BridgeChainId, BalanceOf<T>, u32),
		/// [chainId, asset_identity, reource_id]
		AssetRegistered(bridge::BridgeChainId, Vec<u8>, bridge::ResourceId),
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidTransfer,
		InvalidCommand,
		InvalidPayload,
		InvalidFeeOption,
		FeeOptionsMissing,
		InsufficientBalance,
		ResourceIdInUsed,
		AssetNotRegistered,
		AccountNotExist,
	}

	#[pallet::storage]
	#[pallet::getter(fn bridge_fee)]
	pub type BridgeFee<T: Config> =
		StorageMap<_, Blake2_256, bridge::BridgeChainId, (BalanceOf<T>, u32), ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn assets)]
	pub type Assets<T: Config> = StorageMap<_, Blake2_256, bridge::ResourceId, AssetInfo>;

	#[pallet::storage]
	#[pallet::getter(fn balances)]
	pub type Balances<T: Config> =
		StorageDoubleMap<_, Blake2_256, bridge::ResourceId, Blake2_256, T::AccountId, BalanceOf<T>>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Change extra bridge transfer fee that user should pay
		#[pallet::weight(195_000_000)]
		pub fn change_fee(
			origin: OriginFor<T>,
			min_fee: BalanceOf<T>,
			fee_scale: u32,
			dest_id: bridge::BridgeChainId,
		) -> DispatchResult {
			T::BridgeCommitteeOrigin::ensure_origin(origin)?;
			ensure!(fee_scale <= 1000u32, Error::<T>::InvalidFeeOption);
			BridgeFee::<T>::insert(dest_id, (min_fee, fee_scale));
			Self::deposit_event(Event::FeeUpdated(dest_id, min_fee, fee_scale));
			Ok(())
		}

		/// Register an asset.
		#[pallet::weight(195_000_000)]
		pub fn register_asset(
			origin: OriginFor<T>,
			asset_identity: Vec<u8>,
			dest_id: bridge::BridgeChainId,
		) -> DispatchResult {
			T::BridgeCommitteeOrigin::ensure_origin(origin)?;
			let resource_id = bridge::derive_resource_id(
				dest_id,
				&bridge::hashing::blake2_128(&asset_identity.to_vec()),
			);
			ensure!(
				!Assets::<T>::contains_key(resource_id),
				Error::<T>::ResourceIdInUsed
			);
			Assets::<T>::insert(
				resource_id,
				AssetInfo {
					dest_id,
					asset_identity: asset_identity.clone(),
				},
			);
			Self::deposit_event(Event::AssetRegistered(dest_id, asset_identity, resource_id));
			Ok(())
		}

		/// Transfer some amount of specific asset to some recipient on a (whitelisted) distination chain.
		#[pallet::weight(195_000_000)]
		pub fn transfer_assets(
			origin: OriginFor<T>,
			asset: bridge::ResourceId,
			amount: BalanceOf<T>,
			recipient: Vec<u8>,
			dest_id: bridge::BridgeChainId,
		) -> DispatchResult {
			let source = ensure_signed(origin)?;
			ensure!(
				<bridge::Pallet<T>>::chain_whitelisted(dest_id),
				Error::<T>::InvalidTransfer
			);
			ensure!(
				Assets::<T>::contains_key(&asset),
				Error::<T>::AssetNotRegistered
			);
			ensure!(
				BridgeFee::<T>::contains_key(&dest_id),
				Error::<T>::FeeOptionsMissing
			);
			let (min_fee, fee_scale) = Self::bridge_fee(dest_id);
			let fee_estimated = amount * fee_scale.into() / 1000u32.into();
			let fee = if fee_estimated > min_fee {
				fee_estimated
			} else {
				min_fee
			};

			// check account existence
			ensure!(
				Balances::<T>::contains_key(&asset, &source),
				Error::<T>::AccountNotExist
			);

			// check asset balance to cover transfer amount
			ensure!(
				Self::asset_balance(&asset, &source) >= amount,
				Error::<T>::InsufficientBalance
			);

			// check reserve balance to cover fee
			let reserve_free_balance = T::Currency::free_balance(&source);
			ensure!(reserve_free_balance >= fee, Error::<T>::InsufficientBalance);

			// pay fee to treasury
			let imbalance = T::Currency::withdraw(
				&source,
				fee,
				WithdrawReasons::FEE,
				ExistenceRequirement::AllowDeath,
			)?;
			T::OnFeePay::on_unbalanced(imbalance);

			// withdraw asset
			Self::do_asset_withdraw(&asset, &source, amount);

			<bridge::Pallet<T>>::transfer_fungible(
				dest_id,
				asset,
				recipient,
				U256::from(amount.saturated_into::<u128>()),
			)
		}

		/// Transfers some amount of the native token to some recipient on a (whitelisted) destination chain.
		#[pallet::weight(195_000_000)]
		pub fn transfer_native(
			origin: OriginFor<T>,
			amount: BalanceOf<T>,
			recipient: Vec<u8>,
			dest_id: bridge::BridgeChainId,
		) -> DispatchResult {
			let source = ensure_signed(origin)?;
			ensure!(
				<bridge::Pallet<T>>::chain_whitelisted(dest_id),
				Error::<T>::InvalidTransfer
			);
			let bridge_id = <bridge::Pallet<T>>::account_id();
			ensure!(
				BridgeFee::<T>::contains_key(&dest_id),
				Error::<T>::FeeOptionsMissing
			);
			let (min_fee, fee_scale) = Self::bridge_fee(dest_id);
			let fee_estimated = amount * fee_scale.into() / 1000u32.into();
			let fee = if fee_estimated > min_fee {
				fee_estimated
			} else {
				min_fee
			};
			let free_balance = T::Currency::free_balance(&source);
			ensure!(
				free_balance >= (amount + fee),
				Error::<T>::InsufficientBalance
			);

			let imbalance = T::Currency::withdraw(
				&source,
				fee,
				WithdrawReasons::FEE,
				ExistenceRequirement::AllowDeath,
			)?;
			T::OnFeePay::on_unbalanced(imbalance);
			<T as Config>::Currency::transfer(
				&source,
				&bridge_id,
				amount,
				ExistenceRequirement::AllowDeath,
			)?;

			<bridge::Pallet<T>>::transfer_fungible(
				dest_id,
				T::ReserveTokenId::get(),
				recipient,
				U256::from(amount.saturated_into::<u128>()),
			)
		}

		//
		// Executable calls. These can be triggered by a bridge transfer initiated on another chain
		//

		/// Executes a simple currency transfer using the bridge account as the source
		#[pallet::weight(195_000_000)]
		pub fn transfer(
			origin: OriginFor<T>,
			to: T::AccountId,
			amount: BalanceOf<T>,
			_rid: ResourceId,
		) -> DispatchResult {
			let source = T::BridgeOrigin::ensure_origin(origin)?;
			if _rid == T::ReserveTokenId::get() {
				<T as Config>::Currency::transfer(
					&source,
					&to,
					amount,
					ExistenceRequirement::AllowDeath,
				)?;
			} else {
				// check asset balance to cover transfer amount
				ensure!(
					Self::asset_balance(&_rid, &source) >= amount,
					Error::<T>::InsufficientBalance
				);
				Self::do_asset_deposit(&_rid, &to, amount);
			}

			Ok(())
		}
	}

	impl<T: Config> MessageOriginInfo for Pallet<T> {
		type Config = T;
	}

	impl<T: Config> Pallet<T> {
		pub fn asset_balance(asset: &bridge::ResourceId, who: &T::AccountId) -> BalanceOf<T> {
			Balances::<T>::get(asset, who).unwrap_or(Zero::zero())
		}

		/// Deposit specific amount assets into recipient account.
		///
		/// If recipient is bridge account, assets would deposited to bridge account only,
		/// otherwise, assets would be withdrawn from bridge account and then deposit to
		/// recipient.
		/// Bridge account is treat as holding account of all assets.
		///
		/// DO NOT guarantee asset was registered
		/// DO NOT guarantee bridge account(e.g. hodling account) has enough balance
		pub fn do_asset_deposit(
			asset: &bridge::ResourceId,
			recipient: &T::AccountId,
			amount: BalanceOf<T>,
		) {
			let bridge_id = <bridge::Pallet<T>>::account_id();
			if *recipient != bridge_id {
				Balances::<T>::mutate(asset, bridge_id, |maybe_balance| {
					if let Some(ref mut balance) = maybe_balance {
						balance.saturating_sub(amount);
					}
				});
			}

			Balances::<T>::mutate(asset, recipient, |maybe_balance| {
				if let Some(ref mut balance) = maybe_balance {
					balance.saturating_add(amount);
				} else {
					Some(amount);
				}
			});
		}

		/// Withdraw specific amount assets from sender.
		///
		/// Assets would be withdrawn from the sender and then deposit to bridge account.
		/// Bridge account is treat as holding account of all assets.
		///
		/// DO NOT guarantee asset was registered
		/// DO NOT grarantee sender account has enough balance
		pub fn do_asset_withdraw(
			asset: &bridge::ResourceId,
			sender: &T::AccountId,
			amount: BalanceOf<T>,
		) {
			let bridge_id = <bridge::Pallet<T>>::account_id();
			Balances::<T>::mutate(asset, sender, |maybe_balance| {
				if let Some(ref mut balance) = maybe_balance {
					balance.saturating_sub(amount);
				}
			});
			Balances::<T>::mutate(asset, bridge_id, |maybe_balance| {
				if let Some(ref mut balance) = maybe_balance {
					balance.saturating_add(amount);
				}
			});
		}
	}
}

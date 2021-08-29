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
		traits::{Currency, ExistenceRequirement::AllowDeath, StorageVersion},
	};
	use frame_system::pallet_prelude::*;
	pub use pallet_bridge as bridge;
	use sp_arithmetic::traits::SaturatedConversion;
	use sp_core::U256;
	use sp_std::convert::TryFrom;
	use sp_std::prelude::*;

	use pallet_mq::MessageOriginInfo;
	use phala_pallets::pallet_mq;
	use phala_types::messaging::{DecodedMessage, Lottery, LotteryCommand};

	type ResourceId = bridge::ResourceId;

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

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
		type BridgeTokenId: Get<ResourceId>;
		#[pallet::constant]
		type BridgeLotteryId: Get<ResourceId>;
	}

	#[pallet::event]
	#[pallet::metadata(BalanceOf<T> = "Balance")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// [chainId, min_fee, fee_scale]
		FeeUpdated(bridge::BridgeChainId, BalanceOf<T>, u32),
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidTransfer,
		InvalidCommand,
		InvalidPayload,
		InvalidFeeOption,
		FeeOptionsMissing,
	}

	#[pallet::storage]
	#[pallet::getter(fn bridge_fee)]
	pub type BridgeFee<T: Config> =
		StorageMap<_, Blake2_256, bridge::BridgeChainId, (BalanceOf<T>, u32), ValueQuery>;

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

		/// Transfers an arbitrary signed bitcoin tx to a (whitelisted) destination chain.
		#[pallet::weight(195_000_000)]
		pub fn force_lottery_output(
			origin: OriginFor<T>,
			payload: Vec<u8>,
			dest_id: bridge::BridgeChainId,
		) -> DispatchResult {
			ensure_root(origin)?;
			let lottery = Lottery::decode(&mut &payload[..]).or(Err(Error::<T>::InvalidPayload))?;
			Self::lottery_output(&lottery, dest_id)
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
			T::Currency::transfer(&source, &bridge_id, (amount + fee).into(), AllowDeath)?;

			<bridge::Pallet<T>>::transfer_fungible(
				dest_id,
				T::BridgeTokenId::get(),
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
			<T as Config>::Currency::transfer(&source, &to, amount.into(), AllowDeath)?;
			Ok(())
		}

		/// This can be called by the bridge to demonstrate an arbitrary call from a proposal.
		#[pallet::weight(195_000_000)]
		pub fn lottery_handler(
			origin: OriginFor<T>,
			metadata: Vec<u8>,
			_rid: ResourceId,
		) -> DispatchResult {
			T::BridgeOrigin::ensure_origin(origin)?;

			let op = u8::from_be_bytes(
				<[u8; 1]>::try_from(&metadata[..1]).map_err(|_| Error::<T>::InvalidCommand)?,
			);
			if op == 0 {
				ensure!(metadata.len() == 13, Error::<T>::InvalidCommand);

				Self::push_command(LotteryCommand::new_round(
					u32::from_be_bytes(
						<[u8; 4]>::try_from(&metadata[1..5])
							.map_err(|_| Error::<T>::InvalidCommand)?,
					), // roundId
					u32::from_be_bytes(
						<[u8; 4]>::try_from(&metadata[5..9])
							.map_err(|_| Error::<T>::InvalidCommand)?,
					), // totalCount
					u32::from_be_bytes(
						<[u8; 4]>::try_from(&metadata[9..])
							.map_err(|_| Error::<T>::InvalidCommand)?,
					), // winnerCount
				));
			} else if op == 1 {
				ensure!(metadata.len() > 13, Error::<T>::InvalidCommand);

				let address_len: usize = u32::from_be_bytes(
					<[u8; 4]>::try_from(&metadata[9..13])
						.map_err(|_| Error::<T>::InvalidCommand)?,
				)
				.saturated_into();
				ensure!(
					metadata.len() == (13 + address_len),
					Error::<T>::InvalidCommand
				);

				Self::push_command(LotteryCommand::open_box(
					u32::from_be_bytes(
						<[u8; 4]>::try_from(&metadata[1..5])
							.map_err(|_| Error::<T>::InvalidCommand)?,
					), // roundId
					u32::from_be_bytes(
						<[u8; 4]>::try_from(&metadata[5..9])
							.map_err(|_| Error::<T>::InvalidCommand)?,
					), // tokenId
					metadata[13..].to_vec(), // btcAddress
				));
			} else {
				fail!(Error::<T>::InvalidCommand);
			}

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn force_lottery_new_round(
			origin: OriginFor<T>,
			round_id: u32,
			total_count: u32,
			winner_count: u32,
		) -> DispatchResult {
			ensure_root(origin)?;
			Self::push_command(LotteryCommand::new_round(
				round_id,
				total_count,
				winner_count,
			));
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn force_lottery_open_box(
			origin: OriginFor<T>,
			round_id: u32,
			token_id: u32,
			btc_address: Vec<u8>,
		) -> DispatchResult {
			ensure_root(origin)?;
			Self::push_command(LotteryCommand::open_box(round_id, token_id, btc_address));
			Ok(())
		}
	}

	impl<T: Config> MessageOriginInfo for Pallet<T> {
		type Config = T;
	}

	impl<T: Config> Pallet<T> {
		pub fn lottery_output(payload: &Lottery, dest_id: bridge::BridgeChainId) -> DispatchResult {
			ensure!(
				<bridge::Pallet<T>>::chain_whitelisted(dest_id),
				Error::<T>::InvalidTransfer
			);
			let metadata: Vec<u8> = payload.encode();
			<bridge::Pallet<T>>::transfer_generic(dest_id, T::BridgeLotteryId::get(), metadata)
		}

		pub fn on_message_received(message: DecodedMessage<Lottery>) -> DispatchResult {
			// TODO.kevin: check the sender?
			// Dest chain 0 is EVM chain, and 1 is ourself
			Self::lottery_output(&message.payload, 0)
		}
	}
}

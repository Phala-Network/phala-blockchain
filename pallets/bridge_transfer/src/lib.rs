// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]
use codec::{Decode, Encode};
use frame_support::traits::{Currency, EnsureOrigin, ExistenceRequirement::AllowDeath};
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure, fail,
};
use frame_system::{self as system, ensure_root, ensure_signed};
use pallet_bridge as bridge;
use sp_arithmetic::traits::SaturatedConversion;
use sp_core::U256;
use sp_std::convert::TryFrom;
use sp_std::prelude::*;

use phala_pallets::{pallet_phala, pallet_mq};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

type ResourceId = bridge::ResourceId;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub trait Config: system::Config + bridge::Config + pallet_mq::Config {
	type Event: Into<<Self as frame_system::Config>::Event>;

	/// Specifies the origin check provided by the bridge for calls that can only be called by the bridge pallet
	type BridgeOrigin: EnsureOrigin<Self::Origin, Success = Self::AccountId>;

	/// The currency mechanism.
	type Currency: Currency<Self::AccountId>;
}

decl_storage! {
	trait Store for Module<T: Config> as BridgeTransfer {
		BridgeTokenId get(fn bridge_tokenid): ResourceId;
		BridgeLotteryId get(fn bridge_lotteryid): ResourceId;
	}

	add_extra_genesis {
		config(bridge_tokenid): ResourceId;
		config(bridge_lotteryid): ResourceId;
		build(|config: &GenesisConfig| {
			BridgeTokenId::put(config.bridge_tokenid);
			BridgeLotteryId::put(config.bridge_lotteryid);
		});
	}
}

#[derive(Decode, Encode, Debug, PartialEq, Eq, Clone)]
pub enum Event {
	/// Receive command: Newround. [roundId, totalCount, winnerCount]
	LotteryNewRound(u32, u32, u32),
	/// Receive commnad: Openbox. [roundId, tokenId, btcAddress]
	LotteryOpenBox(u32, u32, Vec<u8>),
}

phala_types::messaging::bind_topic!(Event, b"phala/lottery/event");

decl_error! {
	pub enum Error for Module<T: Config>{
		InvalidTransfer,
		InvalidCommand,
		InvalidPayload,
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		type Error = Error<T>;
		//
		// Initiation calls. These start a bridge transfer.
		//

		/// Transfers an arbitrary signed bitcoin tx to a (whitelisted) destination chain.
		#[weight = 195_000_000]
		pub fn force_lottery_output(origin, payload: Vec<u8>, dest_id: bridge::ChainId) -> DispatchResult {
			ensure_root(origin)?;
			let lottery = Lottery::decode(&mut &payload[..])
				.or(Err(Error::<T>::InvalidPayload))?;
			Self::lottery_output(&lottery, dest_id)
		}

		/// Transfers some amount of the native token to some recipient on a (whitelisted) destination chain.
		#[weight = 195_000_000]
		pub fn transfer_native(origin, amount: BalanceOf<T>, recipient: Vec<u8>, dest_id: bridge::ChainId) -> DispatchResult {
			let source = ensure_signed(origin)?;
			ensure!(<bridge::Module<T>>::chain_whitelisted(dest_id), Error::<T>::InvalidTransfer);
			let bridge_id = <bridge::Module<T>>::account_id();
			T::Currency::transfer(&source, &bridge_id, amount.into(), AllowDeath)?;

			let resource_id = Self::bridge_tokenid();

			<bridge::Module<T>>::transfer_fungible(dest_id, resource_id, recipient, U256::from(amount.saturated_into::<u128>()))
		}

		//
		// Executable calls. These can be triggered by a bridge transfer initiated on another chain
		//

		/// Executes a simple currency transfer using the bridge account as the source
		#[weight = 195_000_000]
		pub fn transfer(origin, to: T::AccountId, amount: BalanceOf<T>, _rid: ResourceId) -> DispatchResult {
			let source = T::BridgeOrigin::ensure_origin(origin)?;
			<T as Config>::Currency::transfer(&source, &to, amount.into(), AllowDeath)?;
			Ok(())
		}

		/// This can be called by the bridge to demonstrate an arbitrary call from a proposal.
		#[weight = 195_000_000]
		pub fn lottery_handler(origin, metadata: Vec<u8>, _rid: ResourceId) -> DispatchResult {
			T::BridgeOrigin::ensure_origin(origin)?;

			let op = u8::from_be_bytes(<[u8; 1]>::try_from(&metadata[..1]).map_err(|_| Error::<T>::InvalidCommand)?);
			if op == 0 {
				ensure!(
					metadata.len() == 13,
					Error::<T>::InvalidCommand
				);

				Self::push_message(Event::LotteryNewRound(
					u32::from_be_bytes(<[u8; 4]>::try_from(&metadata[1..5]).map_err(|_| Error::<T>::InvalidCommand)?),	// roundId
					u32::from_be_bytes(<[u8; 4]>::try_from(&metadata[5..9]).map_err(|_| Error::<T>::InvalidCommand)?),	// totalCount
					u32::from_be_bytes(<[u8; 4]>::try_from(&metadata[9..]).map_err(|_| Error::<T>::InvalidCommand)?)	// winnerCount
				));
			} else if op == 1 {
				ensure!(
					metadata.len() > 13,
					Error::<T>::InvalidCommand
				);

				let address_len: usize = u32::from_be_bytes(<[u8; 4]>::try_from(&metadata[9..13]).map_err(|_| Error::<T>::InvalidCommand)?).saturated_into();
				ensure!(
					metadata.len() == (13 + address_len),
					Error::<T>::InvalidCommand
				);

				Self::push_message(Event::LotteryOpenBox(
					u32::from_be_bytes(<[u8; 4]>::try_from(&metadata[1..5]).map_err(|_| Error::<T>::InvalidCommand)?),	// roundId
					u32::from_be_bytes(<[u8; 4]>::try_from(&metadata[5..9]).map_err(|_| Error::<T>::InvalidCommand)?),	// tokenId
					metadata[13..].to_vec()						// btcAddress
				));
			} else {
				fail!(Error::<T>::InvalidCommand);
			}

			Ok(())
		}

		#[weight = 0]
		fn force_lottery_new_round(origin, round_id: u32, total_count: u32, winner_count: u32) -> DispatchResult {
			ensure_root(origin)?;
			Self::push_message(Event::LotteryNewRound(round_id, total_count, winner_count));
			Ok(())
		}

		#[weight = 0]
		fn force_lottery_open_box(origin, round_id: u32, token_id: u32, btc_address: Vec<u8>) -> DispatchResult {
			ensure_root(origin)?;
			Self::push_message(Event::LotteryOpenBox(round_id, token_id, btc_address));
			Ok(())
		}
	}
}

use pallet_phala::OnMessageReceived;
use phala_types::messaging::{BindTopic, Lottery, Message, MessageOrigin};

impl<T: Config> Module<T> {
	pub fn lottery_output(payload: &Lottery, dest_id: bridge::ChainId) -> DispatchResult {
		ensure!(
			<bridge::Module<T>>::chain_whitelisted(dest_id),
			Error::<T>::InvalidTransfer
		);
		let resource_id = Self::bridge_lotteryid();
		let metadata: Vec<u8> = payload.encode();
		<bridge::Module<T>>::transfer_generic(dest_id, resource_id, metadata)
	}

	fn push_message(payload: impl Encode + BindTopic) {
		let sender = MessageOrigin::Pallet(b"BridgerTransfer".to_vec());
		pallet_mq::Pallet::<T>::push_message_typed(sender, payload);
	}
}

impl<T: Config> OnMessageReceived for Module<T> {
	fn on_message_received(_origin: &MessageOrigin, message: &Message) -> DispatchResult {
		// Dest chain 0 is EVM chain, and 1 is ourself
		let output: Lottery = message.decode_payload().ok_or(Error::<T>::InvalidPayload)?;
		Self::lottery_output(&output, 0)
	}
}

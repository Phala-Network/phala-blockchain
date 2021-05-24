// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::{Currency, EnsureOrigin, ExistenceRequirement::AllowDeath, Get};
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure, fail,
};
use frame_system::{self as system, ensure_root, ensure_signed};
use pallet_bridge as bridge;
use sp_arithmetic::traits::SaturatedConversion;
use sp_core::U256;
use sp_std::prelude::*;
use sp_std::convert::TryFrom;
use codec::{Encode, Decode};

mod mock;
mod tests;

type ResourceId = bridge::ResourceId;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub struct SendLottery {
	round_id: u32,
	chain_id: u8,
    token_id: Vec<u8>,
    tx: Vec<u8>,
    sequence: u64,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct SendLotteryData {
	data: SendLottery,
	signature: Vec<u8>,
}

pub trait Config: system::Config + bridge::Config {
	type Event: From<Event> + Into<<Self as frame_system::Config>::Event>;

	/// Specifies the origin check provided by the bridge for calls that can only be called by the bridge pallet
	type BridgeOrigin: EnsureOrigin<Self::Origin, Success = Self::AccountId>;

	/// The currency mechanism.
	type Currency: Currency<Self::AccountId>;
}

decl_storage! {
	trait Store for Module<T: Config> as BridgeTransfer {
		BridgeTokenId get(fn bridge_tokenid): ResourceId;
		BridgeLotteryId get(fn bridge_lotteryid): ResourceId;
		IngressSequence get(fn ingress_sequence): map hasher(twox_64_concat) u32 => u64;
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

decl_event! {
	pub enum Event {
		/// Receive command: Newround. [roundId, totalCount, winnerCount]
		LotteryNewRound(u32, u32, u32),
		/// Receive commnad: Openbox. [roundId, tokenId, btcAddress]
		LotteryOpenBox(u32, u32, Vec<u8>),
		/// A signed BTC transaction was send. [dest_chain, resource_id, payload]
		BTCSignedTxSend(u32, bridge::ChainId, ResourceId, Vec<u8>, u64),
	}
}

decl_error! {
	pub enum Error for Module<T: Config>{
		InvalidTransfer,
		InvalidCommand,
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		//
		// Initiation calls. These start a bridge transfer.
		//

		/// Transfers an arbitrary signed bitcoin tx to a (whitelisted) destination chain.
		#[weight = 195_000_000]
		pub fn sudo_transfer_lottery(origin, round_id: u32, token_id: Vec<u8>, payload: Vec<u8>, dest_id: bridge::ChainId, sequence: u64) -> DispatchResult {
			ensure_root(origin)?;
			Self::do_transfer_lottery(round_id, token_id, payload, dest_id, sequence)
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
		pub fn transfer(origin, to: T::AccountId, amount: BalanceOf<T>, rid: ResourceId) -> DispatchResult {
			let source = T::BridgeOrigin::ensure_origin(origin)?;
			<T as Config>::Currency::transfer(&source, &to, amount.into(), AllowDeath)?;
			Ok(())
		}

		/// This can be called by the bridge to demonstrate an arbitrary call from a proposal.
		#[weight = 195_000_000]
		pub fn lottery_handler(origin, metadata: Vec<u8>, rid: ResourceId) -> DispatchResult {
			T::BridgeOrigin::ensure_origin(origin)?;

			let op = u8::from_be_bytes(<[u8; 1]>::try_from(&metadata[..1]).map_err(|_| Error::<T>::InvalidCommand)?);
			if op == 0 {
				ensure!(
					metadata.len() == 13,
					Error::<T>::InvalidCommand
				);

				Self::deposit_event(Event::LotteryNewRound(
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

				Self::deposit_event(Event::LotteryOpenBox(
					u32::from_be_bytes(<[u8; 4]>::try_from(&metadata[1..5]).map_err(|_| Error::<T>::InvalidCommand)?),	// roundId
					u32::from_be_bytes(<[u8; 4]>::try_from(&metadata[5..9]).map_err(|_| Error::<T>::InvalidCommand)?),	// tokenId
					metadata[13..].to_vec()						// btcAddress
				));
			} else {
				fail!(Error::<T>::InvalidCommand);
			}

			Ok(())
		}

		#[weight = 195_000_000]
		pub fn transfer_to_chain(origin, data: Vec<u8>) -> DispatchResult {
			const CONTRACT_ID: u32 = 7;
			let transfer_data: SendLotteryData = Decode::decode(&mut &data[..]).map_err(|_| Error::<T>::InvalidCommand)?;
			// Check sequence
			let sequence = IngressSequence::get(CONTRACT_ID);
			ensure!(transfer_data.data.sequence == sequence + 1, Error::<T>::InvalidCommand);

			let round_id = &transfer_data.data.round_id;
			let chain_id = &transfer_data.data.chain_id;
			let token_id = &transfer_data.data.token_id;
			let tx_bytes = &transfer_data.data.tx;
			IngressSequence::insert(CONTRACT_ID, sequence + 1);
			Self::sudo_transfer_lottery(origin, *round_id, token_id.clone(), tx_bytes.to_vec(), *chain_id, sequence + 1)?;
			Ok(())
		}
	}
}

impl<T: Config> Module<T> {
	pub fn do_transfer_lottery(round_id: u32, token_id: Vec<u8>, payload: Vec<u8>, dest_id: bridge::ChainId, sequence: u64) -> DispatchResult {
		ensure!(<bridge::Module<T>>::chain_whitelisted(dest_id), Error::<T>::InvalidTransfer);

		let resource_id = Self::bridge_lotteryid();

		let payload_len: u32 = payload.len().saturated_into();
		let mut encoded_payload_len: Vec<u8> = payload_len.encode();
		encoded_payload_len.reverse();

		let mut encoded_round_id: Vec<u8> = round_id.encode();
		encoded_round_id.reverse();

		let mut encoded_token_id: Vec<u8> = token_id;
		encoded_token_id.reverse();

		let metadata: Vec<u8> = [encoded_round_id, encoded_token_id, encoded_payload_len, payload.clone()].concat();

		Self::deposit_event(Event::BTCSignedTxSend(round_id, dest_id, resource_id, payload, sequence));

		<bridge::Module<T>>::transfer_generic(dest_id, resource_id, metadata)
	}
}

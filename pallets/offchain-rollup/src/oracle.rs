//! # Sample Oracle based on Offchain Rollup

pub use self::pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		dispatch::DispatchResult, pallet_prelude::*, traits::StorageVersion, transactional,
	};
	use frame_system::pallet_prelude::*;
	use sp_core::H256;
	use sp_runtime::AccountId32;
	use sp_std::vec::Vec;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	pub type TradingPairBytes = BoundedVec<u8, ConstU32<64>>;

	/// Mapping from (deployer, traiding_pair) to price feed quotes
	#[pallet::storage]
	#[pallet::getter(fn price_feeds)]
	pub type PriceFeeds<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		AccountId32,
		Blake2_128Concat,
		TradingPairBytes,
		PriceQuote,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// Price quote received
		QuoteReceived {
			contract: H256,
			submitter: T::AccountId,
			owner: AccountId32,
			pair: TradingPairBytes,
			price: u128,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		FailedToAuthenticateResponse,
		FailedToDecodeResponse,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Sends a request to the oracle
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		#[transactional]
		pub fn request(
			origin: OriginFor<T>,
			_name: H256,
			_data: RequestBytes,
			_nonce: u128,
		) -> DispatchResult {
			ensure_signed(origin)?;
			Ok(())
		}
	}

	impl<T: Config> crate::anchor::OnResponse<T::AccountId> for Pallet<T> {
		fn on_response(name: H256, submitter: T::AccountId, data: Vec<u8>) -> DispatchResult {
			let resp: ResponseRecord =
				Decode::decode(&mut &data[..]).or(Err(Error::<T>::FailedToDecodeResponse))?;
			if resp.contract_id != name {
				return Err(Error::<T>::FailedToAuthenticateResponse.into());
			}
			// TODO: is timestamp newer?
			PriceFeeds::<T>::insert(
				&resp.owner,
				&resp.pair,
				PriceQuote {
					contract_id: resp.contract_id,
					price: resp.price,
					timestamp_ms: resp.timestamp_ms,
				},
			);
			Self::deposit_event(Event::QuoteReceived {
				contract: name,
				submitter,
				owner: resp.owner,
				pair: resp.pair,
				price: resp.price,
			});
			Ok(())
		}
	}

	// impl<T: Config> Pallet<T> {
	// }

	// Structures

	pub type RequestBytes = BoundedVec<u8, ConstU32<64>>;

	/// A quote from a price feed oracle
	#[derive(Debug, PartialEq, Eq, Encode, Decode, Clone, scale_info::TypeInfo, MaxEncodedLen)]
	pub struct PriceQuote {
		contract_id: H256,
		price: u128,
		timestamp_ms: u64,
	}

	/// The reponse from the oracle Phat Contract (copied from Phat Contract)
	#[derive(Debug, PartialEq, Eq, Encode, Decode, Clone, scale_info::TypeInfo)]
	pub struct ResponseRecord {
		pub owner: AccountId32,
		pub contract_id: H256,
		pub pair: TradingPairBytes,
		pub price: u128,
		pub timestamp_ms: u64,
	}
}

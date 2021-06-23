/// Public key registry for workers and contracts.
pub use self::pallet::*;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use sp_core::H256;
	use sp_std::convert::TryFrom;
	use sp_std::prelude::*;
	use sp_std::vec;

	use phala_types::{
		messaging::{MessageOrigin, SignedMessage},
		Score, SignedDataType, WorkerInfo, WorkerStateEnum,
	};

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Mapping from worker pubkey to WorkerInfo
	#[pallet::storage]
	pub type WorkerState<T: Config> =
		StorageMap<_, Twox64Concat, Vec<u8>, WorkerInfo<T::BlockNumber>>;

	/// Mapping from contract address to pubkey
	#[pallet::storage]
	pub type ContractKey<T> = StorageMap<_, Twox64Concat, H256, Vec<u8>>;

	/// Pubkey for secret topics.
	#[pallet::storage]
	pub type TopicKey<T> = StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event {
		SomeEvent,
	}

	#[pallet::error]
	pub enum Error<T> {
		CannotHandleUnknownMessage,
		InvalidSender,
		InvalidPubKey,
		MalformedSignature,
		InvalidSignatureLength,
		InvalidSignature,
		UnknwonContract,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Force register a worker with the given pubkey with sudo permission
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn force_register_worker(origin: OriginFor<T>, pubkey: Vec<u8>) -> DispatchResult {
			ensure_root(origin)?;
			let worker_info = WorkerInfo::<T::BlockNumber> {
				machine_id: Vec::new(), // not used
				pubkey: pubkey.clone(),
				last_updated: 0,
				state: WorkerStateEnum::Free,
				score: Some(Score {
					overall_score: 100,
					features: vec![1, 4],
				}),
				confidence_level: 128u8,
				runtime_version: 0,
			};
			WorkerState::<T>::insert(&worker_info.pubkey, &worker_info);
			Ok(())
		}

		/// Force register a contract pubkey
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn force_register_contract(
			origin: OriginFor<T>,
			contract: H256,
			pubkey: Vec<u8>,
		) -> DispatchResult {
			ensure_root(origin)?;
			ContractKey::<T>::insert(contract, pubkey);
			Ok(())
		}

		/// Force register a topic pubkey
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn force_register_topic_pubkey(
			origin: OriginFor<T>,
			topic: Vec<u8>,
			pubkey: Vec<u8>,
		) -> DispatchResult {
			ensure_root(origin)?;
			TopicKey::<T>::insert(topic, pubkey);
			Ok(())
		}
	}

	// TODO.kevin: Move it to mq
	impl<T: Config> Pallet<T> {
		pub fn check_message(message: &SignedMessage) -> DispatchResult {
			let pubkey_copy: Vec<u8>;
			let pubkey = match &message.message.sender {
				MessageOrigin::Worker(pubkey) => pubkey,
				MessageOrigin::Contract(id) => {
					pubkey_copy = ContractKey::<T>::get(id).ok_or(Error::<T>::UnknwonContract)?;
					&pubkey_copy
				}
				_ => return Err(Error::<T>::CannotHandleUnknownMessage.into()),
			};
			Self::verify_signature(pubkey, message)
		}

		fn verify_signature(
			serialized_pk: &Vec<u8>,
			data: &impl SignedDataType<Vec<u8>>,
		) -> DispatchResult {
			ensure!(serialized_pk.len() == 33, Error::<T>::InvalidPubKey);
			let pubkey = sp_core::ecdsa::Public::try_from(serialized_pk.as_slice())
				.or(Err(Error::<T>::InvalidPubKey))?;
			let raw_sig = data.signature();
			ensure!(raw_sig.len() == 65, Error::<T>::InvalidSignatureLength);
			let sig = sp_core::ecdsa::Signature::try_from(raw_sig.as_slice())
				.or(Err(Error::<T>::MalformedSignature))?;
			let data = data.raw_data();
			ensure!(
				sp_io::crypto::ecdsa_verify(&sig, &data, &pubkey),
				Error::<T>::InvalidSignature
			);
			Ok(())
		}
	}
}

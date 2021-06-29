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
		ContractPublicKey, PRuntimeInfo, WorkerPublicKey,
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
	pub type Worker<T: Config> = StorageMap<_, Twox64Concat, WorkerPublicKey, WorkerInfo>;

	/// Mapping from contract address to pubkey
	#[pallet::storage]
	pub type ContractKey<T> = StorageMap<_, Twox64Concat, H256, ContractPublicKey>;

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
		pub fn force_register_worker(
			origin: OriginFor<T>,
			pubkey: WorkerPublicKey,
		) -> DispatchResult {
			ensure_root(origin)?;
			let worker_info = WorkerInfo {
				pubkey: pubkey.clone(),
				runtime_version: 0,
				last_updated: 0,
				confidence_level: 128u8,
				intial_score: None,
				benchmark_start_ts: None,
				legacy_scores: vec![1, 4],
			};
			Worker::<T>::insert(&worker_info.pubkey, &worker_info);
			Ok(())
		}

		/// Force register a contract pubkey
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn force_register_contract(
			origin: OriginFor<T>,
			contract: H256,
			pubkey: ContractPublicKey,
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

		/// (called by anyone on behalf of a worker)
		#[pallet::weight(0)]
		pub fn register_worker(
			origin: OriginFor<T>,
			pruntime_info: PRuntimeInfo,
			attestation: Attestation,
		) -> DispatchResult {
			panic!("unimpleneted");
		}

		/// Unbinds a worker from a miner
		/// (called by a miner)
		#[pallet::weight(0)]
		pub fn miner_unbind(
			origin: OriginFor<T>,
			pruntime_info: PRuntimeInfo,
			attestation: Attestation,
		) -> DispatchResult {
			panic!("unimpleneted");
		}
	}

	// TODO.kevin: Move it to mq
	impl<T: Config> Pallet<T> {
		pub fn check_message(message: &SignedMessage) -> DispatchResult {
			let pubkey_copy: ContractPublicKey;
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
			pubkey: &WorkerPublicKey,
			message: &SignedMessage
		) -> DispatchResult {
			let raw_sig = &message.signature;
			ensure!(raw_sig.len() == 65, Error::<T>::InvalidSignatureLength);
			let sig = sp_core::ecdsa::Signature::try_from(raw_sig.as_slice())
				.or(Err(Error::<T>::MalformedSignature))?;
			let data = message.data_be_signed();
			ensure!(
				sp_io::crypto::ecdsa_verify(&sig, &data, &pubkey),
				Error::<T>::InvalidSignature
			);
			Ok(())
		}
	}

	#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq)]
	pub enum Attestation {
		SgxIas {
			ra_report: Vec<u8>,
			signature: Vec<u8>,
		},
	}

	#[derive(Encode, Decode, Default, Debug, Clone)]
	pub struct WorkerInfo {
		// identity
		pubkey: WorkerPublicKey,
		// system
		runtime_version: u32,
		last_updated: u64,
		// platform
		confidence_level: u8,
		// scoring
		intial_score: Option<u32>,
		benchmark_start_ts: Option<u64>,
		legacy_scores: Vec<u8>,
	}
}

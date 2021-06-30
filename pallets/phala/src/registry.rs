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
	use codec::Encode;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::UnixTime};
	use frame_system::pallet_prelude::*;
	use sp_core::H256;
	use sp_runtime::SaturatedConversion;
	use sp_std::convert::TryFrom;
	use sp_std::prelude::*;
	use sp_std::vec;

	use crate::attestation::{validate_ias_report, Error as AttestationError};

	use phala_types::{
		messaging::{MessageOrigin, SignedMessage},
		ContractPublicKey, PRuntimeInfo, WorkerPublicKey,
	};

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event> + IsType<<Self as frame_system::Config>::Event>;

		type UnixTime: UnixTime;
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
		// IAS related
		InvalidIASSigningCert,
		InvalidReport,
		InvalidQuoteStatus,
		BadIASReport,
		OutdatedIASReport,
		UnknownQuoteBodyFormat,
		// Report validation
		InvalidRuntimeInfoHash,
		InvalidRuntimeInfo,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Force register a worker with the given pubkey with sudo permission
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn force_register_worker(
			origin: OriginFor<T>,
			pubkey: WorkerPublicKey,
			ecdh_pubkey: WorkerPublicKey,
		) -> DispatchResult {
			ensure_root(origin)?;
			let worker_info = WorkerInfo {
				pubkey: pubkey.clone(),
				ecdh_pubkey,
				runtime_version: 0,
				last_updated: 0,
				confidence_level: 128u8,
				intial_score: None,
				benchmark_start_ts: None,
				features: vec![1, 4],
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
			ensure_signed(origin)?;
			// Validate RA report
			let now = T::UnixTime::now().as_secs().saturated_into::<u64>();
			let fields = match attestation {
				Attestation::SgxIas {
					ra_report,
					signature,
					raw_signing_cert,
				} => validate_ias_report(&ra_report, &signature, &raw_signing_cert, now)
					.map_err(Into::<Error<T>>::into)?,
			};
			// Validate fields

			// TODO(h4x): Add back mrenclave whitelist check
			// let whitelist = MREnclaveWhitelist::get();
			// let t_mrenclave = Self::extend_mrenclave(&fields.mr_enclave, &fields.mr_signer, &fields.isv_prod_id, &fields.isv_svn);
			// ensure!(whitelist.contains(&t_mrenclave), Error::<T>::WrongMREnclave);

			// Validate pruntime_info
			let runtime_info_hash = sp_core::blake2_256(&Encode::encode(&pruntime_info));
			let commit = &fields.report_data[..32];
			ensure!(
				runtime_info_hash.to_vec() == commit,
				Error::<T>::InvalidRuntimeInfoHash
			);
			let runtime_version = pruntime_info.version;
			let machine_id = pruntime_info.machine_id.to_vec();
			// Update the registry
			Worker::<T>::mutate(pruntime_info.pubkey.clone(), |v| {
				match v {
					Some(worker_info) => {
						// Case 1 - Only refresh the RA report; no need to redo benchmark
						worker_info.last_updated = now;
					}
					None => {
						// Case 2 - New worker register
						*v = Some(WorkerInfo {
							pubkey: pruntime_info.pubkey,
							ecdh_pubkey: Default::default(), // TODO(shelvenzhou): add ecdh key
							runtime_version: pruntime_info.version,
							last_updated: now,
							confidence_level: fields.confidence_level,
							intial_score: None,
							benchmark_start_ts: Some(now),
							features: pruntime_info.features,
						});
						// TODO(kevin): Send msg to Worker: StartInitialBenchmark
						//
						// Alternatively, we can also ask the worker to watch a kvdb entry in this
						// pallet (i.e. Worker[pubkey].initial_score == None)
					}
				}
			});

			Ok(())
		}

		/// Unbinds a worker from a miner
		/// (called by a miner)
		#[pallet::weight(0)]
		pub fn miner_unbind(origin: OriginFor<T>) -> DispatchResult {
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

		fn verify_signature(pubkey: &WorkerPublicKey, message: &SignedMessage) -> DispatchResult {
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

	// TODO(kevin): Handle messages
	//
	// match message {
	// 	RegistryMessage::EndBench { iterations } => {
	// 		WorkerInfo::<T>::mutate(|info| {
	// 			let now = now();
	// 			let socre = calculate_score(iterations, info.benchmark_start_ts, now);
	// 			info.intial_score = Some(score);
	// 		})
	// 	}
	// }

	#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq)]
	pub enum Attestation {
		SgxIas {
			ra_report: Vec<u8>,
			signature: Vec<u8>,
			raw_signing_cert: Vec<u8>,
		},
	}

	#[derive(Encode, Decode, Default, Debug, Clone)]
	pub struct WorkerInfo {
		// identity
		pubkey: WorkerPublicKey,
		ecdh_pubkey: WorkerPublicKey,
		// system
		runtime_version: u32,
		last_updated: u64,
		// platform
		confidence_level: u8,
		// scoring
		benchmark_start_ts: Option<u64>,
		intial_score: Option<u32>,
		features: Vec<u32>,
	}

	impl<T: Config> From<AttestationError> for Error<T> {
		fn from(err: AttestationError) -> Self {
			match err {
				AttestationError::InvalidIASSigningCert => Self::InvalidIASSigningCert,
				AttestationError::InvalidReport => Self::InvalidReport,
				AttestationError::InvalidQuoteStatus => Self::InvalidQuoteStatus,
				AttestationError::BadIASReport => Self::BadIASReport,
				AttestationError::OutdatedIASReport => Self::OutdatedIASReport,
				AttestationError::UnknownQuoteBodyFormat => Self::UnknownQuoteBodyFormat,
			}
		}
	}
}

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
	use crate::mq::MessageOriginInfo;

	use phala_types::{
		messaging::{bind_topic, Message, MessageOrigin, SignedMessage, SystemEvent},
		ContractPublicKey, PRuntimeInfo, WorkerPublicKey,
	};

	bind_topic!(RegistryEvent, b"^phala/registry/event");
	#[derive(Encode, Decode, Clone, Debug)]
	pub enum RegistryEvent {
		BenchReport { start_time: u64, iterations: u64 },
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event> + IsType<<Self as frame_system::Config>::Event>;

		type UnixTime: UnixTime;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Gatekeeper pubkey list
	#[pallet::storage]
	pub type Gatekeeper<T: Config> = StorageValue<_, Vec<WorkerPublicKey>, ValueQuery>;

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
		GatekeeperAdded(WorkerPublicKey),
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
		InvalidInput,
		InvalidBenchReport,
		WorkerNotFound,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		T: crate::mq::Config,
	{
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
				session_id: 1,
				features: vec![1, 4],
			};
			Worker::<T>::insert(&worker_info.pubkey, &worker_info);
			Self::push_message(SystemEvent::WorkerAttached {
				pubkey,
				session_id: 1,
			});
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

		/// Register a gatekeeper.
		///
		/// Must be called by the Root origin.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn register_gatekeeper(
			origin: OriginFor<T>,
			gatekeeper: WorkerPublicKey,
		) -> DispatchResult {
			ensure_root(origin)?;
			let mut gatekeepers = Gatekeeper::<T>::get();
			if !gatekeepers.contains(&gatekeeper) {
				gatekeepers.push(gatekeeper.clone());
				Gatekeeper::<T>::put(gatekeepers);
				Self::deposit_event(Event::GatekeeperAdded(gatekeeper));
			}
			Ok(())
		}

		/// (called by anyone on behalf of a worker)
		#[pallet::weight(0)]
		pub fn register_worker(
			origin: OriginFor<T>,
			pruntime_info: PRuntimeInfo<T::AccountId>,
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
			let runtime_info_hash = crate::hashing::blake2_256(&Encode::encode(&pruntime_info));
			let commit = &fields.report_data[..32];
			ensure!(
				&runtime_info_hash == commit,
				Error::<T>::InvalidRuntimeInfoHash
			);
			let runtime_version = pruntime_info.version;
			let machine_id = pruntime_info.machine_id.to_vec();
			// Update the registry
			Worker::<T>::mutate(pruntime_info.pubkey.clone(), |v| {
				match v {
					Some(worker_info) => {
						// Case 1 - Refresh the RA report and redo benchmark
						worker_info.last_updated = now;
						worker_info.session_id += 1;
						Self::push_message(SystemEvent::WorkerAttached {
							pubkey: pruntime_info.pubkey.clone(),
							session_id: worker_info.session_id,
						});
						let now = T::UnixTime::now().as_secs().saturated_into::<u64>();
						Self::push_message(SystemEvent::BenchStart {
							pubkey: pruntime_info.pubkey,
							start_time: now,
						});
					}
					None => {
						// Case 2 - New worker register
						let session_id = 1;
						*v = Some(WorkerInfo {
							pubkey: pruntime_info.pubkey.clone(),
							ecdh_pubkey: Default::default(), // TODO(shelvenzhou): add ecdh key
							runtime_version: pruntime_info.version,
							last_updated: now,
							confidence_level: fields.confidence_level,
							intial_score: None,
							session_id,
							features: pruntime_info.features,
						});
						Self::push_message(SystemEvent::WorkerAttached {
							pubkey: pruntime_info.pubkey.clone(),
							session_id,
						});
						let now = T::UnixTime::now().as_secs().saturated_into::<u64>();
						Self::push_message(SystemEvent::BenchStart {
							pubkey: pruntime_info.pubkey,
							start_time: now,
						});
					}
				}
			});

			Ok(())
		}

		/// Unbinds a worker from a miner
		///
		/// Requirements:
		//  1. `origin` is the `worker`'s operator
		#[pallet::weight(0)]
		pub fn unbind(origin: OriginFor<T>, worker: WorkerPublicKey) -> DispatchResult {
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

		pub fn on_message_received(message: &Message) -> DispatchResult {
			let worker_pubkey = match &message.sender {
				MessageOrigin::Worker(key) => key,
				_ => return Err(Error::<T>::InvalidSender.into()),
			};

			let message: RegistryEvent =
				message.decode_payload().ok_or(Error::<T>::InvalidInput)?;

			match message {
				RegistryEvent::BenchReport {
					start_time,
					iterations,
				} => {
					let now = T::UnixTime::now().as_secs().saturated_into::<u64>();
					if now <= start_time {
						// Oops, should not happen
						return Err(Error::<T>::InvalidBenchReport.into());
					}

					const MAX_SCORE: u32 = 6000;
					let score = iterations / (now - start_time);
					let score = MAX_SCORE.min(score as u32);

					Worker::<T>::mutate(worker_pubkey, |val| {
						if let Some(val) = val {
							val.intial_score = Some(score);
						}
					})
				}
			}
			Ok(())
		}
	}

	impl<T: Config + crate::mq::Config> MessageOriginInfo for Pallet<T> {
		type Config = T;
	}

	#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq)]
	pub enum Attestation {
		SgxIas {
			ra_report: Vec<u8>,
			signature: Vec<u8>,
			raw_signing_cert: Vec<u8>,
		},
	}

	// TODO.shelven: handle the WorkerInfo in phala_types
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
		session_id: u64,
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

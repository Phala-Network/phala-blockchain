/// Public key registry for workers and contracts.
pub use self::pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use codec::Encode;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::UnixTime};
	use frame_system::pallet_prelude::*;
	use sp_core::H256;
	use sp_runtime::SaturatedConversion;
	use sp_std::prelude::*;
	use sp_std::{
		convert::{TryFrom, TryInto},
		vec,
	};

	use crate::attestation::{validate_ias_report, Error as AttestationError};
	use crate::mq::MessageOriginInfo;

	use phala_types::{
		messaging::{
			self, bind_topic, DecodedMessage, GatekeeperEvent, MessageOrigin, SignedMessage,
			SystemEvent, WorkerEvent,
		},
		ContractPublicKey, EcdhPublicKey, MasterPublicKey, WorkerPublicKey, WorkerRegistrationInfo,
	};

	bind_topic!(RegistryEvent, b"^phala/registry/event");
	#[derive(Encode, Decode, Clone, Debug)]
	pub enum RegistryEvent {
		BenchReport { start_time: u64, iterations: u64 },
		MasterPubkey { master_pubkey: MasterPublicKey },
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

	/// Gatekeeper master pubkey
	#[pallet::storage]
	pub type GatekeeperMasterPubkey<T: Config> = StorageValue<_, MasterPublicKey>;

	/// Mapping from worker pubkey to WorkerInfo
	#[pallet::storage]
	pub type Workers<T: Config> =
		StorageMap<_, Twox64Concat, WorkerPublicKey, WorkerInfo<T::AccountId>>;

	/// Mapping from contract address to pubkey
	#[pallet::storage]
	pub type ContractKey<T> = StorageMap<_, Twox64Concat, H256, ContractPublicKey>;

	/// Pubkey for secret topics.
	#[pallet::storage]
	pub type TopicKey<T> = StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>>;

	#[pallet::storage]
	pub type BenchmarkDuration<T: Config> = StorageValue<_, u32>;

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
		// Gatekeeper related
		InvalidGatekeeper,
		InvalidMasterPubkey,
		MasterKeyMismatch,
		MasterKeyUninitialized,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		T: crate::mq::Config,
	{
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn force_set_benchmark_duration(origin: OriginFor<T>, value: u32) -> DispatchResult {
			ensure_root(origin)?;
			BenchmarkDuration::<T>::put(value);
			Ok(())
		}

		/// Force register a worker with the given pubkey with sudo permission
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn force_register_worker(
			origin: OriginFor<T>,
			pubkey: WorkerPublicKey,
			ecdh_pubkey: EcdhPublicKey,
			operator: Option<T::AccountId>,
		) -> DispatchResult {
			ensure_root(origin)?;
			let worker_info = WorkerInfo {
				pubkey: pubkey.clone(),
				ecdh_pubkey,
				runtime_version: 0,
				last_updated: 0,
				operator,
				confidence_level: 128u8,
				initial_score: None,
				features: vec![1, 4],
			};
			Workers::<T>::insert(&worker_info.pubkey, &worker_info);
			Self::push_message(SystemEvent::new_worker_event(
				pubkey,
				WorkerEvent::Registered(messaging::WorkerInfo {
					confidence_level: worker_info.confidence_level,
				}),
			));
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

			// wait for the lead gatekeeper to upload the master pubkey
			ensure!(
				gatekeepers.is_empty() || GatekeeperMasterPubkey::<T>::get().is_some(),
				Error::<T>::MasterKeyUninitialized
			);

			if !gatekeepers.contains(&gatekeeper) {
				let worker_info =
					Workers::<T>::try_get(&gatekeeper).or(Err(Error::<T>::WorkerNotFound))?;
				gatekeepers.push(gatekeeper.clone());
				let gatekeeper_count = gatekeepers.len() as u32;
				Gatekeeper::<T>::put(gatekeepers);
				Self::push_message(GatekeeperEvent::gatekeeper_registered(
					gatekeeper,
					worker_info.ecdh_pubkey,
					gatekeeper_count,
				));
			}
			Ok(())
		}

		/// Unregister a gatekeeper, must be called by gatekeeper himself
		///
		/// Requirements:
		//  1. `sig` is the valid signature of specific unregister message
		#[allow(unused_variables)]
		#[pallet::weight(0)]
		pub fn unregister_gatekeeper(
			origin: OriginFor<T>,
			gatekeeper: WorkerPublicKey,
			sig: [u8; 64],
		) -> DispatchResult {
			// TODO.shelven
			panic!("unimpleneted");
		}

		/// (called by anyone on behalf of a worker)
		#[pallet::weight(0)]
		pub fn register_worker(
			origin: OriginFor<T>,
			pruntime_info: WorkerRegistrationInfo<T::AccountId>,
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
			// Update the registry
			let pubkey = pruntime_info.pubkey.clone();
			Workers::<T>::mutate(pubkey.clone(), |v| {
				match v {
					Some(worker_info) => {
						// Case 1 - Refresh the RA report and redo benchmark
						worker_info.last_updated = now;
						Self::push_message(SystemEvent::new_worker_event(
							pubkey.clone(),
							WorkerEvent::Registered(messaging::WorkerInfo {
								confidence_level: fields.confidence_level,
							}),
						));
					}
					None => {
						// Case 2 - New worker register
						*v = Some(WorkerInfo {
							pubkey: pubkey.clone(),
							ecdh_pubkey: pruntime_info.ecdh_pubkey,
							runtime_version: pruntime_info.version,
							last_updated: now,
							operator: pruntime_info.operator,
							confidence_level: fields.confidence_level,
							initial_score: None,
							features: pruntime_info.features,
						});
						Self::push_message(SystemEvent::new_worker_event(
							pubkey.clone(),
							WorkerEvent::Registered(messaging::WorkerInfo {
								confidence_level: fields.confidence_level,
							}),
						));
					}
				}
			});
			// Trigger benchmark anyway
			let duration = BenchmarkDuration::<T>::get().unwrap_or_default();
			Self::push_message(SystemEvent::new_worker_event(
				pubkey,
				WorkerEvent::BenchStart { duration },
			));
			Ok(())
		}
	}

	// TODO.kevin: Move it to mq
	impl<T: Config> Pallet<T>
	where
		T: crate::mq::Config,
	{
		pub fn check_message(message: &SignedMessage) -> DispatchResult {
			let pubkey_copy: ContractPublicKey;
			let pubkey = match &message.message.sender {
				MessageOrigin::Worker(pubkey) => pubkey,
				MessageOrigin::Contract(id) => {
					pubkey_copy = ContractKey::<T>::get(id).ok_or(Error::<T>::UnknwonContract)?;
					&pubkey_copy
				}
				MessageOrigin::Gatekeeper => {
					// GatekeeperMasterPubkey should not be None
					pubkey_copy = GatekeeperMasterPubkey::<T>::get()
						.ok_or(Error::<T>::MasterKeyUninitialized)?;
					&pubkey_copy
				}
				_ => return Err(Error::<T>::CannotHandleUnknownMessage.into()),
			};
			Self::verify_signature(pubkey, message)
		}

		fn verify_signature(pubkey: &WorkerPublicKey, message: &SignedMessage) -> DispatchResult {
			let raw_sig = &message.signature;
			ensure!(raw_sig.len() == 64, Error::<T>::InvalidSignatureLength);
			let sig = sp_core::sr25519::Signature::try_from(raw_sig.as_slice())
				.or(Err(Error::<T>::MalformedSignature))?;
			let data = message.data_be_signed();
			ensure!(
				sp_io::crypto::sr25519_verify(&sig, &data, &pubkey),
				Error::<T>::InvalidSignature
			);
			Ok(())
		}

		pub fn on_message_received(message: DecodedMessage<RegistryEvent>) -> DispatchResult {
			let worker_pubkey = match &message.sender {
				MessageOrigin::Worker(key) => key,
				_ => return Err(Error::<T>::InvalidSender.into()),
			};

			match message.payload {
				RegistryEvent::BenchReport {
					start_time,
					iterations,
				} => {
					let now = T::UnixTime::now().as_millis().saturated_into::<u64>();
					if now <= start_time {
						// Oops, should not happen
						return Err(Error::<T>::InvalidBenchReport.into());
					}

					const MAX_SCORE: u32 = 6000;
					let score = iterations / ((now - start_time) / 1000);
					let score = score * 6; // iterations per 6s
					let score = MAX_SCORE.min(score as u32);

					Workers::<T>::mutate(worker_pubkey, |val| {
						if let Some(val) = val {
							val.initial_score = Some(score);
							val.last_updated = now;
						}
					});

					Self::push_message(SystemEvent::new_worker_event(
						worker_pubkey.clone(),
						WorkerEvent::BenchScore(score),
					));
				}
				RegistryEvent::MasterPubkey { master_pubkey } => {
					let gatekeepers = Gatekeeper::<T>::get();
					if !gatekeepers.contains(&worker_pubkey) {
						return Err(Error::<T>::InvalidGatekeeper.into());
					}

					match GatekeeperMasterPubkey::<T>::try_get() {
						Ok(saved_pubkey) => {
							ensure!(
								saved_pubkey.0 == master_pubkey.0,
								Error::<T>::MasterKeyMismatch // Oops, this is really bad
							);
						}
						_ => {
							GatekeeperMasterPubkey::<T>::put(master_pubkey);
						}
					}
				}
			}
			Ok(())
		}

		#[cfg(test)]
		pub(crate) fn internal_set_benchmark(worker: &WorkerPublicKey, score: Option<u32>) {
			Workers::<T>::mutate(worker, |w| {
				if let Some(w) = w {
					w.initial_score = score;
				}
			});
		}
	}

	// Genesis config build

	/// Genesis config to add some genesis worker or gatekeeper for testing purpose.
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		/// [(identity, ecdh, operator)]
		pub workers: Vec<(WorkerPublicKey, Vec<u8>, Option<T::AccountId>)>,
		/// [identity]
		pub gatekeepers: Vec<WorkerPublicKey>,
		pub benchmark_duration: u32,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				workers: Default::default(),
				gatekeepers: Default::default(),
				benchmark_duration: 8u32,
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T>
	where
		T: crate::mq::Config,
	{
		fn build(&self) {
			for (pubkey, ecdh_pubkey, operator) in &self.workers {
				Workers::<T>::insert(
					&pubkey,
					&WorkerInfo {
						pubkey: pubkey.clone(),
						ecdh_pubkey: ecdh_pubkey.as_slice().try_into().expect("Bad ecdh key"),
						runtime_version: 0,
						last_updated: 0,
						operator: operator.clone(),
						confidence_level: 128u8,
						initial_score: None,
						features: vec![1, 4],
					},
				);
				Pallet::<T>::queue_message(SystemEvent::new_worker_event(
					pubkey.clone(),
					WorkerEvent::Registered(messaging::WorkerInfo {
						confidence_level: 128u8,
					}),
				));
				Pallet::<T>::queue_message(SystemEvent::new_worker_event(
					pubkey.clone(),
					WorkerEvent::BenchStart {
						duration: self.benchmark_duration,
					},
				));
				BenchmarkDuration::<T>::put(self.benchmark_duration);
			}
			let mut gatekeepers: Vec<WorkerPublicKey> = Vec::new();
			for gatekeeper in &self.gatekeepers {
				match Workers::<T>::try_get(&gatekeeper) {
					Ok(worker_info) => {
						gatekeepers.push(gatekeeper.clone());
						let gatekeeper_count = gatekeepers.len() as u32;
						Gatekeeper::<T>::put(gatekeepers.clone());
						Pallet::<T>::queue_message(GatekeeperEvent::gatekeeper_registered(
							gatekeeper.clone(),
							worker_info.ecdh_pubkey,
							gatekeeper_count,
						));
					}
					_ => {}
				}
			}
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

	#[derive(Encode, Decode, Default, Debug, Clone)]
	pub struct WorkerInfo<AccountId> {
		// identity
		pubkey: WorkerPublicKey,
		ecdh_pubkey: EcdhPublicKey,
		// system
		runtime_version: u32,
		last_updated: u64,
		pub operator: Option<AccountId>,
		// platform
		pub confidence_level: u8,
		// scoring
		pub initial_score: Option<u32>,
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

/// Public key registry for workers and contracts.
pub use self::pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use codec::Encode;
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		traits::{Currency, StorageVersion, UnixTime},
	};
	use frame_system::pallet_prelude::*;
	use scale_info::TypeInfo;
	use sp_core::{sr25519, H256};
	use sp_runtime::SaturatedConversion;
	use sp_std::prelude::*;
	use sp_std::{convert::TryFrom, vec};

	use crate::attestation::{AttestationValidator, Error as AttestationError};
	use crate::mq::MessageOriginInfo;
	// Re-export
	pub use crate::attestation::{Attestation, IasValidator};

	use phala_types::{
		messaging::{
			self, bind_topic, ContractClusterId, ContractId, DecodedMessage, GatekeeperChange,
			GatekeeperLaunch, MessageOrigin, SignedMessage, SystemEvent, WorkerEvent,
		},
		ClusterPublicKey, ContractPublicKey, EcdhPublicKey, MasterPublicKey, WorkerPublicKey,
		WorkerRegistrationInfo,
	};

	bind_topic!(RegistryEvent, b"^phala/registry/event");
	#[derive(Encode, Decode, TypeInfo, Clone, Debug)]
	pub enum RegistryEvent {
		BenchReport { start_time: u64, iterations: u64 },
		MasterPubkey { master_pubkey: MasterPublicKey },
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The currency in which fees are paid and contract balances are held.
		type Currency: Currency<Self::AccountId>;

		type UnixTime: UnixTime;
		type AttestationValidator: AttestationValidator;

		/// Verify attestation, SHOULD NOT SET FALSE ON PRODUCTION !!!
		#[pallet::constant]
		type VerifyPRuntime: Get<bool>;

		/// Verify relaychain genesis, SHOULD NOT SET FALSE ON PRODUCTION !!!
		#[pallet::constant]
		type VerifyRelaychainGenesisBlockHash: Get<bool>;

		/// Origin used to administer the pallet
		type GovernanceOrigin: EnsureOrigin<Self::Origin>;
	}

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
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
	pub type ContractKeys<T> = StorageMap<_, Twox64Concat, ContractId, ContractPublicKey>;

	#[pallet::storage]
	pub type ClusterKeys<T> = StorageMap<_, Twox64Concat, ContractClusterId, ClusterPublicKey>;

	/// Pubkey for secret topics.
	#[pallet::storage]
	pub type TopicKey<T> = StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>>;

	#[pallet::storage]
	pub type BenchmarkDuration<T: Config> = StorageValue<_, u32>;

	/// Allow list of pRuntime binary digest
	///
	/// Only pRuntime within the list can register.
	#[pallet::storage]
	#[pallet::getter(fn pruntime_allowlist)]
	pub type PRuntimeAllowList<T: Config> = StorageValue<_, Vec<Vec<u8>>, ValueQuery>;

	/// Allow list of relaychain genesis
	///
	/// Only genesis within the list can do register.
	#[pallet::storage]
	#[pallet::getter(fn relaychain_genesis_allowlist)]
	pub type RelaychainGenesisBlockHashAllowList<T: Config> =
		StorageValue<_, Vec<H256>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
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
		UnknownContract,
		UnknownCluster,
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
		// GenesisBlockHash related
		GenesisBlockHashRejected,
		GenesisBlockHashAlreadyExists,
		GenesisBlockHashNotFound,
		// PRuntime related
		PRuntimeRejected,
		PRuntimeAlreadyExists,
		PRuntimeNotFound,
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
				pubkey,
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
			T::GovernanceOrigin::ensure_origin(origin)?;

			let mut gatekeepers = Gatekeeper::<T>::get();

			// wait for the lead gatekeeper to upload the master pubkey
			ensure!(
				gatekeepers.is_empty() || GatekeeperMasterPubkey::<T>::get().is_some(),
				Error::<T>::MasterKeyUninitialized
			);

			if !gatekeepers.contains(&gatekeeper) {
				let worker_info =
					Workers::<T>::get(&gatekeeper).ok_or(Error::<T>::WorkerNotFound)?;
				gatekeepers.push(gatekeeper);
				let gatekeeper_count = gatekeepers.len() as u32;
				Gatekeeper::<T>::put(gatekeepers);

				if gatekeeper_count == 1 {
					Self::push_message(GatekeeperLaunch::first_gatekeeper(
						gatekeeper,
						worker_info.ecdh_pubkey,
					));
				} else {
					Self::push_message(GatekeeperChange::gatekeeper_registered(
						gatekeeper,
						worker_info.ecdh_pubkey,
					));
				}
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
			// Validate RA report & embedded user data
			let now = T::UnixTime::now().as_secs().saturated_into::<u64>();
			let runtime_info_hash = crate::hashing::blake2_256(&Encode::encode(&pruntime_info));
			let fields = T::AttestationValidator::validate(
				&attestation,
				&runtime_info_hash,
				now,
				T::VerifyPRuntime::get(),
				PRuntimeAllowList::<T>::get(),
			)
			.map_err(Into::<Error<T>>::into)?;

			if T::VerifyRelaychainGenesisBlockHash::get() {
				let genesis_block_hash = pruntime_info.genesis_block_hash;
				let allowlist = RelaychainGenesisBlockHashAllowList::<T>::get();
				ensure!(
					allowlist.contains(&genesis_block_hash),
					Error::<T>::GenesisBlockHashRejected
				);
			}

			// Update the registry
			let pubkey = pruntime_info.pubkey;
			Workers::<T>::mutate(pubkey, |v| {
				match v {
					Some(worker_info) => {
						// Case 1 - Refresh the RA report, optionally update the operator, and redo benchmark
						worker_info.last_updated = now;
						worker_info.operator = pruntime_info.operator;
						Self::push_message(SystemEvent::new_worker_event(
							pubkey,
							WorkerEvent::Registered(messaging::WorkerInfo {
								confidence_level: fields.confidence_level,
							}),
						));
					}
					None => {
						// Case 2 - New worker register
						*v = Some(WorkerInfo {
							pubkey,
							ecdh_pubkey: pruntime_info.ecdh_pubkey,
							runtime_version: pruntime_info.version,
							last_updated: now,
							operator: pruntime_info.operator,
							confidence_level: fields.confidence_level,
							initial_score: None,
							features: pruntime_info.features,
						});
						Self::push_message(SystemEvent::new_worker_event(
							pubkey,
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

		/// Registers a pRuntime image as the canonical runtime with its digest.
		#[pallet::weight(0)]
		pub fn add_pruntime(origin: OriginFor<T>, pruntime_hash: Vec<u8>) -> DispatchResult {
			T::GovernanceOrigin::ensure_origin(origin)?;

			let mut allowlist = PRuntimeAllowList::<T>::get();
			ensure!(
				!allowlist.contains(&pruntime_hash),
				Error::<T>::PRuntimeAlreadyExists
			);

			allowlist.push(pruntime_hash);
			PRuntimeAllowList::<T>::put(allowlist);

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn remove_pruntime(origin: OriginFor<T>, pruntime_hash: Vec<u8>) -> DispatchResult {
			T::GovernanceOrigin::ensure_origin(origin)?;

			let allowlist = PRuntimeAllowList::<T>::get();
			ensure!(
				allowlist.contains(&pruntime_hash),
				Error::<T>::PRuntimeNotFound
			);

			let filtered: Vec<_> = allowlist
				.into_iter()
				.filter(|h| *h != pruntime_hash)
				.collect();
			PRuntimeAllowList::<T>::put(filtered);

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn add_relaychain_genesis_block_hash(
			origin: OriginFor<T>,
			genesis_block_hash: H256,
		) -> DispatchResult {
			T::GovernanceOrigin::ensure_origin(origin)?;

			let mut allowlist = RelaychainGenesisBlockHashAllowList::<T>::get();
			ensure!(
				!allowlist.contains(&genesis_block_hash),
				Error::<T>::GenesisBlockHashAlreadyExists
			);

			allowlist.push(genesis_block_hash);
			RelaychainGenesisBlockHashAllowList::<T>::put(allowlist);

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn remove_relaychain_genesis_block_hash(
			origin: OriginFor<T>,
			genesis_block_hash: H256,
		) -> DispatchResult {
			T::GovernanceOrigin::ensure_origin(origin)?;

			let allowlist = RelaychainGenesisBlockHashAllowList::<T>::get();
			ensure!(
				allowlist.contains(&genesis_block_hash),
				Error::<T>::GenesisBlockHashNotFound
			);

			let filtered: Vec<_> = allowlist
				.into_iter()
				.filter(|h| *h != genesis_block_hash)
				.collect();
			RelaychainGenesisBlockHashAllowList::<T>::put(filtered);

			Ok(())
		}
	}

	// TODO.kevin: Move it to mq
	impl<T: Config> Pallet<T>
	where
		T: crate::mq::Config,
	{
		pub fn check_message(message: &SignedMessage) -> DispatchResult {
			let pubkey_copy: sr25519::Public;
			let pubkey = match &message.message.sender {
				MessageOrigin::Worker(pubkey) => pubkey,
				MessageOrigin::Cluster(id) => {
					pubkey_copy = ClusterKeys::<T>::get(id).ok_or(Error::<T>::UnknownCluster)?;
					&pubkey_copy
				}
				MessageOrigin::Contract(id) => {
					pubkey_copy = ContractKeys::<T>::get(id).ok_or(Error::<T>::UnknownContract)?;
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
				sp_io::crypto::sr25519_verify(&sig, &data, pubkey),
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
						*worker_pubkey,
						WorkerEvent::BenchScore(score),
					));
				}
				RegistryEvent::MasterPubkey { master_pubkey } => {
					let gatekeepers = Gatekeeper::<T>::get();
					ensure!(
						gatekeepers.contains(worker_pubkey),
						Error::<T>::InvalidGatekeeper
					);
					match GatekeeperMasterPubkey::<T>::try_get() {
						Ok(saved_pubkey) => {
							ensure!(
								saved_pubkey.0 == master_pubkey.0,
								Error::<T>::MasterKeyMismatch // Oops, this is really bad
							);
						}
						_ => {
							GatekeeperMasterPubkey::<T>::put(master_pubkey);
							Self::push_message(GatekeeperLaunch::master_pubkey_on_chain(
								master_pubkey,
							));
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
			use std::convert::TryInto;
			for (pubkey, ecdh_pubkey, operator) in &self.workers {
				Workers::<T>::insert(
					&pubkey,
					&WorkerInfo {
						pubkey: *pubkey,
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
					*pubkey,
					WorkerEvent::Registered(messaging::WorkerInfo {
						confidence_level: 128u8,
					}),
				));
				Pallet::<T>::queue_message(SystemEvent::new_worker_event(
					*pubkey,
					WorkerEvent::BenchStart {
						duration: self.benchmark_duration,
					},
				));
				BenchmarkDuration::<T>::put(self.benchmark_duration);
			}
			let mut gatekeepers: Vec<WorkerPublicKey> = Vec::new();
			for gatekeeper in &self.gatekeepers {
				if let Ok(worker_info) = Workers::<T>::try_get(&gatekeeper) {
					gatekeepers.push(*gatekeeper);
					let gatekeeper_count = gatekeepers.len() as u32;
					Gatekeeper::<T>::put(gatekeepers.clone());
					if gatekeeper_count == 1 {
						Pallet::<T>::queue_message(GatekeeperLaunch::first_gatekeeper(
							*gatekeeper,
							worker_info.ecdh_pubkey,
						));
					} else {
						Pallet::<T>::queue_message(GatekeeperChange::gatekeeper_registered(
							*gatekeeper,
							worker_info.ecdh_pubkey,
						));
					}
				}
			}
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_runtime_upgrade() -> Weight {
			let mut w = 0;
			let old = Self::on_chain_storage_version();
			w += T::DbWeight::get().reads(1);

			if old == 0 {
				w += migrations::initialize::<T>();
				STORAGE_VERSION.put::<super::Pallet<T>>();
				w += T::DbWeight::get().writes(1);
			}
			w
		}
	}

	mod migrations {
		use super::{BenchmarkDuration, Config};
		use frame_support::pallet_prelude::*;

		pub fn initialize<T: Config>() -> Weight {
			log::info!("phala_pallet::registry: initialize()");
			BenchmarkDuration::<T>::put(50);
			T::DbWeight::get().writes(1)
		}
	}

	impl<T: Config + crate::mq::Config> MessageOriginInfo for Pallet<T> {
		type Config = T;
	}

	#[derive(Encode, Decode, TypeInfo, Debug, Clone)]
	pub struct WorkerInfo<AccountId> {
		// identity
		pub pubkey: WorkerPublicKey,
		pub ecdh_pubkey: EcdhPublicKey,
		// system
		pub runtime_version: u32,
		pub last_updated: u64,
		pub operator: Option<AccountId>,
		// platform
		pub confidence_level: u8,
		// scoring
		pub initial_score: Option<u32>,
		pub features: Vec<u32>,
	}

	impl<T: Config> From<AttestationError> for Error<T> {
		fn from(err: AttestationError) -> Self {
			match err {
				AttestationError::PRuntimeRejected => Self::PRuntimeRejected,
				AttestationError::InvalidIASSigningCert => Self::InvalidIASSigningCert,
				AttestationError::InvalidReport => Self::InvalidReport,
				AttestationError::InvalidQuoteStatus => Self::InvalidQuoteStatus,
				AttestationError::BadIASReport => Self::BadIASReport,
				AttestationError::OutdatedIASReport => Self::OutdatedIASReport,
				AttestationError::UnknownQuoteBodyFormat => Self::UnknownQuoteBodyFormat,
				AttestationError::InvalidUserDataHash => Self::InvalidRuntimeInfoHash,
			}
		}
	}

	#[cfg(test)]
	mod test {
		use frame_support::{assert_noop, assert_ok};

		use super::*;
		use crate::mock::{
			ecdh_pubkey, elapse_seconds, new_test_ext, set_block_1,
			setup_relaychain_genesis_allowlist, worker_pubkey, Origin, Test,
		};
		// Pallets
		use crate::mock::PhalaRegistry;

		#[test]
		fn test_register_worker() {
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_relaychain_genesis_allowlist();

				// New registration without valid genesis_block_hash
				assert_noop!(
					PhalaRegistry::register_worker(
						Origin::signed(1),
						WorkerRegistrationInfo::<u64> {
							version: 1,
							machine_id: Default::default(),
							pubkey: worker_pubkey(1),
							ecdh_pubkey: ecdh_pubkey(1),
							genesis_block_hash: Default::default(),
							features: vec![4, 1],
							operator: Some(1),
						},
						Attestation::SgxIas {
							ra_report: Vec::new(),
							signature: Vec::new(),
							raw_signing_cert: Vec::new(),
						}
					),
					Error::<Test>::GenesisBlockHashRejected
				);

				// New registration
				assert_ok!(PhalaRegistry::register_worker(
					Origin::signed(1),
					WorkerRegistrationInfo::<u64> {
						version: 1,
						machine_id: Default::default(),
						pubkey: worker_pubkey(1),
						ecdh_pubkey: ecdh_pubkey(1),
						genesis_block_hash: H256::repeat_byte(1),
						features: vec![4, 1],
						operator: Some(1),
					},
					Attestation::SgxIas {
						ra_report: Vec::new(),
						signature: Vec::new(),
						raw_signing_cert: Vec::new(),
					},
				));
				let worker = Workers::<Test>::get(worker_pubkey(1)).unwrap();
				assert_eq!(worker.operator, Some(1));
				// Refreshed validator
				elapse_seconds(100);
				assert_ok!(PhalaRegistry::register_worker(
					Origin::signed(1),
					WorkerRegistrationInfo::<u64> {
						version: 1,
						machine_id: Default::default(),
						pubkey: worker_pubkey(1),
						ecdh_pubkey: ecdh_pubkey(1),
						genesis_block_hash: H256::repeat_byte(1),
						features: vec![4, 1],
						operator: Some(2),
					},
					Attestation::SgxIas {
						ra_report: Vec::new(),
						signature: Vec::new(),
						raw_signing_cert: Vec::new(),
					},
				));
				let worker = Workers::<Test>::get(worker_pubkey(1)).unwrap();
				assert_eq!(worker.last_updated, 100);
				assert_eq!(worker.operator, Some(2));
			});
		}

		#[test]
		fn test_pruntime_allowlist_works() {
			new_test_ext().execute_with(|| {
				// Set block number to 1 to test the events
				set_block_1();

				let sample: Vec<u8> = [1, 2, 3, 4].to_vec();
				assert_ok!(PhalaRegistry::add_pruntime(Origin::root(), sample.clone()));
				assert_noop!(
					PhalaRegistry::add_pruntime(Origin::root(), sample.clone()),
					Error::<Test>::PRuntimeAlreadyExists
				);
				assert_eq!(PRuntimeAllowList::<Test>::get().len(), 1);
				assert_ok!(PhalaRegistry::remove_pruntime(
					Origin::root(),
					sample.clone()
				));
				assert_noop!(
					PhalaRegistry::remove_pruntime(Origin::root(), sample.clone()),
					Error::<Test>::PRuntimeNotFound
				);
				assert_eq!(PRuntimeAllowList::<Test>::get().len(), 0);
			});
		}

		#[test]
		fn test_relaychain_genesis_block_hash_allowlist_works() {
			new_test_ext().execute_with(|| {
				// Set block number to 1 to test the events
				set_block_1();

				let sample: H256 = H256::repeat_byte(1);
				assert_ok!(PhalaRegistry::add_relaychain_genesis_block_hash(
					Origin::root(),
					sample.clone()
				));
				assert_noop!(
					PhalaRegistry::add_relaychain_genesis_block_hash(
						Origin::root(),
						sample.clone()
					),
					Error::<Test>::GenesisBlockHashAlreadyExists
				);
				assert_eq!(RelaychainGenesisBlockHashAllowList::<Test>::get().len(), 1);
				assert_ok!(PhalaRegistry::remove_relaychain_genesis_block_hash(
					Origin::root(),
					sample.clone()
				));
				assert_noop!(
					PhalaRegistry::remove_relaychain_genesis_block_hash(
						Origin::root(),
						sample.clone()
					),
					Error::<Test>::GenesisBlockHashNotFound
				);
				assert_eq!(RelaychainGenesisBlockHashAllowList::<Test>::get().len(), 0);
			});
		}
	}
}

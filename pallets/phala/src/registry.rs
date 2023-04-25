//! Manages the public key of offchain components (i.e. workers and contracts)

pub use self::pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use codec::{Decode, Encode};
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

	use crate::mq::MessageOriginInfo;
	use crate::utils::attestation::Error as AttestationError;
	use phala_types::{
		messaging::{
			self, bind_topic, ContractClusterId, ContractId, DecodedMessage, GatekeeperChange,
			GatekeeperLaunch, MessageOrigin, SignedMessage, SystemEvent, WorkerEvent,
		},
		wrap_content_to_sign, AttestationProvider, ClusterPublicKey, ContractPublicKey,
		EcdhPublicKey, MasterPublicKey, SignedContentType, VersionedWorkerEndpoints,
		WorkerEndpointPayload, WorkerIdentity, WorkerPublicKey, WorkerRegistrationInfo,
		WorkerRegistrationInfoV2,
	};

	pub use phala_types::AttestationReport;
	// Re-export
	// TODO: Legacy
	pub use crate::utils::attestation_legacy::{
		Attestation, AttestationValidator, IasFields, IasValidator,
	};

	bind_topic!(RegistryEvent, b"^phala/registry/event");
	#[derive(Encode, Decode, TypeInfo, Clone, Debug)]
	pub enum RegistryEvent {
		BenchReport {
			start_time: u64,
			iterations: u64,
		},
		///	MessageOrigin::Worker -> Pallet
		///
		/// Only used for first master pubkey upload, the origin has to be worker identity since there is no master pubkey
		/// on-chain yet.
		MasterPubkey {
			master_pubkey: MasterPublicKey,
		},
	}

	bind_topic!(GatekeeperRegistryEvent, b"^phala/registry/gk_event");
	#[derive(Encode, Decode, TypeInfo, Clone, Debug)]
	pub enum GatekeeperRegistryEvent {
		RotatedMasterPubkey {
			rotation_id: u64,
			master_pubkey: MasterPublicKey,
		},
	}

	#[derive(Encode, Decode, TypeInfo, Clone, Debug, Default)]
	pub struct KnownConsensusVersion {
		version: u32,
		count: u32,
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The currency in which fees are paid and contract balances are held.
		type Currency: Currency<Self::AccountId>;

		type UnixTime: UnixTime;

		type LegacyAttestationValidator: AttestationValidator;

		/// Enable None Attestation, SHOULD BE SET TO FALSE ON PRODUCTION !!!
		#[pallet::constant]
		type NoneAttestationEnabled: Get<bool>;

		/// Verify attestation
		///
		/// SHOULD NOT SET TO FALSE ON PRODUCTION!!!
		#[pallet::constant]
		type VerifyPRuntime: Get<bool>;

		/// Verify relaychain genesis
		///
		/// SHOULD NOT SET TO FALSE ON PRODUCTION!!!
		#[pallet::constant]
		type VerifyRelaychainGenesisBlockHash: Get<bool>;

		/// Callback to get parachain id
		type ParachainId: Get<u32>;

		/// Origin used to govern the pallet
		type GovernanceOrigin: EnsureOrigin<Self::RuntimeOrigin>;
	}

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(7);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Gatekeeper pubkey list
	#[pallet::storage]
	pub type Gatekeeper<T: Config> = StorageValue<_, Vec<WorkerPublicKey>, ValueQuery>;

	/// Gatekeeper master pubkey
	#[pallet::storage]
	pub type GatekeeperMasterPubkey<T: Config> = StorageValue<_, MasterPublicKey>;

	/// The block number and unix timestamp when the gatekeeper is launched
	#[pallet::storage]
	pub type GatekeeperLaunchedAt<T: Config> = StorageValue<_, (T::BlockNumber, u64)>;

	/// The rotation counter starting from 1, it always equals to the latest rotation id.
	/// The totation id 0 is reserved for the first master key before we introduce the rotation.
	#[pallet::storage]
	pub type RotationCounter<T> = StorageValue<_, u64, ValueQuery>;

	/// Current rotation info including rotation id
	///
	/// Only one rotation process is allowed at one time.
	/// Since the rotation request is broadcasted to all gatekeepers, it should be finished only if there is one functional
	/// gatekeeper.
	#[pallet::storage]
	pub type MasterKeyRotationLock<T: Config> = StorageValue<_, Option<u64>, ValueQuery>;

	/// Mapping from worker pubkey to WorkerInfo
	#[pallet::storage]
	pub type Workers<T: Config> =
		StorageMap<_, Twox64Concat, WorkerPublicKey, WorkerInfoV2<T::AccountId>>;

	/// The first time registered block number for each worker.
	#[pallet::storage]
	pub type WorkerAddedAt<T: Config> =
		StorageMap<_, Twox64Concat, WorkerPublicKey, T::BlockNumber>;

	/// Mapping from contract address to pubkey
	#[pallet::storage]
	pub type ContractKeys<T> = StorageMap<_, Twox64Concat, ContractId, ContractPublicKey>;

	#[pallet::storage]
	pub type ClusterKeys<T> = StorageMap<_, Twox64Concat, ContractClusterId, ClusterPublicKey>;

	/// Pubkey for secret topics.
	#[pallet::storage]
	pub type TopicKey<T> = StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>>;

	/// The number of blocks to run the benchmark
	#[pallet::storage]
	pub type BenchmarkDuration<T: Config> = StorageValue<_, u32>;

	/// Allow list of pRuntime binary digest
	///
	/// Only pRuntime within the list can register.
	#[pallet::storage]
	#[pallet::getter(fn pruntime_allowlist)]
	pub type PRuntimeAllowList<T: Config> = StorageValue<_, Vec<Vec<u8>>, ValueQuery>;

	/// The effective height of pRuntime binary
	#[pallet::storage]
	pub type PRuntimeAddedAt<T: Config> = StorageMap<_, Twox64Concat, Vec<u8>, T::BlockNumber>;

	/// Allow list of relaychain genesis
	///
	/// Only genesis within the list can do register.
	#[pallet::storage]
	#[pallet::getter(fn relaychain_genesis_allowlist)]
	pub type RelaychainGenesisBlockHashAllowList<T: Config> =
		StorageValue<_, Vec<H256>, ValueQuery>;

	/// Mapping from worker pubkey to Phala Network identity
	#[pallet::storage]
	pub type Endpoints<T: Config> =
		StorageMap<_, Twox64Concat, WorkerPublicKey, VersionedWorkerEndpoints>;

	/// Allow list of pRuntime binary digest
	///
	/// Only pRuntime within the list can register.
	#[pallet::storage]
	#[pallet::getter(fn temp_workers_iter_key)]
	pub type TempWorkersIterKey<T: Config> = StorageValue<_, Option<Vec<u8>>, ValueQuery>;

	/// PRuntimes whoes version less than MinimumPRuntimeVersion would be forced to quit.
	#[pallet::storage]
	pub type MinimumPRuntimeVersion<T: Config> = StorageValue<_, (u32, u32, u32), ValueQuery>;

	/// The consensus version used by pruntime. PRuntimes would switch some code path according
	/// the current consensus version.
	#[pallet::storage]
	pub type PRuntimeConsensusVersion<T: Config> = StorageValue<_, u32, ValueQuery>;

	/// The max consensus version that pruntime has report via register_worker
	#[pallet::storage]
	pub type MaxKnownPRuntimeConsensusVersion<T: Config> =
		StorageValue<_, KnownConsensusVersion, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new Gatekeeper is enabled on the blockchain
		GatekeeperAdded {
			pubkey: WorkerPublicKey,
		},
		GatekeeperRemoved {
			pubkey: WorkerPublicKey,
		},
		WorkerAdded {
			pubkey: WorkerPublicKey,
			attestation_provider: Option<AttestationProvider>,
			confidence_level: u8,
		},
		WorkerUpdated {
			pubkey: WorkerPublicKey,
			attestation_provider: Option<AttestationProvider>,
			confidence_level: u8,
		},
		MasterKeyRotated {
			rotation_id: u64,
			master_pubkey: MasterPublicKey,
		},
		MasterKeyRotationFailed {
			rotation_lock: Option<u64>,
			gatekeeper_rotation_id: u64,
		},
		InitialScoreSet {
			pubkey: WorkerPublicKey,
			init_score: u32,
		},
		MinimumPRuntimeVersionChangedTo(u32, u32, u32),
		PRuntimeConsensusVersionChangedTo(u32),
		GatekeeperLaunched,
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
		NoneAttestationDisabled,
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
		// Additional
		UnknownCluster,
		NotImplemented,
		CannotRemoveLastGatekeeper,
		MasterKeyInRotation,
		InvalidRotatedMasterPubkey,
		// PRouter related
		InvalidEndpointSigningTime,
		/// Migration root not authorized
		NotMigrationRoot,
		ParachainIdMismatch,
		InvalidConsensusVersion,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		T: crate::mq::Config,
	{
		/// Sets [`BenchmarkDuration`]
		///
		/// Can only be called by `GovernanceOrigin`.
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(10_000u64, 0) + T::DbWeight::get().writes(1u64))]
		pub fn force_set_benchmark_duration(origin: OriginFor<T>, value: u32) -> DispatchResult {
			T::GovernanceOrigin::ensure_origin(origin)?;
			BenchmarkDuration::<T>::put(value);
			Ok(())
		}

		/// Force register a worker with the given pubkey with sudo permission
		///
		/// For test only.
		#[pallet::call_index(1)]
		#[pallet::weight(Weight::from_parts(10_000u64, 0) + T::DbWeight::get().writes(1u64))]
		pub fn force_register_worker(
			origin: OriginFor<T>,
			pubkey: WorkerPublicKey,
			ecdh_pubkey: EcdhPublicKey,
			operator: Option<T::AccountId>,
		) -> DispatchResult {
			ensure_root(origin)?;
			let worker_info = WorkerInfoV2 {
				pubkey,
				ecdh_pubkey,
				runtime_version: 0,
				last_updated: 1,
				operator,
				attestation_provider: Some(AttestationProvider::Root),
				confidence_level: 128u8,
				initial_score: None,
				features: vec![1, 4],
			};
			Workers::<T>::insert(worker_info.pubkey, &worker_info);
			WorkerAddedAt::<T>::insert(
				worker_info.pubkey,
				frame_system::Pallet::<T>::block_number(),
			);
			Self::push_message(SystemEvent::new_worker_event(
				pubkey,
				WorkerEvent::Registered(messaging::WorkerInfo {
					confidence_level: worker_info.confidence_level,
				}),
			));
			Self::deposit_event(Event::<T>::WorkerAdded {
				pubkey,
				attestation_provider: Some(AttestationProvider::Root),
				confidence_level: worker_info.confidence_level,
			});

			Ok(())
		}

		/// Force register a topic pubkey
		///
		/// For test only.
		#[pallet::call_index(2)]
		#[pallet::weight(Weight::from_parts(10_000u64, 0) + T::DbWeight::get().writes(1u64))]
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
		/// Can only be called by `GovernanceOrigin`.
		#[pallet::call_index(3)]
		#[pallet::weight(Weight::from_parts(10_000u64, 0) + T::DbWeight::get().writes(1u64))]
		pub fn register_gatekeeper(
			origin: OriginFor<T>,
			gatekeeper: WorkerPublicKey,
		) -> DispatchResult {
			T::GovernanceOrigin::ensure_origin(origin)?;

			// disable gatekeeper change during key rotation
			let rotating = MasterKeyRotationLock::<T>::get();
			ensure!(rotating.is_none(), Error::<T>::MasterKeyInRotation);

			let mut gatekeepers = Gatekeeper::<T>::get();
			// wait for the lead gatekeeper to upload the master pubkey
			ensure!(
				gatekeepers.is_empty() || GatekeeperMasterPubkey::<T>::get().is_some(),
				Error::<T>::MasterKeyUninitialized
			);

			if !gatekeepers.contains(&gatekeeper) {
				let worker_info =
					Workers::<T>::get(gatekeeper).ok_or(Error::<T>::WorkerNotFound)?;
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

			Self::deposit_event(Event::<T>::GatekeeperAdded { pubkey: gatekeeper });
			Ok(())
		}

		/// Unregister a gatekeeper
		///
		/// At least one gatekeeper should be available
		#[pallet::call_index(4)]
		#[pallet::weight(Weight::from_parts(10_000u64, 0) + T::DbWeight::get().writes(1u64))]
		pub fn unregister_gatekeeper(
			origin: OriginFor<T>,
			gatekeeper: WorkerPublicKey,
		) -> DispatchResult {
			T::GovernanceOrigin::ensure_origin(origin)?;

			// disable gatekeeper change during key rotation
			let rotating = MasterKeyRotationLock::<T>::get();
			ensure!(rotating.is_none(), Error::<T>::MasterKeyInRotation);

			let mut gatekeepers = Gatekeeper::<T>::get();
			ensure!(
				gatekeepers.contains(&gatekeeper),
				Error::<T>::InvalidGatekeeper
			);
			ensure!(
				gatekeepers.len() > 1,
				Error::<T>::CannotRemoveLastGatekeeper
			);

			gatekeepers.retain(|g| *g != gatekeeper);
			Gatekeeper::<T>::put(gatekeepers);
			Self::push_message(GatekeeperChange::gatekeeper_unregistered(gatekeeper));
			Ok(())
		}

		/// Rotate the master key
		#[pallet::call_index(5)]
		#[pallet::weight(Weight::from_parts(10_000u64, 0) + T::DbWeight::get().writes(1u64))]
		pub fn rotate_master_key(origin: OriginFor<T>) -> DispatchResult {
			T::GovernanceOrigin::ensure_origin(origin)?;

			let rotating = MasterKeyRotationLock::<T>::get();
			ensure!(rotating.is_none(), Error::<T>::MasterKeyInRotation);

			let gatekeepers = Gatekeeper::<T>::get();
			let gk_identities = gatekeepers
				.iter()
				.map(|gk| {
					let worker_info = Workers::<T>::get(gk).ok_or(Error::<T>::WorkerNotFound)?;
					Ok(WorkerIdentity {
						pubkey: worker_info.pubkey,
						ecdh_pubkey: worker_info.ecdh_pubkey,
					})
				})
				.collect::<Result<Vec<WorkerIdentity>, Error<T>>>()?;

			let rotation_id = RotationCounter::<T>::mutate(|counter| {
				*counter += 1;
				*counter
			});

			MasterKeyRotationLock::<T>::put(Some(rotation_id));
			Self::push_message(GatekeeperLaunch::rotate_master_key(
				rotation_id,
				gk_identities,
			));
			Ok(())
		}

		/// Registers a worker on the blockchain
		/// This is the legacy version that support EPID attestation type only.
		///
		/// Usually called by a bridging relayer program (`pherry` and `prb`). Can be called by
		/// anyone on behalf of a worker.
		#[pallet::call_index(6)]
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
			let fields = T::LegacyAttestationValidator::validate(
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
						worker_info.runtime_version = pruntime_info.version;
						worker_info.confidence_level = fields.confidence_level;
						worker_info.features = pruntime_info.features;
						// TODO: We should reset `initial_score` here, but we need ensure no breaking.
						// worker_info.initial_score = None;

						Self::push_message(SystemEvent::new_worker_event(
							pubkey,
							WorkerEvent::Registered(messaging::WorkerInfo {
								confidence_level: fields.confidence_level,
							}),
						));
						Self::deposit_event(Event::<T>::WorkerUpdated {
							pubkey,
							attestation_provider: Some(AttestationProvider::Ias),
							confidence_level: fields.confidence_level,
						});
					}
					None => {
						// Case 2 - New worker register
						*v = Some(WorkerInfoV2 {
							pubkey,
							ecdh_pubkey: pruntime_info.ecdh_pubkey,
							runtime_version: pruntime_info.version,
							last_updated: now,
							operator: pruntime_info.operator,
							attestation_provider: Some(AttestationProvider::Ias),
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
						Self::deposit_event(Event::<T>::WorkerAdded {
							pubkey,
							attestation_provider: Some(AttestationProvider::Ias),
							confidence_level: fields.confidence_level,
						});
						WorkerAddedAt::<T>::insert(
							pubkey,
							frame_system::Pallet::<T>::block_number(),
						);
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

		/// Registers a worker on the blockchain.
		/// This is the version 2 that both support DCAP attestation type.
		///
		/// Usually called by a bridging relayer program (`pherry` and `prb`). Can be called by
		/// anyone on behalf of a worker.
		#[pallet::call_index(7)]
		#[pallet::weight(0)]
		pub fn register_worker_v2(
			origin: OriginFor<T>,
			pruntime_info: WorkerRegistrationInfoV2<T::AccountId>,
			attestation: Option<AttestationReport>,
		) -> DispatchResult {
			ensure_signed(origin)?;
			// Validate RA report & embedded user data
			let now = T::UnixTime::now().as_secs().saturated_into::<u64>();
			let runtime_info_hash = crate::hashing::blake2_256(&Encode::encode(&pruntime_info));
			let attestation_report = crate::attestation::validate(
				attestation,
				&runtime_info_hash,
				now,
				T::VerifyPRuntime::get(),
				PRuntimeAllowList::<T>::get(),
				T::NoneAttestationEnabled::get(),
			)
			.map_err(Into::<Error<T>>::into)?;

			ensure!(
				pruntime_info.para_id == T::ParachainId::get(),
				Error::<T>::ParachainIdMismatch
			);
			if T::VerifyRelaychainGenesisBlockHash::get() {
				let genesis_block_hash = pruntime_info.genesis_block_hash;
				let allowlist = RelaychainGenesisBlockHashAllowList::<T>::get();
				ensure!(
					allowlist.contains(&genesis_block_hash),
					Error::<T>::GenesisBlockHashRejected
				);
			}

			MaxKnownPRuntimeConsensusVersion::<T>::mutate(|info| {
				use core::cmp::Ordering::*;
				match info.version.cmp(&pruntime_info.max_consensus_version) {
					Less => {
						info.version = pruntime_info.max_consensus_version;
						info.count = 1;
					}
					Equal => {
						info.count += 1;
					}
					Greater => {}
				}
			});

			// Update the registry
			let pubkey = pruntime_info.pubkey;
			Workers::<T>::mutate(pubkey, |v| {
				match v {
					Some(worker_info) => {
						// Case 1 - Refresh the RA report, optionally update the operator, and redo benchmark
						worker_info.last_updated = now;
						worker_info.operator = pruntime_info.operator;
						worker_info.runtime_version = pruntime_info.version;
						worker_info.confidence_level = attestation_report.confidence_level;
						worker_info.features = pruntime_info.features;
						// TODO: We should reset `initial_score` here, but we need ensure no breaking.
						// worker_info.initial_score = None;

						Self::push_message(SystemEvent::new_worker_event(
							pubkey,
							WorkerEvent::Registered(messaging::WorkerInfo {
								confidence_level: attestation_report.confidence_level,
							}),
						));
						Self::deposit_event(Event::<T>::WorkerUpdated {
							pubkey,
							attestation_provider: attestation_report.provider,
							confidence_level: attestation_report.confidence_level,
						});
					}
					None => {
						// Case 2 - New worker register
						*v = Some(WorkerInfoV2 {
							pubkey,
							ecdh_pubkey: pruntime_info.ecdh_pubkey,
							runtime_version: pruntime_info.version,
							last_updated: now,
							operator: pruntime_info.operator,
							attestation_provider: attestation_report.provider,
							confidence_level: attestation_report.confidence_level,
							initial_score: None,
							features: pruntime_info.features,
						});
						Self::push_message(SystemEvent::new_worker_event(
							pubkey,
							WorkerEvent::Registered(messaging::WorkerInfo {
								confidence_level: attestation_report.confidence_level,
							}),
						));
						Self::deposit_event(Event::<T>::WorkerAdded {
							pubkey,
							attestation_provider: attestation_report.provider,
							confidence_level: attestation_report.confidence_level,
						});
						WorkerAddedAt::<T>::insert(
							pubkey,
							frame_system::Pallet::<T>::block_number(),
						);
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

		#[pallet::call_index(8)]
		#[pallet::weight(0)]
		pub fn update_worker_endpoint(
			origin: OriginFor<T>,
			endpoint_payload: WorkerEndpointPayload,
			signature: Vec<u8>,
		) -> DispatchResult {
			ensure_signed(origin)?;

			// Validate the signature
			ensure!(signature.len() == 64, Error::<T>::InvalidSignatureLength);
			let sig = sp_core::sr25519::Signature::try_from(signature.as_slice())
				.or(Err(Error::<T>::MalformedSignature))?;
			let encoded_data = endpoint_payload.encode();
			let data_to_sign = wrap_content_to_sign(&encoded_data, SignedContentType::EndpointInfo);

			ensure!(
				sp_io::crypto::sr25519_verify(&sig, &data_to_sign, &endpoint_payload.pubkey),
				Error::<T>::InvalidSignature
			);

			// Validate the time
			let expiration = 4 * 60 * 60 * 1000; // 4 hours
			let now = T::UnixTime::now().as_millis().saturated_into::<u64>();
			ensure!(
				endpoint_payload.signing_time < now
					&& now <= endpoint_payload.signing_time + expiration,
				Error::<T>::InvalidEndpointSigningTime
			);

			// Validate the public key
			ensure!(
				Workers::<T>::contains_key(endpoint_payload.pubkey),
				Error::<T>::InvalidPubKey
			);

			Endpoints::<T>::insert(
				endpoint_payload.pubkey,
				endpoint_payload.versioned_endpoints,
			);

			Ok(())
		}

		/// Registers a pruntime binary to [`PRuntimeAllowList`]
		///
		/// Can only be called by `GovernanceOrigin`.
		#[pallet::call_index(9)]
		#[pallet::weight(0)]
		pub fn add_pruntime(origin: OriginFor<T>, pruntime_hash: Vec<u8>) -> DispatchResult {
			T::GovernanceOrigin::ensure_origin(origin)?;

			let mut allowlist = PRuntimeAllowList::<T>::get();
			ensure!(
				!allowlist.contains(&pruntime_hash),
				Error::<T>::PRuntimeAlreadyExists
			);

			allowlist.push(pruntime_hash.clone());
			PRuntimeAllowList::<T>::put(allowlist);

			let now = frame_system::Pallet::<T>::block_number();
			PRuntimeAddedAt::<T>::insert(&pruntime_hash, now);

			Ok(())
		}

		/// Removes a pruntime binary from [`PRuntimeAllowList`]
		///
		/// Can only be called by `GovernanceOrigin`.
		#[pallet::call_index(10)]
		#[pallet::weight(0)]
		pub fn remove_pruntime(origin: OriginFor<T>, pruntime_hash: Vec<u8>) -> DispatchResult {
			T::GovernanceOrigin::ensure_origin(origin)?;

			let mut allowlist = PRuntimeAllowList::<T>::get();
			ensure!(
				allowlist.contains(&pruntime_hash),
				Error::<T>::PRuntimeNotFound
			);

			allowlist.retain(|h| *h != pruntime_hash);
			PRuntimeAllowList::<T>::put(allowlist);

			PRuntimeAddedAt::<T>::remove(&pruntime_hash);

			Ok(())
		}

		/// Adds an entry in [`RelaychainGenesisBlockHashAllowList`]
		///
		/// Can only be called by `GovernanceOrigin`.
		#[pallet::call_index(11)]
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

		/// Deletes an entry in [`RelaychainGenesisBlockHashAllowList`]
		///
		/// Can only be called by `GovernanceOrigin`.
		#[pallet::call_index(12)]
		#[pallet::weight(0)]
		pub fn remove_relaychain_genesis_block_hash(
			origin: OriginFor<T>,
			genesis_block_hash: H256,
		) -> DispatchResult {
			T::GovernanceOrigin::ensure_origin(origin)?;

			let mut allowlist = RelaychainGenesisBlockHashAllowList::<T>::get();
			ensure!(
				allowlist.contains(&genesis_block_hash),
				Error::<T>::GenesisBlockHashNotFound
			);

			allowlist.retain(|h| *h != genesis_block_hash);
			RelaychainGenesisBlockHashAllowList::<T>::put(allowlist);

			Ok(())
		}

		/// Set minimum pRuntime version. Versions less than MinimumPRuntimeVersion would be forced to quit.
		///
		/// Can only be called by `GovernanceOrigin`.
		#[pallet::call_index(13)]
		#[pallet::weight(0)]
		pub fn set_minimum_pruntime_version(
			origin: OriginFor<T>,
			major: u32,
			minor: u32,
			patch: u32,
		) -> DispatchResult {
			T::GovernanceOrigin::ensure_origin(origin)?;
			MinimumPRuntimeVersion::<T>::put((major, minor, patch));
			Self::deposit_event(Event::<T>::MinimumPRuntimeVersionChangedTo(
				major, minor, patch,
			));
			Ok(())
		}

		/// Set the consensus version used by pruntime. PRuntimes would switch some code path according
		/// the current consensus version.
		///
		/// Can only be called by `GovernanceOrigin`.
		#[pallet::call_index(14)]
		#[pallet::weight(0)]
		pub fn set_pruntime_consensus_version(
			origin: OriginFor<T>,
			version: u32,
		) -> DispatchResult {
			T::GovernanceOrigin::ensure_origin(origin)?;
			PRuntimeConsensusVersion::<T>::put(version);
			Self::deposit_event(Event::<T>::PRuntimeConsensusVersionChangedTo(version));
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
			let data = wrap_content_to_sign(&data, SignedContentType::MqMessage);
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
					Self::deposit_event(Event::<T>::InitialScoreSet {
						pubkey: *worker_pubkey,
						init_score: score,
					});
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
							Self::on_gatekeeper_launched();
						}
					}
				}
			}
			Ok(())
		}

		pub fn on_gk_message_received(
			message: DecodedMessage<GatekeeperRegistryEvent>,
		) -> DispatchResult {
			if !message.sender.is_gatekeeper() {
				return Err(Error::<T>::InvalidSender.into());
			}

			match message.payload {
				GatekeeperRegistryEvent::RotatedMasterPubkey {
					rotation_id,
					master_pubkey,
				} => {
					let rotating = MasterKeyRotationLock::<T>::get();
					if rotating.is_none() || rotating.unwrap() != rotation_id {
						Self::deposit_event(Event::<T>::MasterKeyRotationFailed {
							rotation_lock: rotating,
							gatekeeper_rotation_id: rotation_id,
						});
						return Err(Error::<T>::InvalidRotatedMasterPubkey.into());
					}

					GatekeeperMasterPubkey::<T>::put(master_pubkey);
					MasterKeyRotationLock::<T>::put(Option::<u64>::None);
					Self::deposit_event(Event::<T>::MasterKeyRotated {
						rotation_id,
						master_pubkey,
					});
					Self::push_message(GatekeeperLaunch::master_pubkey_rotated(master_pubkey));
				}
			}
			Ok(())
		}

		fn on_gatekeeper_launched() {
			let block_number = frame_system::Pallet::<T>::block_number();
			let now = T::UnixTime::now().as_secs().saturated_into::<u64>();
			GatekeeperLaunchedAt::<T>::put((block_number, now));
			Self::deposit_event(Event::<T>::GatekeeperLaunched);
		}

		pub fn is_worker_registered_after_gk_launched(worker: &WorkerPublicKey) -> bool {
			let Some(worker) = Workers::<T>::get(worker) else {
				return false;
			};
			let Some((_, gk_launched_at)) = GatekeeperLaunchedAt::<T>::get() else {
				return false;
			};
			worker.last_updated > gk_launched_at
		}

		#[cfg(test)]
		pub(crate) fn internal_set_benchmark(worker: &WorkerPublicKey, score: Option<u32>) {
			Workers::<T>::mutate(worker, |w| {
				if let Some(w) = w {
					w.initial_score = score;
				}
			});
		}

		#[cfg(test)]
		pub(crate) fn internal_set_gk_launched_at(block: T::BlockNumber, ts: u64) {
			GatekeeperLaunchedAt::<T>::put((block, ts));
		}
	}

	// Genesis config build

	/// Genesis config to add some genesis worker or gatekeeper for testing purpose.
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		/// List of `(identity, ecdh, operator)` tuple
		pub workers: Vec<(WorkerPublicKey, Vec<u8>, Option<T::AccountId>)>,
		/// List of Gatekeeper identities
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
					pubkey,
					WorkerInfoV2 {
						pubkey: *pubkey,
						ecdh_pubkey: ecdh_pubkey.as_slice().try_into().expect("Bad ecdh key"),
						runtime_version: 0,
						last_updated: 0,
						operator: operator.clone(),
						attestation_provider: Some(AttestationProvider::Root),
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
				if let Ok(worker_info) = Workers::<T>::try_get(gatekeeper) {
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

	impl<T: Config + crate::mq::Config> MessageOriginInfo for Pallet<T> {
		type Config = T;
	}

	#[derive(Encode, Decode, TypeInfo, Debug, Clone)]
	pub struct OldWorkerInfo<AccountId> {
		/// The identity public key of the worker
		pub pubkey: WorkerPublicKey,
		/// The public key for ECDH communication
		pub ecdh_pubkey: EcdhPublicKey,
		/// The pruntime version of the worker upon registering
		pub runtime_version: u32,
		/// The unix timestamp of the last updated time
		pub last_updated: u64,
		/// The stake pool owner that can control this worker
		///
		/// When initializing pruntime, the user can specify an _operator account_. Then this field
		/// will be updated when the worker is being registered on the blockchain. Once it's set,
		/// the worker can only be added to a stake pool if the pool owner is the same as the
		/// operator. It ensures only the trusted person can control the worker.
		pub operator: Option<AccountId>,
		/// The [confidence level](https://wiki.phala.network/en-us/mine/solo/1-2-confidential-level-evaluation/#confidence-level-of-a-miner)
		/// of the worker
		pub confidence_level: u8,
		/// The performance score by benchmark
		///
		/// When a worker is registered, this field is set to `None`, indicating the worker is
		/// requested to run a benchmark. The benchmark lasts [`BenchmarkDuration`] blocks, and
		/// this field will be updated with the score once it finishes.
		///
		/// The `initial_score` is used as the baseline for mining performance test.
		pub initial_score: Option<u32>,
		/// Deprecated
		pub features: Vec<u32>,
	}

	/// The basic information of a registered worker
	#[derive(Encode, Decode, TypeInfo, Debug, Clone)]
	pub struct WorkerInfoV2<AccountId> {
		/// The identity public key of the worker
		pub pubkey: WorkerPublicKey,
		/// The public key for ECDH communication
		pub ecdh_pubkey: EcdhPublicKey,
		/// The pruntime version of the worker upon registering
		pub runtime_version: u32,
		/// The unix timestamp of the last updated time
		pub last_updated: u64,
		/// The stake pool owner that can control this worker
		///
		/// When initializing pruntime, the user can specify an _operator account_. Then this field
		/// will be updated when the worker is being registered on the blockchain. Once it's set,
		/// the worker can only be added to a stake pool if the pool owner is the same as the
		/// operator. It ensures only the trusted person can control the worker.
		pub operator: Option<AccountId>,
		/// Who issues the attestation
		pub attestation_provider: Option<AttestationProvider>,
		/// The [confidence level](https://wiki.phala.network/en-us/mine/solo/1-2-confidential-level-evaluation/#confidence-level-of-a-miner)
		/// of the worker
		pub confidence_level: u8,
		/// The performance score by benchmark
		///
		/// When a worker is registered, this field is set to `None`, indicating the worker is
		/// requested to run a benchmark. The benchmark lasts [`BenchmarkDuration`] blocks, and
		/// this field will be updated with the score once it finishes.
		///
		/// The `initial_score` is used as the baseline for mining performance test.
		pub initial_score: Option<u32>,
		/// Deprecated
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
				AttestationError::NoneAttestationDisabled => Self::NoneAttestationDisabled,
			}
		}
	}

	#[cfg(test)]
	mod test {
		use frame_support::{assert_noop, assert_ok};

		use super::*;
		use crate::mock::{
			ecdh_pubkey, elapse_seconds, new_test_ext, set_block_1,
			setup_relaychain_genesis_allowlist, worker_pubkey, RuntimeOrigin as Origin, Test,
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
		fn test_register_worker_v2() {
			new_test_ext().execute_with(|| {
				set_block_1();
				setup_relaychain_genesis_allowlist();

				// New registration without valid genesis_block_hash
				assert_noop!(
					PhalaRegistry::register_worker_v2(
						Origin::signed(1),
						WorkerRegistrationInfoV2::<u64> {
							version: 1,
							machine_id: Default::default(),
							pubkey: worker_pubkey(1),
							ecdh_pubkey: ecdh_pubkey(1),
							genesis_block_hash: Default::default(),
							para_id: 0,
							features: vec![4, 1],
							operator: Some(1),
							max_consensus_version: 0,
						},
						None
					),
					Error::<Test>::GenesisBlockHashRejected
				);

				// New registration with wrong para_id
				assert_noop!(
					PhalaRegistry::register_worker_v2(
						Origin::signed(1),
						WorkerRegistrationInfoV2::<u64> {
							version: 1,
							machine_id: Default::default(),
							pubkey: worker_pubkey(1),
							ecdh_pubkey: ecdh_pubkey(1),
							genesis_block_hash: H256::repeat_byte(1),
							para_id: 1,
							features: vec![4, 1],
							operator: Some(1),
							max_consensus_version: 0,
						},
						None
					),
					Error::<Test>::ParachainIdMismatch
				);

				// New registration
				assert_ok!(PhalaRegistry::register_worker_v2(
					Origin::signed(1),
					WorkerRegistrationInfoV2::<u64> {
						version: 1,
						machine_id: Default::default(),
						pubkey: worker_pubkey(1),
						ecdh_pubkey: ecdh_pubkey(1),
						genesis_block_hash: H256::repeat_byte(1),
						para_id: 0,
						features: vec![4, 1],
						operator: Some(1),
						max_consensus_version: 0,
					},
					None,
				));
				let worker = Workers::<Test>::get(worker_pubkey(1)).unwrap();
				assert_eq!(worker.operator, Some(1));
				// Refreshed validator
				elapse_seconds(100);
				assert_ok!(PhalaRegistry::register_worker_v2(
					Origin::signed(1),
					WorkerRegistrationInfoV2::<u64> {
						version: 1,
						machine_id: Default::default(),
						pubkey: worker_pubkey(1),
						ecdh_pubkey: ecdh_pubkey(1),
						genesis_block_hash: H256::repeat_byte(1),
						para_id: 0,
						features: vec![4, 1],
						operator: Some(2),
						max_consensus_version: 0,
					},
					None,
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
				assert!(PRuntimeAddedAt::<Test>::contains_key(&sample));
				assert_ok!(PhalaRegistry::remove_pruntime(
					Origin::root(),
					sample.clone()
				));
				assert_noop!(
					PhalaRegistry::remove_pruntime(Origin::root(), sample.clone()),
					Error::<Test>::PRuntimeNotFound
				);
				assert_eq!(PRuntimeAllowList::<Test>::get().len(), 0);
				assert!(!PRuntimeAddedAt::<Test>::contains_key(&sample));
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
					sample
				));
				assert_noop!(
					PhalaRegistry::add_relaychain_genesis_block_hash(Origin::root(), sample),
					Error::<Test>::GenesisBlockHashAlreadyExists
				);
				assert_eq!(RelaychainGenesisBlockHashAllowList::<Test>::get().len(), 1);
				assert_ok!(PhalaRegistry::remove_relaychain_genesis_block_hash(
					Origin::root(),
					sample
				));
				assert_noop!(
					PhalaRegistry::remove_relaychain_genesis_block_hash(Origin::root(), sample),
					Error::<Test>::GenesisBlockHashNotFound
				);
				assert_eq!(RelaychainGenesisBlockHashAllowList::<Test>::get().len(), 0);
			});
		}
	}
}

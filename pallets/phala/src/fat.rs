/// Public key registry for workers and contracts.
pub use self::pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use codec::Encode;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::StorageVersion};
	use frame_system::pallet_prelude::*;
	use sp_core::H256;
	use sp_runtime::traits::Hash;
	use sp_std::prelude::*;

	use crate::{mq::MessageOriginInfo, registry};
	// Re-export
	pub use crate::attestation::{Attestation, IasValidator};

	use phala_types::{
		contract::messaging::{ClusterEvent, ContractEvent, ContractOperation},
		contract::{
			ClusterInfo, ClusterPermission, CodeIndex, ContractClusterId, ContractId, ContractInfo,
		},
		messaging::{bind_topic, DecodedMessage, MessageOrigin, WorkerContractReport},
		EcdhPublicKey, WorkerIdentity, WorkerPublicKey,
	};

	bind_topic!(ClusterRegistryEvent, b"^phala/registry/cluster");
	#[derive(Encode, Decode, Clone, Debug)]
	pub enum ClusterRegistryEvent {
		PubkeyAvailable {
			cluster: ContractClusterId,
			ecdh_pubkey: EcdhPublicKey,
		},
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Mapping from an original code hash to the original code, untouched by instrumentation
	#[pallet::storage]
	pub type Code<T: Config> = StorageMap<_, Twox64Concat, CodeHash<T>, Vec<u8>>;

	#[pallet::storage]
	pub type Contracts<T: Config> =
		StorageMap<_, Twox64Concat, ContractId, ContractInfo<CodeHash<T>, T::AccountId>>;

	/// The contract cluster counter, it always equals to the latest cluster id.
	#[pallet::storage]
	pub type ClusterCounter<T> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	pub type Clusters<T: Config> =
		StorageMap<_, Twox64Concat, ContractClusterId, ClusterInfo<T::AccountId>>;

	#[pallet::storage]
	pub type ClusterWorkers<T> =
		StorageMap<_, Twox64Concat, ContractClusterId, Vec<WorkerPublicKey>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ClusterCreated {
			cluster: ContractClusterId,
		},
		ClusterPubkeyAvailable {
			cluster: ContractClusterId,
			ecdh_pubkey: EcdhPublicKey,
		},
		CodeUploaded {
			hash: CodeHash<T>,
		},
		Instantiating {
			contract: ContractId,
			cluster: ContractClusterId,
			deployer: T::AccountId,
		},
		Instantiated {
			contract: ContractId,
			cluster: ContractClusterId,
			deployer: H256,
		},
		InstantiationFailed {
			contract: ContractId,
			cluster: ContractClusterId,
			deployer: H256,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		CodeNotFound,
		ContractClusterNotFound,
		DuplicatedContract,
		DuplicatedDeployment,
		NoWorkerSpecified,
		InvalidSender,
		WorkerNotFound,
	}

	type CodeHash<T> = <T as frame_system::Config>::Hash;

	fn check_cluster_permission<T: Config>(
		deployer: &T::AccountId,
		permission: &ClusterPermission<T::AccountId>,
	) -> bool {
		match permission {
			ClusterPermission::Public => true,
			ClusterPermission::OnlyOwner(owner) => deployer == owner,
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		T: crate::mq::Config + crate::registry::Config,
		T::AccountId: AsRef<[u8]>,
	{
		#[pallet::weight(0)]
		pub fn add_cluster(
			origin: OriginFor<T>,
			permission: ClusterPermission<T::AccountId>,
			description: String,
			deploy_workers: Vec<WorkerPublicKey>,
		) -> DispatchResult {
			// for now, we only allow root account to create cluster
			ensure_root(origin.clone())?;

			ensure!(deploy_workers.len() > 0, Error::<T>::NoWorkerSpecified);
			let mut workers = Vec::new();
			for worker in &deploy_workers {
				let worker_info =
					registry::Workers::<T>::try_get(worker).or(Err(Error::<T>::WorkerNotFound))?;
				workers.push(WorkerIdentity {
					pubkey: worker_info.pubkey,
					ecdh_pubkey: worker_info.ecdh_pubkey,
				});
			}

			let origin: T::AccountId = ensure_signed(origin)?;
			let cluster_info = ClusterInfo {
				owner: origin,
				permission,
				contracts: Vec::new(),
				description,
			};

			let counter = ClusterCounter::<T>::mutate(|counter| {
				*counter += 1;
				*counter
			});
			let cluster = ContractClusterId::from_low_u64_be(counter);

			Clusters::<T>::insert(&cluster, &cluster_info);
			Self::deposit_event(Event::ClusterCreated { cluster });
			ClusterWorkers::<T>::insert(&cluster, deploy_workers);
			Self::push_message(ClusterEvent::DeployCluster { cluster, workers });
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn upload_code(origin: OriginFor<T>, code: Vec<u8>) -> DispatchResult {
			ensure_signed(origin)?;
			let hash = T::Hashing::hash(&code);
			Code::<T>::insert(&hash, &code);
			Self::deposit_event(Event::CodeUploaded { hash });
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn upload_code_to_cluster(
			origin: OriginFor<T>,
			code: Vec<u8>,
			cluster_id: ContractClusterId,
		) -> DispatchResult {
			let origin: T::AccountId = ensure_signed(origin)?;
			// TODO.shelven: check permission?
			Self::push_message(ContractOperation::UploadCodeToCluster {
				origin,
				code,
				cluster_id,
			});
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn instantiate_contract(
			origin: OriginFor<T>,
			code_index: CodeIndex<CodeHash<T>>,
			data: Vec<u8>,
			salt: Vec<u8>,
			cluster_id: ContractClusterId,
		) -> DispatchResult {
			let deployer = ensure_signed(origin)?;

			match code_index {
				CodeIndex::NativeCode(_) => {}
				CodeIndex::WasmCode(code_hash) => {
					ensure!(Code::<T>::contains_key(code_hash), Error::<T>::CodeNotFound);
				}
			}

			// We send code index instead of raw code here to reduce message size
			let contract_info = ContractInfo {
				deployer,
				code_index,
				salt,
				cluster_id,
				instantiate_data: data,
			};
			let contract_id = contract_info.contract_id(Box::new(crate::hashing::blake2_256));
			ensure!(
				!Contracts::<T>::contains_key(contract_id),
				Error::<T>::DuplicatedContract
			);
			Contracts::<T>::insert(&contract_id, &contract_info);

			Self::push_message(ContractEvent::instantiate_code(contract_info.clone()));
			Self::deposit_event(Event::Instantiating {
				contract: contract_id,
				cluster: contract_info.cluster_id,
				deployer: contract_info.deployer,
			});

			Ok(())
		}
	}

	impl<T: Config> Pallet<T>
	where
		T: crate::mq::Config + crate::registry::Config,
	{
		pub fn on_cluster_message_received(
			message: DecodedMessage<ClusterRegistryEvent>,
		) -> DispatchResult {
			ensure!(
				message.sender == MessageOrigin::Gatekeeper,
				Error::<T>::InvalidSender
			);
			match message.payload {
				ClusterRegistryEvent::PubkeyAvailable {
					cluster,
					ecdh_pubkey,
				} => {
					registry::ClusterKeys::<T>::insert(&cluster, &ecdh_pubkey);
					Self::deposit_event(Event::ClusterPubkeyAvailable {
						cluster,
						ecdh_pubkey,
					});
				}
			}
			Ok(())
		}

		pub fn on_contract_message_received(
			message: DecodedMessage<ContractRegistryEvent>,
		) -> DispatchResult {
			ensure!(
				message.sender == MessageOrigin::Gatekeeper,
				Error::<T>::InvalidSender
			);
			match message.payload {
				ContractRegistryEvent::PubkeyAvailable { contract, pubkey } => {
					registry::ContractKeys::<T>::insert(contract, pubkey);
					Self::deposit_event(Event::PubkeyAvailable { contract, pubkey });
				}
			}
			Ok(())
		}

		pub fn on_worker_contract_message_received(
			message: DecodedMessage<WorkerContractReport>,
		) -> DispatchResult {
			let _worker_pubkey = match &message.sender {
				MessageOrigin::Worker(worker_pubkey) => worker_pubkey,
				_ => return Err(Error::<T>::InvalidSender.into()),
			};
			match message.payload {
				WorkerContractReport::ContractInstantiated {
					id,
					cluster_id,
					deployer,
					pubkey: _,
				} => {
					Self::deposit_event(Event::Instantiated {
						contract: id,
						cluster: cluster_id,
						deployer,
					});
				}
				WorkerContractReport::ContractInstantiationFailed {
					id,
					cluster_id,
					deployer,
				} => {
					Self::deposit_event(Event::InstantiationFailed {
						contract: id,
						cluster: cluster_id,
						deployer,
					});
					// TODO.shelven: some cleanup?
				}
			}
			Ok(())
		}
	}

	impl<T: Config + crate::mq::Config> MessageOriginInfo for Pallet<T> {
		type Config = T;
	}
}

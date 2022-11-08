//! The Fat Contract registry

pub use self::pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use codec::Encode;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::StorageVersion};
	use frame_system::pallet_prelude::*;
	use sp_core::H256;
	use sp_runtime::AccountId32;
	use sp_std::prelude::*;

	use crate::{mq::MessageOriginInfo, registry};
	use phala_types::{
		contract::{
			messaging::{
				ClusterEvent, ClusterOperation, ContractOperation, ResourceType,
				WorkerClusterReport,
			},
			ClusterInfo, ClusterPermission, CodeIndex, ContractClusterId, ContractId, ContractInfo,
		},
		messaging::{bind_topic, DecodedMessage, MessageOrigin},
		ClusterPublicKey, ContractPublicKey, WorkerIdentity, WorkerPublicKey,
	};

	bind_topic!(ClusterRegistryEvent, b"^phala/registry/cluster");
	#[derive(Encode, Decode, Clone, Debug)]
	pub enum ClusterRegistryEvent {
		PubkeyAvailable {
			cluster: ContractClusterId,
			pubkey: ClusterPublicKey,
		},
	}

	bind_topic!(ContractRegistryEvent, b"^phala/registry/contract");
	#[derive(Encode, Decode, Clone, Debug)]
	pub enum ContractRegistryEvent {
		PubkeyAvailable {
			contract: ContractId,
			pubkey: ContractPublicKey,
			deployer: ContractId,
		},
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type InkCodeSizeLimit: Get<u32>;
		type SidevmCodeSizeLimit: Get<u32>;
	}

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(7);

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

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
	pub type ClusterContracts<T: Config> =
		StorageMap<_, Twox64Concat, ContractClusterId, Vec<ContractId>, ValueQuery>;

	#[pallet::storage]
	pub type ClusterWorkers<T> =
		StorageMap<_, Twox64Concat, ContractClusterId, Vec<WorkerPublicKey>, ValueQuery>;

	/// The pink-system contract code used to deploy new clusters
	#[pallet::storage]
	pub type PinkSystemCode<T> = StorageValue<_, (u16, Vec<u8>), ValueQuery>;
	/// The blake2_256 hash of the pink-system contract code.
	#[pallet::storage]
	pub type PinkSystemCodeHash<T> = StorageValue<_, H256, OptionQuery>;

	/// The next pink-system contract code to be applied from the next block
	#[pallet::storage]
	pub type NextPinkSystemCode<T> = StorageValue<_, Vec<u8>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ClusterCreated {
			cluster: ContractClusterId,
			system_contract: ContractId,
		},
		ClusterPubkeyAvailable {
			cluster: ContractClusterId,
			pubkey: ClusterPublicKey,
		},
		ClusterDeployed {
			cluster: ContractClusterId,
			pubkey: ClusterPublicKey,
			worker: WorkerPublicKey,
		},
		ClusterDeploymentFailed {
			cluster: ContractClusterId,
			worker: WorkerPublicKey,
		},
		Instantiating {
			contract: ContractId,
			cluster: ContractClusterId,
			deployer: T::AccountId,
		},
		ContractPubkeyAvailable {
			contract: ContractId,
			cluster: ContractClusterId,
			pubkey: ContractPublicKey,
		},
		Instantiated {
			contract: ContractId,
			cluster: ContractClusterId,
			deployer: H256,
		},
		ClusterDestroyed {
			cluster: ContractClusterId,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		CodeNotFound,
		ClusterNotFound,
		ClusterNotDeployed,
		ClusterPermissionDenied,
		DuplicatedContract,
		DuplicatedDeployment,
		NoWorkerSpecified,
		InvalidSender,
		WorkerNotFound,
		PayloadTooLarge,
		NoPinkSystemCode,
	}

	type CodeHash<T> = <T as frame_system::Config>::Hash;

	fn check_cluster_permission<T: Config>(
		deployer: &T::AccountId,
		cluster: &ClusterInfo<T::AccountId>,
	) -> bool {
		match &cluster.permission {
			ClusterPermission::Public => true,
			ClusterPermission::OnlyOwner(owner) => deployer == owner,
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		T: crate::mq::Config + crate::registry::Config,
		T: frame_system::Config<AccountId = AccountId32>,
	{
		#[pallet::weight(0)]
		pub fn add_cluster(
			origin: OriginFor<T>,
			owner: T::AccountId,
			permission: ClusterPermission<T::AccountId>,
			deploy_workers: Vec<WorkerPublicKey>,
		) -> DispatchResult {
			T::GovernanceOrigin::ensure_origin(origin)?;

			ensure!(!deploy_workers.is_empty(), Error::<T>::NoWorkerSpecified);
			let workers = deploy_workers
				.iter()
				.map(|worker| {
					let worker_info =
						registry::Workers::<T>::get(worker).ok_or(Error::<T>::WorkerNotFound)?;
					Ok(WorkerIdentity {
						pubkey: worker_info.pubkey,
						ecdh_pubkey: worker_info.ecdh_pubkey,
					})
				})
				.collect::<Result<Vec<WorkerIdentity>, Error<T>>>()?;

			let cluster_id = ClusterCounter::<T>::mutate(|counter| {
				let cluster_id = *counter;
				*counter += 1;
				cluster_id
			});
			let cluster = ContractClusterId::from_low_u64_be(cluster_id);

			let system_code_hash =
				PinkSystemCodeHash::<T>::get().ok_or(Error::<T>::NoPinkSystemCode)?;
			let selector = vec![0xed, 0x4b, 0x9d, 0x1b]; // The default() constructor
			let system_contract_info = ContractInfo {
				deployer: owner.clone(),
				code_index: CodeIndex::WasmCode(system_code_hash),
				salt: Default::default(),
				cluster_id: cluster,
				instantiate_data: selector,
			};

			let system_contract = system_contract_info.contract_id(crate::hashing::blake2_256);

			let cluster_info = ClusterInfo {
				owner: owner.clone(),
				permission,
				workers: deploy_workers,
				system_contract,
			};

			Clusters::<T>::insert(cluster, &cluster_info);
			Self::deposit_event(Event::ClusterCreated {
				cluster,
				system_contract,
			});
			Self::push_message(ClusterEvent::DeployCluster {
				owner,
				cluster,
				workers,
			});
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn cluster_upload_resource(
			origin: OriginFor<T>,
			cluster_id: ContractClusterId,
			resource_type: ResourceType,
			resource_data: Vec<u8>,
		) -> DispatchResult {
			let origin: T::AccountId = ensure_signed(origin)?;
			let cluster_info = Clusters::<T>::get(cluster_id).ok_or(Error::<T>::ClusterNotFound)?;
			ensure!(
				check_cluster_permission::<T>(&origin, &cluster_info),
				Error::<T>::ClusterPermissionDenied
			);

			let size_limit = match resource_type {
				ResourceType::InkCode => T::InkCodeSizeLimit::get(),
				ResourceType::SidevmCode => T::SidevmCodeSizeLimit::get(),
			} as usize;
			ensure!(
				resource_data.len() <= size_limit,
				Error::<T>::PayloadTooLarge
			);

			Self::push_message(ClusterOperation::<_, T::BlockNumber>::UploadResource {
				origin,
				cluster_id,
				resource_type,
				resource_data,
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
			let cluster_info = Clusters::<T>::get(cluster_id).ok_or(Error::<T>::ClusterNotFound)?;
			ensure!(
				check_cluster_permission::<T>(&deployer, &cluster_info),
				Error::<T>::ClusterPermissionDenied
			);

			let contract_info = ContractInfo {
				deployer,
				code_index,
				salt,
				cluster_id,
				instantiate_data: data,
			};
			let contract_id = contract_info.contract_id(crate::hashing::blake2_256);
			ensure!(
				!Contracts::<T>::contains_key(contract_id),
				Error::<T>::DuplicatedContract
			);
			Contracts::<T>::insert(contract_id, &contract_info);

			Self::push_message(ContractOperation::instantiate_code(contract_info.clone()));
			Self::deposit_event(Event::Instantiating {
				contract: contract_id,
				cluster: contract_info.cluster_id,
				deployer: contract_info.deployer,
			});

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn cluster_destroy(origin: OriginFor<T>, cluster: ContractClusterId) -> DispatchResult {
			ensure_root(origin)?;

			Clusters::<T>::take(cluster).ok_or(Error::<T>::ClusterNotFound)?;
			Self::push_message(
				ClusterOperation::<T::AccountId, T::BlockNumber>::DestroyCluster(cluster),
			);
			Self::deposit_event(Event::ClusterDestroyed { cluster });
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn set_pink_system_code(
			origin: OriginFor<T>,
			code: BoundedVec<u8, T::InkCodeSizeLimit>,
		) -> DispatchResult {
			T::GovernanceOrigin::ensure_origin(origin)?;
			NextPinkSystemCode::<T>::put(code);
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
				ClusterRegistryEvent::PubkeyAvailable { cluster, pubkey } => {
					// The cluster key can be over-written with the latest value by Gatekeeper
					registry::ClusterKeys::<T>::insert(cluster, pubkey);
					Self::deposit_event(Event::ClusterPubkeyAvailable { cluster, pubkey });
				}
			}
			Ok(())
		}

		pub fn on_contract_message_received(
			message: DecodedMessage<ContractRegistryEvent>,
		) -> DispatchResult {
			let cluster = match message.sender {
				MessageOrigin::Cluster(cluster) => cluster,
				_ => return Err(Error::<T>::InvalidSender.into()),
			};
			match message.payload {
				ContractRegistryEvent::PubkeyAvailable {
					contract,
					pubkey,
					deployer,
				} => {
					registry::ContractKeys::<T>::insert(contract, pubkey);
					Self::deposit_event(Event::ContractPubkeyAvailable {
						contract,
						cluster,
						pubkey,
					});
					ClusterContracts::<T>::append(cluster, contract);
					Self::deposit_event(Event::Instantiated {
						contract,
						cluster,
						deployer,
					});
				}
			}
			Ok(())
		}

		pub fn on_worker_cluster_message_received(
			message: DecodedMessage<WorkerClusterReport>,
		) -> DispatchResult {
			let worker_pubkey = match message.sender {
				MessageOrigin::Worker(worker_pubkey) => worker_pubkey,
				_ => return Err(Error::<T>::InvalidSender.into()),
			};
			match message.payload {
				WorkerClusterReport::ClusterDeployed { id, pubkey } => {
					// TODO.shelven: scalability concern for large number of workers
					ClusterWorkers::<T>::append(id, worker_pubkey);
					Self::deposit_event(Event::ClusterDeployed {
						cluster: id,
						pubkey,
						worker: worker_pubkey,
					});
				}
				WorkerClusterReport::ClusterDeploymentFailed { id } => {
					Self::deposit_event(Event::ClusterDeploymentFailed {
						cluster: id,
						worker: worker_pubkey,
					});
				}
			}
			Ok(())
		}

		pub fn get_system_contract(contract: &ContractId) -> Option<ContractId> {
			let contract_info = Contracts::<T>::get(contract)?;
			let cluster_info = Clusters::<T>::get(contract_info.cluster_id)?;
			Some(cluster_info.system_contract)
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_now: BlockNumberFor<T>) -> Weight {
			// TODO.kevin: use `let else` to early return once the next rustc released
			if let Some(next_code) = NextPinkSystemCode::<T>::take() {
				let hash: H256 = crate::hashing::blake2_256(&next_code).into();
				PinkSystemCodeHash::<T>::put(hash);
				PinkSystemCode::<T>::mutate(|(ver, code)| {
					*ver += 1;
					*code = next_code;
				});
			}
			Weight::zero()
		}
	}

	impl<T: Config + crate::mq::Config> MessageOriginInfo for Pallet<T> {
		type Config = T;
	}
}

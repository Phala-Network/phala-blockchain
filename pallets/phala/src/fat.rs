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
	use sp_std::vec;

	use crate::{mq::MessageOriginInfo, registry};
	// Re-export
	pub use crate::attestation::{Attestation, IasValidator};

	use phala_types::{
		contract::messaging::{ContractEvent, ContractOperation},
		contract::{CodeIndex, ContractClusterId, ContractId, ContractInfo},
		messaging::{bind_topic, DecodedMessage, MessageOrigin, WorkerContractReport},
		ContractPublicKey, WorkerIdentity, WorkerPublicKey,
	};

	bind_topic!(ContractRegistryEvent, b"^phala/registry/contract");
	#[derive(Encode, Decode, Clone, Debug)]
	pub enum ContractRegistryEvent {
		PubkeyAvailable {
			contract_id: ContractId,
			pubkey: ContractPublicKey,
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
	pub struct Pallet<T>(_);

	/// Mapping from an original code hash to the original code, untouched by instrumentation
	#[pallet::storage]
	pub type ContractCode<T: Config> = StorageMap<_, Twox64Concat, CodeHash<T>, Vec<u8>>;

	/// The contract cluster counter.
	#[pallet::storage]
	pub type ContractClusterCounter<T> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	pub type ContractClusters<T> = StorageMap<_, Twox64Concat, ContractClusterId, Vec<ContractId>>;

	#[pallet::storage]
	pub type Contracts<T: Config> =
		StorageMap<_, Twox64Concat, ContractId, ContractInfo<CodeHash<T>, T::AccountId>>;

	#[pallet::storage]
	pub type ContractWorkers<T> = StorageMap<_, Twox64Concat, ContractId, Vec<WorkerPublicKey>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CodeUploaded(CodeHash<T>),
		PubkeyAvailable(ContractId, ContractPublicKey),
		Instantiating(ContractId, ContractClusterId, T::AccountId),
		Instantiated(ContractId, ContractClusterId, H256),
		InstantiationFailed(ContractId, ContractClusterId, H256),
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

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		T: crate::mq::Config + crate::registry::Config,
	{
		#[pallet::weight(0)]
		pub fn upload_code(origin: OriginFor<T>, code: Vec<u8>) -> DispatchResult {
			ensure_signed(origin)?;
			let code_hash = T::Hashing::hash(&code);
			ContractCode::<T>::insert(&code_hash, &code);
			Self::deposit_event(Event::CodeUploaded(code_hash));
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
			// #[pallet::compact] endowment: BalanceOf<T>,
			// #[pallet::compact] gas_limit: Weight,
			code_index: CodeIndex<CodeHash<T>>,
			data: Vec<u8>,
			salt: Vec<u8>,
			cluster: Option<ContractClusterId>,
			deploy_workers: Vec<WorkerPublicKey>,
		) -> DispatchResult {
			let deployer = ensure_signed(origin)?;

			ensure!(deploy_workers.len() > 0, Error::<T>::NoWorkerSpecified);

			match code_index {
				CodeIndex::NativeCode(_) => {}
				CodeIndex::WasmCode(code_hash) => {
					ensure!(
						ContractCode::<T>::contains_key(code_hash),
						Error::<T>::CodeNotFound
					);
				}
			}

			let mut workers = Vec::new();
			for worker in deploy_workers.into_iter() {
				let worker_info =
					registry::Workers::<T>::try_get(&worker).or(Err(Error::<T>::WorkerNotFound))?;
				workers.push(WorkerIdentity {
					pubkey: worker_info.pubkey,
					ecdh_pubkey: worker_info.ecdh_pubkey,
				});
			}

			let cluster_id = match cluster {
				Some(cluster_id) => {
					ensure!(
						ContractClusters::<T>::contains_key(cluster_id),
						Error::<T>::ContractClusterNotFound
					);
					cluster_id
				}
				None => {
					let counter = ContractClusterCounter::<T>::mutate(|counter| {
						*counter += 1;
						*counter
					});
					ContractClusterId::from_low_u64_be(counter)
				}
			};

			// keep syncing with get_contract_id() in crates/phactory/src/contracts/mod.rs
			fn get_contract_id(
				deployer: &[u8],
				code_hash: &[u8],
				cluster_id: &[u8],
				salt: &[u8],
			) -> ContractId {
				let buf: Vec<_> = deployer
					.iter()
					.chain(code_hash)
					.chain(cluster_id)
					.chain(salt)
					.cloned()
					.collect();
				crate::hashing::blake2_256(&buf).into()
			}

			let contract_id = get_contract_id(
				deployer.encode().as_ref(),
				code_index.code_hash().as_ref(),
				cluster_id.as_ref(),
				salt.as_ref(),
			);
			ensure!(
				!Contracts::<T>::contains_key(contract_id),
				Error::<T>::DuplicatedContract
			);
			// we send code index instead of raw code here to reduce message size
			let contract_info = ContractInfo {
				deployer,
				code_index,
				salt,
				cluster_id,
				instantiate_data: data,
			};
			Contracts::<T>::insert(&contract_id, &contract_info);
			Self::push_message(ContractEvent::instantiate_code(
				contract_info.clone(),
				workers,
			));
			Self::deposit_event(Event::Instantiating(
				contract_id,
				contract_info.cluster_id,
				contract_info.deployer,
			));

			Ok(())
		}
	}

	impl<T: Config> Pallet<T>
	where
		T: crate::mq::Config + crate::registry::Config,
	{
		pub fn on_contract_message_received(
			message: DecodedMessage<ContractRegistryEvent>,
		) -> DispatchResult {
			ensure!(
				message.sender == MessageOrigin::Gatekeeper,
				Error::<T>::InvalidSender
			);
			match message.payload {
				ContractRegistryEvent::PubkeyAvailable {
					contract_id,
					pubkey,
				} => {
					registry::ContractKeys::<T>::insert(contract_id, pubkey);
					Self::deposit_event(Event::PubkeyAvailable(contract_id, pubkey));
				}
			}
			Ok(())
		}

		pub fn on_worker_contract_message_received(
			message: DecodedMessage<WorkerContractReport>,
		) -> DispatchResult {
			let worker_pubkey = match &message.sender {
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
					let mut cluster = ContractClusters::<T>::try_get(&cluster_id).unwrap_or(vec![]);
					cluster.push(id);
					ContractClusters::<T>::insert(&cluster_id, cluster);

					let mut workers = ContractWorkers::<T>::try_get(id).unwrap_or(vec![]);
					if !workers.contains(worker_pubkey) {
						workers.push(worker_pubkey.clone());
						ContractWorkers::<T>::insert(&id, workers);
					}
					Self::deposit_event(Event::Instantiated(id, cluster_id, deployer));
				}
				WorkerContractReport::ContractInstantiationFailed {
					id,
					cluster_id,
					deployer,
				} => {
					Self::deposit_event(Event::InstantiationFailed(id, cluster_id, deployer));
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

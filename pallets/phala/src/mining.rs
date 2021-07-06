pub use self::pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::Currency};
	use frame_system::pallet_prelude::*;
	use phala_types::messaging::{BlockRewardInfo, SystemEvent, Message, MessageOrigin, MiningReportEvent};
	use crate::mq::{self, MessageOriginInfo};

	#[pallet::config]
	pub trait Config: frame_system::Config + mq::Config {
		type Event: From<Event> + IsType<<Self as frame_system::Config>::Event>;

		type Currency: Currency<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// ///
	// #[pallet::storage]
	// pub type OffchainIngress<T> = StorageMap<_, Twox64Concat, MessageOrigin, u64>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event {
		/// Meh. [n]
		Meh(u32),
	}

	#[pallet::error]
	pub enum Error<T> {
		BadSender,
		InvalidMessage,
	}

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Desposits some tokan as stake
		///
		/// Requires:
		/// 1. Ther miner is in Ready state
		#[pallet::weight(0)]
		pub fn deposit(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
			panic!("unimplemented")
		}

		/// Withdraw some tokan from the stake
		///
		/// Requires:
		/// 1. Ther miner is in Ready state
		#[pallet::weight(0)]
		pub fn withdraw(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
			panic!("unimplemented")
		}

		/// Starts mining
		///
		/// Requires:
		/// 1. Ther miner is in Ready state
		#[pallet::weight(0)]
		pub fn start_mining(origin: OriginFor<T>) -> DispatchResult {
			panic!("unimplemented")
		}

		/// Stops mining, enterying cooling down state
		///
		/// Requires:
		/// 1. Ther miner is in Idle, Active, or Unresponsive state
		#[pallet::weight(0)]
		pub fn stop_mining(origin: OriginFor<T>) -> DispatchResult {
			panic!("unimplemented")
		}

		/// Turns the miner back to Ready state after cooling down
		///
		/// Requires:
		/// 1. Ther miner is in CoolingDown state and the cooling down period has passed
		#[pallet::weight(0)]
		pub fn cleanup(origin: OriginFor<T>) -> DispatchResult {
			panic!("unimplemented")
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn on_finalize(_n: T::BlockNumber) {
			Self::handle_block_reward();
		}
	}

	// TODO(wenfeng):
	// - push_message(SystemEvent::RewardSeed) regularly.
	// - push_message(SystemEvent(WorkerEvent::MiningStart)) when start mining.
	// - push_message(SystemEvent(WorkerEvent::MiningStop)) when entering CoolingDown state.
	// - push_message(SystemEvent(WorkerEvent::MiningEnterUnresponsive)) when entering MiningUnresponsive state.
	// - push_message(SystemEvent(WorkerEvent::MiningExitUnresponsive)) when recovered to MiningIdle from MiningUnresponsive state.
	// - Properly handle heartbeat message.
	impl<T: Config> Pallet<T>
	{
		fn handle_block_reward() {
			let seed_info: BlockRewardInfo = todo!("TODO(wenfeng):");
			Self::push_message(SystemEvent::RewardSeed(seed_info));
		}

		pub fn on_message_received(message: &Message) -> DispatchResult {
			let worker_pubkey = match &message.sender {
				MessageOrigin::Worker(worker) => worker,
				_ => return Err(Error::<T>::BadSender.into()),
			};

			let event: MiningReportEvent = message.decode_payload().ok_or(Error::<T>::InvalidMessage)?;
			match event {
				MiningReportEvent::Heartbeat {
					block_num,
					mining_start_time,
					iterations,
					claim_online,
					claim_compute,
				} => {
					todo!("TODO(wenfeng):");
				}
			}
			Ok(())
		}
	}

	impl<T: Config> MessageOriginInfo for Pallet<T>
	{
		type Config = T;
	}
}

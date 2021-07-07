pub use self::pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use crate::mq::{self, MessageOriginInfo};
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		traits::{Currency, Randomness},
	};
	use frame_system::pallet_prelude::*;
	use phala_types::messaging::{
		BlockRewardInfo, Message, MessageOrigin, MiningReportEvent, SystemEvent,
	};
	use sp_core::U256;
	use sp_std::cmp;

	const DEFAULT_EXPECTED_HEARTBEAT_COUNT: u32 = 20;

	#[pallet::config]
	pub trait Config: frame_system::Config + mq::Config {
		type Event: From<Event> + IsType<<Self as frame_system::Config>::Event>;

		type Currency: Currency<Self::AccountId>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Total online miners
	#[pallet::storage]
	pub type OnlineMiners<T> = StorageValue<_, u32>;

	/// The expected heartbeat count (default: 20)
	#[pallet::storage]
	pub type ExpectedHeartbeatCount<T> = StorageValue<_, u32>;

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
			Self::heartbeat_challenge();
		}
	}

	// TODO(wenfeng):
	// - push_message(SystemEvent::RewardSeed) regularly.
	// - push_message(SystemEvent(WorkerEvent::MiningStart)) when start mining.
	// - push_message(SystemEvent(WorkerEvent::MiningStop)) when entering CoolingDown state.
	// - push_message(SystemEvent(WorkerEvent::MiningEnterUnresponsive)) when entering MiningUnresponsive state.
	// - push_message(SystemEvent(WorkerEvent::MiningExitUnresponsive)) when recovered to MiningIdle from MiningUnresponsive state.
	// - Properly handle heartbeat message.
	impl<T: Config> Pallet<T> {
		fn heartbeat_challenge() {
			// Random seed for the heartbeat challenge
			let seed_hash = T::Randomness::random(crate::constants::RANDOMNESS_SUBJECT).0;
			let seed: U256 = AsRef::<[u8]>::as_ref(&seed_hash).into();
			// PoW target for the random sampling
			let online_miners = OnlineMiners::<T>::get().unwrap_or(0);
			let num_tx =
				ExpectedHeartbeatCount::<T>::get().unwrap_or(DEFAULT_EXPECTED_HEARTBEAT_COUNT);
			let online_target = pow_target(num_tx, online_miners);
			let seed_info = BlockRewardInfo {
				seed,
				online_target,
			};
			Self::push_message(SystemEvent::RewardSeed(seed_info));
		}

		pub fn on_message_received(message: &Message) -> DispatchResult {
			let worker_pubkey = match &message.sender {
				MessageOrigin::Worker(worker) => worker,
				_ => return Err(Error::<T>::BadSender.into()),
			};

			let event: MiningReportEvent =
				message.decode_payload().ok_or(Error::<T>::InvalidMessage)?;
			match event {
				MiningReportEvent::Heartbeat {
					block_num,
					mining_start_time,
					iterations,
				} => {
					todo!("TODO(wenfeng):");
				}
			}
			Ok(())
		}
	}

	fn pow_target(num_tx: u32, num_workers: u32) -> U256 {
		use substrate_fixed::types::U32F32;
		if num_workers == 0 {
			return U256::zero();
		}
		let num_workers = U32F32::from_num(num_workers);
		let num_tx = U32F32::from_num(num_tx);
		// Limit tx per block for a single miner
		//     t <= max_tx_per_hour * N/T (max_tx_per_hour = 2)
		let max_tx = num_workers * U32F32::from_num(2) / U32F32::from_num(3600 / 12);
		let target_tx = cmp::min(num_tx, max_tx);
		// Convert to U256 target
		//     target = MAX * tx / num_workers
		let frac: u32 = (target_tx / num_workers)
			.checked_shl(24)
			.expect("No overflow; qed.")
			.to_num();
		U256::MAX / (1u32 << 24) * frac
	}

	impl<T: Config> MessageOriginInfo for Pallet<T> {
		type Config = T;
	}
}

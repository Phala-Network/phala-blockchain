pub use self::pallet::*;
pub use frame_support::storage::generator::StorageMap as StorageMapTrait;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	use phala_types::messaging::{Message, MessageOrigin, SignedMessage};

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event> + IsType<<Self as frame_system::Config>::Event>;
		// config
		type QueueNotifyConfig: QueueNotifyConfig;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// The next expected sequence of a ingress message coming from a certain sender (origin)
	#[pallet::storage]
	pub type OffchainIngress<T> = StorageMap<_, Twox64Concat, MessageOrigin, u64>;

	#[pallet::event]
	// #[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event {
		/// Got an outbound message. [mesage]
		OutboundMessage(Message),
	}

	#[pallet::error]
	pub enum Error<T> {
		BadSender,
		BadSequence,
		BadDestination,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		T: crate::registry::Config,
	{
		/// Syncs an unverified offchain message to the message queue
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn sync_offchain_message(
			origin: OriginFor<T>,
			signed_message: SignedMessage,
		) -> DispatchResult {
			ensure_signed(origin)?;

			// Check sender
			let sender = &signed_message.message.sender;
			ensure!(sender.is_offchain(), Error::<T>::BadSender);

			// Check destination
			ensure!(signed_message.message.destination.is_valid(), Error::<T>::BadDestination);

			// Check ingress sequence
			let expected_seq = OffchainIngress::<T>::get(sender).unwrap_or(0);
			ensure!(
				signed_message.sequence == expected_seq,
				Error::<T>::BadSequence
			);
			// Validate signature
			crate::registry::Pallet::<T>::check_message(&signed_message)?;
			// Update ingress
			OffchainIngress::<T>::insert(sender.clone(), expected_seq + 1);
			// Call push_message
			Self::push_message(signed_message.message);
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Push a validated message to the queue
		pub fn push_message(message: Message) {
			// Notify subcribers
			T::QueueNotifyConfig::on_message_received(&message);
			// Notify the off-chain components
			if T::QueueNotifyConfig::should_push_event(&message) {
				Self::deposit_event(Event::OutboundMessage(message));
			}
		}
	}

	/// Defines the behavior of received messages.
	pub trait QueueNotifyConfig {
		/// If true, the message queue will emit an event to notify the subscribers
		fn should_push_event(message: &Message) -> bool {
			message.destination.is_offchain()
		}
		/// Handles an incoming message
		fn on_message_received(_message: &Message) {}
	}
	impl QueueNotifyConfig for () {}
}

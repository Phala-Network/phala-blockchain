//! # Off-chain Rollup Anchor
//!
//! Off-chain rollup anchor is a pallet that handles the off-chain rollup logic. It maintains a
//! kv-store for Phat Contract to read and write, and allows the Phat Contract to send arbitrary
//! messages to the blockchain to trigger custom actions.
//!
//! Off-chain Rollup enables ACID operations on the blockchain for Phat Contracts. The kv-stroe
//! access and the customzed actions are wrapped as Rollup Transactions. It guarantees that the
//! transactions are isolated and atomic. No conflicting transactions will be accepted.
//!
//! The anchor pallet is designed to implement such ACID machanism. It accepts the Rollup
//! Transaction submitted by the rollup client running in Phat Contract, validates it, and aplly
//! the changes.
//!
//! On the other hand, the pallet provides two features to the other pallets:
//!
//! 1. Push messages to the Phat Contract via the `push_message(name, content)`
//! 2. Receive messages from Phat Contract by handling `Config::OnResponse` callback trait
//!
//! ## Register a contract
//!
//! The anchor pallet allows arbitrary Phat Contract to connect to it. Before using, the Phat
//! Contract must register itself in the pallet to claim a name (in `H256`) by calling extrinsic
//! `claim_name(name)` by the _submitter account_.
//!
//! The _submitter account_ should be an account solely controlled by the Phat Contract. After
//! the name is claimed, the submitter account will be saved on the blockchain for access control.
//! The future rollup transactions must be submitted by that account. The mechanism ensures that
//! the transaction submission are genuine.
//!
//! The name can be arbitrary. However, usually it's suggested to use the contract id as the name
//! since it's unique. The name will be used to identify the connected contract and the associated
//! resources (kv-store and the queue).
//!
//! ## Outbound message queue
//!
//! The anchor pallet provides a message queue to help pass messages to the Phat Contracts:
//!
//! - `push_message(name, message)`: Push a message (arbitrary bytes) to the contract and return
//!    the id of the message. The id starts from 0.
//! - `queue_head(name)`: Return the id of the first unprocessed message
//! - `queue_tail(name)`: Return the id of the last unprocessed message
//!
//! ## Receive a message
//!
//! The anchor pallet allows Phat Contracts to send message back to the blockchain. To subscribe
//! the messages, the receiver pallet should implement the [`OnResponse`] trait.
//!
//! ```ignore
//! impl<T: Config> OnResponse for Pallet<T> {
//! 	fn on_response(name: H256, submitter: AccountId, data: Vec<u8>) -> DispatchResult {
//! 		// Check `name` and handle the message data
//! 	}
//! }
//! ```
//!
#![allow(clippy::tabs_in_doc_comments)]

pub use self::pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use crate::types::*;
	use core::fmt::Debug;
	use frame_support::{
		dispatch::DispatchResult, pallet_prelude::*, traits::StorageVersion, transactional,
	};
	use frame_system::pallet_prelude::*;
	use sp_core::H256;
	use sp_std::vec::Vec;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type OnResponse: OnResponse<Self::AccountId>;
		type QueuePrefix: Get<&'static [u8]>;
		type QueueCapacity: Get<u32>;
	}

	/// Anchor response handler trait
	pub trait OnResponse<AccountId> {
		fn on_response(name: H256, submitter: AccountId, data: Vec<u8>) -> DispatchResult;
	}
	// Default implementation
	impl<AccountId> OnResponse<AccountId> for () {
		fn on_response(_name: H256, _submitter: AccountId, _data: Vec<u8>) -> DispatchResult {
			Ok(())
		}
	}

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	/// Many-to-one mapping between names and their submitters
	#[pallet::storage]
	#[pallet::getter(fn submitter_by_names)]
	pub type SubmitterByNames<T: Config> = StorageMap<_, Blake2_128Concat, H256, T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn states)]
	pub type States<T> =
		StorageDoubleMap<_, Blake2_128Concat, H256, Blake2_128Concat, KeyBytes, ValueBytes>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A name is claimed by a submitter
		NameClaimed { submitter: T::AccountId, name: H256 },
		/// A rollup transaction is executed
		RollupExecuted {
			submitter: T::AccountId,
			name: H256,
			nonce: u128,
		},
	}

	#[pallet::error]
	#[derive(Eq, PartialEq)]
	pub enum Error<T> {
		/// A name was already claimed. You must switch to another name.
		NameAlreadyClaimed,
		/// The name doesn't exist
		NameNotExist,
		/// The operation is forbidden because it's not done by the name owner
		NotOwner,
		/// Rollup condition doesn't meet
		CondNotMet,
		/// Cannot decode the action
		FailedToDecodeAction,
		/// The queue is full.
		QueueIsFull,
		/// Trying to set an invalid queue head
		InvalidQueueHead,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Claims a name and assign the caller as the owner of the name
		///
		/// Once the name is claimed, we don't allow to change the owner or deregister any more.
		#[pallet::weight(0)]
		#[transactional]
		pub fn claim_name(origin: OriginFor<T>, name: H256) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(
				SubmitterByNames::<T>::get(name).is_none(),
				Error::<T>::NameAlreadyClaimed
			);
			SubmitterByNames::<T>::insert(name, &who);
			Self::deposit_event(Event::NameClaimed {
				submitter: who,
				name,
			});
			Ok(())
		}

		/// Triggers a rollup with an optional nonce
		#[pallet::weight(0)]
		#[transactional]
		pub fn rollup(
			origin: OriginFor<T>,
			name: H256,
			tx: RollupTx,
			nonce: u128,
		) -> DispatchResult {
			// Check submitter
			let who = ensure_signed(origin)?;
			Self::ensure_name_owner(&name, &who)?;
			// Check conditions
			for cond in tx.conds {
				let Cond::Eq(key, opt_value) = cond;
				ensure!(
					States::<T>::get(name, key) == opt_value,
					Error::<T>::CondNotMet
				);
			}
			// Apply updates
			for (key, opt_value) in tx.updates {
				if let Some(v) = opt_value {
					States::<T>::insert(name, key, v);
				} else {
					States::<T>::remove(name, key);
				}
			}
			// Exec actions
			for raw_act in tx.actions {
				let act: Action =
					Decode::decode(&mut &raw_act[..]).or(Err(Error::<T>::FailedToDecodeAction))?;
				match act {
					Action::Reply(data) => {
						T::OnResponse::on_response(name, who.clone(), data.into())?
					}
					Action::QueueHeadSet(head) => {
						Self::queue_head_set(&name, head)?;
					}
				}
			}
			Self::deposit_event(Event::RollupExecuted {
				submitter: who,
				name,
				nonce,
			});
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Cheks the name is owned by the caller
		fn ensure_name_owner(name: &H256, caller: &T::AccountId) -> DispatchResult {
			let owner = SubmitterByNames::<T>::get(name).ok_or(Error::<T>::NameNotExist)?;
			ensure!(&owner == caller, Error::<T>::NotOwner);
			Ok(())
		}

		/// Pushes a message to the target rollup instance by `name`
		///
		/// Returns the index of the message if succeeded
		pub fn push_message(name: &H256, data: ValueBytes) -> Result<u32, Error<T>> {
			ensure!(
				SubmitterByNames::<T>::contains_key(name),
				Error::<T>::NameNotExist
			);
			ensure!(
				Self::queue_len(name) < T::QueueCapacity::get(),
				Error::<T>::QueueIsFull
			);
			let end = Self::queue_tail(name);
			Self::queue_set(name, &end, data);
			Self::queue_tail_set(name, end + 1);
			Ok(end)
		}

		/// Returns the position of the message queue head element
		///
		/// When `queue_head() == queue_tail()`, the queue is empty.
		pub fn queue_head(name: &H256) -> u32 {
			Self::queue_get_u32(name, b"_head").expect("BUG: Failed to decode queue head")
		}

		/// Returns the position of the message queue tail element
		///
		/// When `queue_head() == queue_tail()`, the queue is empty.
		pub fn queue_tail(name: &H256) -> u32 {
			Self::queue_get_u32(name, b"_tail").expect("BUG: Failed to decode queue tail")
		}

		/// Returns number of elements in the queue
		pub fn queue_len(name: &H256) -> u32 {
			Self::queue_tail(name).saturating_sub(Self::queue_head(name))
		}
	}

	/// Private helper methods
	impl<T: Config> Pallet<T> {
		fn queue_get_u32(name: &H256, index: &[u8; 5]) -> Result<u32, impl Debug> {
			let Some(bytes) = Self::queue_get(name, index) else {
				return Ok(0);
			};
			u32::decode(&mut &bytes[..])
		}

		fn queue_set_u32(name: &H256, index: &[u8; 5], value: u32) {
			let value = value
				.encode()
				.try_into()
				.expect("BUG: Failed to encode u32");
			Self::queue_set(name, index, value);
		}

		fn queue_head_set(name: &H256, index: u32) -> DispatchResult {
			let head = Self::queue_head(name);
			let tail = Self::queue_tail(name);
			ensure!(head <= index && index <= tail, Error::<T>::InvalidQueueHead);
			for i in head..index {
				Self::queue_remove(name, &i);
			}
			Self::queue_set_u32(name, b"_head", index);
			Ok(())
		}

		fn queue_tail_set(name: &H256, index: u32) {
			Self::queue_set_u32(name, b"_tail", index)
		}

		fn queue_key(index: &impl Encode) -> KeyBytes {
			let mut key = T::QueuePrefix::get().to_vec();
			index.encode_to(&mut key);
			key.try_into()
				.expect("BUG: Failed to make the queue key, the prefix might be too long")
		}

		fn queue_set(name: &H256, index: &impl Encode, data: ValueBytes) {
			States::<T>::insert(name, Self::queue_key(index), data);
		}

		fn queue_get(name: &H256, index: &impl Encode) -> Option<ValueBytes> {
			States::<T>::get(name, Self::queue_key(index))
		}

		fn queue_remove(name: &H256, index: &impl Encode) {
			States::<T>::remove(name, Self::queue_key(index))
		}
	}

	#[cfg(test)]
	mod test {
		use super::*;
		use crate::mock::{
			bvec, new_test_ext, set_block_1, take_events, Anchor, RuntimeEvent,
			RuntimeOrigin as Origin, Test,
		};
		// Pallets
		use frame_support::{assert_noop, assert_ok};

		const NAME1: H256 = H256([1u8; 32]);

		#[test]
		fn rollup_works() {
			new_test_ext().execute_with(|| {
				set_block_1();
				assert_ok!(Anchor::claim_name(Origin::signed(1), NAME1));

				// Can apply updates
				assert_ok!(Anchor::rollup(
					Origin::signed(1),
					NAME1,
					RollupTx {
						conds: vec![],
						actions: vec![],
						updates: vec![(bvec(b"key"), Some(bvec(b"value")))],
					},
					1u128
				));
				assert_eq!(Anchor::states(NAME1, bvec(b"key")), Some(bvec(b"value")));

				// Condition check can work
				assert_ok!(Anchor::rollup(
					Origin::signed(1),
					NAME1,
					RollupTx {
						conds: vec![Cond::Eq(bvec(b"key"), Some(bvec(b"value")))],
						actions: vec![],
						updates: vec![(bvec(b"key"), Some(bvec(b"new-value")))],
					},
					2u128
				));
				assert_eq!(
					Anchor::states(NAME1, bvec(b"key")),
					Some(bvec(b"new-value"))
				);

				// Reject conflicting tx
				assert_noop!(
					Anchor::rollup(
						Origin::signed(1),
						NAME1,
						RollupTx {
							conds: vec![Cond::Eq(bvec(b"key"), Some(bvec(b"value")))],
							actions: vec![],
							updates: vec![],
						},
						3u128
					),
					Error::<Test>::CondNotMet
				);

				// Delete update
				assert_ok!(Anchor::rollup(
					Origin::signed(1),
					NAME1,
					RollupTx {
						conds: vec![],
						actions: vec![],
						updates: vec![(bvec(b"key"), None)],
					},
					4u128
				));
				assert_eq!(Anchor::states(NAME1, bvec(b"key")), None);

				// Action received
				let resposne = crate::oracle::ResponseRecord {
					owner: sp_runtime::AccountId32::from([0u8; 32]),
					contract_id: NAME1,
					pair: bvec(b"polkadot_usd"),
					price: 5_000000000000,
					timestamp_ms: 1000,
				};
				let act = Action::Reply(bvec(&resposne.encode()));
				let _ = take_events();
				assert_ok!(Anchor::rollup(
					Origin::signed(1),
					NAME1,
					RollupTx {
						conds: vec![],
						actions: vec![bvec(&act.encode())],
						updates: vec![],
					},
					5u128
				));
				assert_eq!(
					take_events(),
					vec![
						RuntimeEvent::Oracle(crate::oracle::Event::<Test>::QuoteReceived {
							contract: NAME1,
							submitter: 1,
							owner: sp_runtime::AccountId32::from([0u8; 32]),
							pair: bvec(b"polkadot_usd"),
							price: 5000000000000,
						}),
						RuntimeEvent::Anchor(crate::anchor::Event::<Test>::RollupExecuted {
							submitter: 1,
							name: NAME1,
							nonce: 5,
						}),
					]
				);
			});
		}

		#[test]
		fn name_cannot_claim_twice() {
			new_test_ext().execute_with(|| {
				set_block_1();
				assert_ok!(Anchor::claim_name(Origin::signed(1), NAME1));
				assert_noop!(
					Anchor::claim_name(Origin::signed(2), NAME1),
					Error::<Test>::NameAlreadyClaimed
				);
			});
		}

		#[test]
		fn queue_key_is_correct() {
			assert_eq!(&Anchor::queue_key(&1)[..], b"_q/\x01\x00\x00\x00");
			assert_eq!(
				&Anchor::queue_key(b"_head")[..],
				[95, 113, 47, 95, 104, 101, 97, 100]
			);
		}

		#[test]
		fn queue_works() {
			new_test_ext().execute_with(|| {
				set_block_1();
				assert_ok!(Anchor::claim_name(Origin::signed(1), NAME1));

				assert_eq!(Anchor::queue_head(&NAME1), 0);
				assert_eq!(Anchor::queue_tail(&NAME1), 0);
				assert_eq!(Anchor::queue_len(&NAME1), 0);

				assert_eq!(Anchor::push_message(&NAME1, bvec(b"foo")), Ok(0));
				assert_eq!(Anchor::queue_head(&NAME1), 0);
				assert_eq!(Anchor::queue_tail(&NAME1), 1);
				assert_eq!(Anchor::queue_len(&NAME1), 1);

				let cap = <<Test as Config>::QueueCapacity as Get<_>>::get();
				for i in 1..cap {
					assert_eq!(Anchor::push_message(&NAME1, bvec(b"foo")), Ok(i));
				}
				assert_eq!(Anchor::queue_head(&NAME1), 0);
				assert_eq!(Anchor::queue_tail(&NAME1), cap);
				assert_eq!(Anchor::queue_len(&NAME1), cap);
				assert_eq!(
					Anchor::push_message(&NAME1, bvec(b"foo")),
					Err(Error::<Test>::QueueIsFull)
				);
				assert_eq!(Anchor::queue_head(&NAME1), 0);
				assert_eq!(Anchor::queue_tail(&NAME1), cap);
				assert_eq!(Anchor::queue_len(&NAME1), cap);

				for i in 0..cap {
					assert!(Anchor::queue_get(&NAME1, &i).is_some());
				}

				// Pop all elements except the last one
				assert_ok!(Anchor::rollup(
					Origin::signed(1),
					NAME1,
					RollupTx {
						conds: vec![],
						actions: vec![bvec(&Action::QueueHeadSet(cap - 1).encode())],
						updates: vec![],
					},
					1u128
				));

				assert_eq!(Anchor::queue_head(&NAME1), cap - 1);
				assert_eq!(Anchor::queue_tail(&NAME1), cap);
				assert_eq!(Anchor::queue_len(&NAME1), 1);
				for i in 0..cap - 1 {
					assert!(Anchor::queue_get(&NAME1, &i).is_none());
				}
				let last = cap - 1;
				assert!(Anchor::queue_get(&NAME1, &last).is_some());

				// Pop the last one
				assert_ok!(Anchor::rollup(
					Origin::signed(1),
					NAME1,
					RollupTx {
						conds: vec![],
						actions: vec![bvec(&Action::QueueHeadSet(cap).encode())],
						updates: vec![],
					},
					1u128
				));
				assert_eq!(Anchor::queue_len(&NAME1), 0);

				// Pop more should fail
				assert_noop!(
					Anchor::rollup(
						Origin::signed(1),
						NAME1,
						RollupTx {
							conds: vec![],
							actions: vec![bvec(&Action::QueueHeadSet(cap + 1).encode())],
							updates: vec![],
						},
						1u128
					),
					Error::<Test>::InvalidQueueHead
				);
			});
		}

		// TODO: test cases
		//
		// fn rollup_bad_cond
		// fn rollup_bad_action
	}
}

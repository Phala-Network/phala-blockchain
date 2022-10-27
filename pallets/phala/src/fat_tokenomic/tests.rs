use super::*;
use frame_support::{assert_err, assert_ok};
use mock::{RuntimeOrigin as Origin, Test, DOLLARS};

use sp_core::crypto::AccountId32;
use sp_core::H256;

mod mock;

const ALICE: AccountId32 = AccountId32::new([1u8; 32]);
const BOB: AccountId32 = AccountId32::new([2u8; 32]);
const CONTRACT: H256 = H256([42u8; 32]);

macro_rules! stake {
	($user: expr, $amount: expr) => {
		Pallet::<Test>::adjust_stake(Origin::signed($user), CONTRACT, $amount)
	};
}

fn stake_of_contract() -> u128 {
	ContractTotalStakes::<Test>::get(CONTRACT)
}

fn stake_of_user(user: &AccountId32) -> u128 {
	ContractUserStakes::<Test>::get(user, CONTRACT)
}

fn balance_of_user(user: &AccountId32) -> u128 {
	mock::System::account(user).data.free
}

fn prapare() {
	mock::System::set_block_number(1);
	mock::Balances::set_balance(Origin::root(), ALICE, 100 * DOLLARS, 0).unwrap();
	mock::Balances::set_balance(Origin::root(), BOB, 100 * DOLLARS, 0).unwrap();
	MinStake::<Test>::put(DOLLARS);
}

#[test]
fn should_be_happy_to_stake() {
	mock::new_test_ext().execute_with(|| {
		prapare();
		assert_ok!(stake!(ALICE, DOLLARS));
		assert_eq!(stake_of_user(&ALICE), DOLLARS);
		assert_eq!(stake_of_user(&BOB), 0);
		assert_eq!(stake_of_contract(), DOLLARS);
		assert_eq!(balance_of_user(&ALICE), 99 * DOLLARS);

		assert_ok!(stake!(BOB, 2 * DOLLARS));
		assert_eq!(stake_of_user(&ALICE), DOLLARS);
		assert_eq!(stake_of_user(&BOB), 2 * DOLLARS);
		assert_eq!(stake_of_contract(), 3 * DOLLARS);
		assert_eq!(balance_of_user(&BOB), 98 * DOLLARS);

		assert_ok!(stake!(BOB, 0));
		assert_ok!(stake!(ALICE, 0));
		assert_eq!(stake_of_user(&ALICE), 0);
		assert_eq!(stake_of_user(&BOB), 0);
		assert_eq!(stake_of_contract(), 0);
		assert_eq!(balance_of_user(&ALICE), 100 * DOLLARS);
		assert_eq!(balance_of_user(&BOB), 100 * DOLLARS);

		let events = mock::take_events();
		insta::assert_debug_snapshot!(events);
	});
}

#[test]
fn can_not_stake_less_than_minstake() {
	mock::new_test_ext().execute_with(|| {
		prapare();
		assert_err!(
			stake!(ALICE, DOLLARS - 1),
			Error::<Test>::InvalidAmountOfStake
		);
	});
}

#[test]
fn can_restake_without_any_changes() {
	mock::new_test_ext().execute_with(|| {
		prapare();

		assert_ok!(stake!(ALICE, DOLLARS));
		assert_eq!(stake_of_user(&ALICE), DOLLARS);
		assert_eq!(stake_of_contract(), DOLLARS);
		assert_eq!(balance_of_user(&ALICE), 99 * DOLLARS);

		let events = mock::take_events();
		insta::assert_debug_snapshot!(events);

		assert_ok!(stake!(ALICE, DOLLARS));
		assert_eq!(stake_of_user(&ALICE), DOLLARS);
		assert_eq!(stake_of_contract(), DOLLARS);
		assert_eq!(balance_of_user(&ALICE), 99 * DOLLARS);

		let events = mock::take_events();
		insta::assert_debug_snapshot!(events);
	});
}

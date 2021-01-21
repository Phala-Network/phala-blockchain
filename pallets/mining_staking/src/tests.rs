use frame_support::{assert_ok, assert_noop, traits::Currency};
use pallet_balances::Error as BalancesError;

use crate::{Error, mock::*};

#[test]
fn test_group_by() {
	let double_map: Vec<(u32, u32, u32)> = vec![
		(10, 100, 1),
		(10, 101, 2),
		(11, 110, 3),
		(12, 120, 4),
	];

	let mut group_key = Vec::<u32>::new();
	let mut group_size = Vec::<usize>::new();
	crate::group_by_key(double_map.iter().cloned(), |k1, group| {
		group_key.push(*k1);
		group_size.push(group.len());
	});

	assert_eq!(group_key, vec![10, 11, 12]);
	assert_eq!(group_size, vec![2, 1, 1]);
}

#[test]
fn test_deposit_withdraw() {
	new_test_ext().execute_with(|| {
		let imbalance = Balances::deposit_creating(&1, 100);
		drop(imbalance);
		// Deposit too much token
		assert_noop!(
			MiningStaking::deposit(Origin::signed(1), 101),
			BalancesError::<Test, _>::InsufficientBalance);
		// Deposit some token
		assert_ok!(MiningStaking::deposit(Origin::signed(1), 100));
		assert_eq!(MiningStaking::wallet(1), 100);
		// Withdraw too much token
		assert_noop!(
			MiningStaking::withdraw(Origin::signed(1), 101),
			Error::<Test>::InsufficientFunds);
		// Withdraw som token
		assert_ok!(MiningStaking::withdraw(Origin::signed(1), 100));
		assert_eq!(MiningStaking::wallet(1), 0);
	});
}

fn setup_deposit() {
	let imbalance = Balances::deposit_creating(&1, 100);
	drop(imbalance);
	assert_ok!(MiningStaking::deposit(Origin::signed(1), 100));
}

#[test]
fn test_stake() {
	new_test_ext().execute_with(|| {
		setup_deposit();
		// Stake 50 to 2 in total
		assert_ok!(MiningStaking::stake(Origin::signed(1), 2, 30));
		assert_eq!(MiningStaking::pending_staking(1, 2), 30);
		assert_ok!(MiningStaking::stake(Origin::signed(1), 2, 20));
		assert_eq!(MiningStaking::pending_staking(1, 2), 50);
		// Stake 30 to 3
		assert_ok!(MiningStaking::stake(Origin::signed(1), 3, 30));
		assert_eq!(MiningStaking::pending_staking(1, 3), 30);
		// 20 remains available
		assert_eq!(MiningStaking::available(&1), 20);
		assert_eq!(MiningStaking::wallet_locked(1), 80);
		// Stake more than we have (stake 31 to 4)
		assert_noop!(
			MiningStaking::stake(Origin::signed(1), 4, 21),
			Error::<Test>::InsufficientFunds);
		// Cancel some staking
		assert_ok!(MiningStaking::unstake(Origin::signed(1), 3, 10));
		assert_eq!(MiningStaking::pending_staking(1, 3), 20);
		assert_eq!(MiningStaking::wallet_locked(1), 70);
		// Apply the pending staking
		MiningStaking::handle_round_end();
		assert_eq!(MiningStaking::wallet(1), 30);
		assert_eq!(MiningStaking::staked(1, 2), 50);
		assert_eq!(MiningStaking::staked(1, 3), 20);
		assert_eq!(MiningStaking::stake_received(2), 50);
		assert_eq!(MiningStaking::stake_received(3), 20);
	});
}

#[test]
fn test_unstake() {
	new_test_ext().execute_with(|| {
		setup_deposit();
		assert_ok!(MiningStaking::stake(Origin::signed(1), 2, 50));
		assert_ok!(MiningStaking::stake(Origin::signed(1), 3, 30));
		MiningStaking::handle_round_end();
		// Unstake 20 to 3 in total
		assert_ok!(MiningStaking::unstake(Origin::signed(1), 3, 10));
		assert_ok!(MiningStaking::unstake(Origin::signed(1), 3, 10));
		assert_eq!(MiningStaking::pending_unstaking(1, 3), 20);
		// Cancel some unstaking
		assert_ok!(MiningStaking::stake(Origin::signed(1), 3, 10));
		assert_eq!(MiningStaking::pending_unstaking(1, 3), 10);
		assert_eq!(MiningStaking::pending_staking(1, 3), 0);
		assert_eq!(MiningStaking::wallet_locked(1), 0);
		// Unstake too much
		assert_noop!(
			MiningStaking::unstake(Origin::signed(1), 2, 51), Error::<Test>::InsufficientStake);
		// Apply the pending unstaking
		MiningStaking::handle_round_end();
		assert_eq!(MiningStaking::wallet(1), 30);
		assert_eq!(MiningStaking::staked(1, 2), 50);
		assert_eq!(MiningStaking::staked(1, 3), 20);
		assert_eq!(MiningStaking::stake_received(2), 50);
		assert_eq!(MiningStaking::stake_received(3), 20);
	});
}

use super::*;
use crate::{mock::*, pallet::Error};
use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;
use hex_literal::hex;

#[test]
fn store_erc20_burned_transactions_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(pallet::EndHeight::<Test>::get(), None);

		assert_ok!(PhaClaim::change_relayer(RawOrigin::Root.into(), 1));
		let tx_hash = EthereumTxHash(hex![
			"9c944a0627a9032489f0673246d1f272032f47d29857500ddb7e891c1287079a"
		]);
		let eth_addr = EthereumAddress(hex!["9863a44303d29120b00d2d9ea6344751d9ad9ccc"]);
		assert_ok!(PhaClaim::store_erc20_burned_transactions(
			Origin::signed(1),
			10000,
			vec![(tx_hash.clone(), eth_addr.clone(), 100)]
		));
		assert_eq!(
			pallet::BurnedTransactions::<Test>::get(&tx_hash),
			Some((eth_addr, 100))
		);
		assert_eq!(pallet::EndHeight::<Test>::get(), Some(10000));
	});
}

#[test]
fn store_erc20_burned_transactions_no_relayer() {
	new_test_ext().execute_with(|| {
		let tx_hash = EthereumTxHash(hex![
			"9c944a0627a9032489f0673246d1f272032f47d29857500ddb7e891c1287079a"
		]);
		let eth_addr = EthereumAddress(hex!["9863a44303d29120b00d2d9ea6344751d9ad9ccc"]);
		assert_noop!(
			PhaClaim::store_erc20_burned_transactions(
				Origin::signed(1),
				10000,
				vec![(tx_hash, eth_addr, 100)]
			),
			Error::<Test>::NoRelayer
		);
	});
}

#[test]
fn store_erc20_burned_transactions_wrong_relayer() {
	new_test_ext().execute_with(|| {
		assert_ok!(PhaClaim::change_relayer(RawOrigin::Root.into(), 1));
		let tx_hash = EthereumTxHash(hex![
			"9c944a0627a9032489f0673246d1f272032f47d29857500ddb7e891c1287079a"
		]);
		let eth_addr = EthereumAddress(hex!["9863a44303d29120b00d2d9ea6344751d9ad9ccc"]);
		assert_noop!(
			PhaClaim::store_erc20_burned_transactions(
				Origin::signed(2),
				10000,
				vec![(tx_hash, eth_addr, 100)]
			),
			Error::<Test>::CallerNotRelayer
		);
	});
}

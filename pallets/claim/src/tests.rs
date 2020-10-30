use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use super::*;
use hex_literal::hex;

#[test]
fn store_erc20_burned_transaction_works() {
    new_test_ext().execute_with(|| {
		let tx_hash = hex!["9c944a0627a9032489f0673246d1f272032f47d29857500ddb7e891c1287079a"];
		let tx_hash = EthereumTxHash(tx_hash);
		let eth_addr = hex!["9863a44303d29120b00d2d9ea6344751d9ad9ccc"];
		let eth_addr = EthereumAddress(eth_addr);
		assert_ok!(ClaimModule::store_erc20_burned_transaction(Origin::signed(1), 10000, tx_hash.clone(), eth_addr.clone(), 100));
    	assert_eq!(BurnedTransactions::get(&tx_hash), (eth_addr, 100))
	});
}

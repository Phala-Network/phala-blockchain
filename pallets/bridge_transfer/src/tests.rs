#![cfg(test)]

use super::mock::{
	assert_events, balances, event_exists, expect_event, new_test_ext, Balances, Bridge,
	BridgeTransfer, Call, Event, Origin, ProposalLifetime, Test, ENDOWED_BALANCE, RELAYER_A,
	RELAYER_B, RELAYER_C,
};
use super::*;
use frame_support::dispatch::DispatchError;
use frame_support::{assert_noop, assert_ok};

use codec::Encode;

use phala_types::messaging::Lottery;

const TEST_THRESHOLD: u32 = 2;

fn make_generic_proposal(metada: Vec<u8>) -> Call {
	let resource_id = BridgeTransfer::bridge_lotteryid();
	Call::BridgeTransfer(crate::Call::lottery_handler(metada, resource_id))
}

fn make_transfer_proposal(to: u64, amount: u64) -> Call {
	let resource_id = BridgeTransfer::bridge_lotteryid();
	Call::BridgeTransfer(crate::Call::transfer(to, amount.into(), resource_id))
}

#[test]
fn lottery_output() {
	new_test_ext().execute_with(|| {
		let dest_chain = 0;
		let resource_id = BridgeTransfer::bridge_lotteryid();
		let lottery: Lottery = Lottery::BtcAddresses {
			address_set: vec![b"123456".to_vec()].to_vec(),
		};

		assert_ok!(Bridge::set_threshold(Origin::root(), TEST_THRESHOLD,));

		assert_ok!(Bridge::whitelist_chain(Origin::root(), dest_chain.clone()));
		assert_ok!(BridgeTransfer::lottery_output(&lottery, dest_chain,));

		expect_event(bridge::RawEvent::GenericTransfer(
			dest_chain,
			1,
			resource_id,
			lottery.encode(),
		));
	})
}

#[test]
fn transfer_native() {
	new_test_ext().execute_with(|| {
		let dest_chain = 0;
		let resource_id = BridgeTransfer::bridge_tokenid();
		let amount: u64 = 100;
		let recipient = vec![99];

		assert_ok!(Bridge::whitelist_chain(Origin::root(), dest_chain.clone()));
		assert_ok!(BridgeTransfer::transfer_native(
			Origin::signed(RELAYER_A),
			amount.clone(),
			recipient.clone(),
			dest_chain,
		));

		expect_event(bridge::RawEvent::FungibleTransfer(
			dest_chain,
			1,
			resource_id,
			amount.into(),
			recipient,
		));
	})
}

#[test]
fn execute_lottery_handler_bad_origin() {
	new_test_ext().execute_with(|| {
		let payload: Vec<u8> = hex::decode("00000000010000000100000001").unwrap();
		let resource_id = BridgeTransfer::bridge_lotteryid();
		assert_ok!(BridgeTransfer::lottery_handler(
			Origin::signed(Bridge::account_id()),
			payload.clone(),
			resource_id
		));
		// Don't allow any signed origin except from bridge addr
		assert_noop!(
			BridgeTransfer::lottery_handler(
				Origin::signed(RELAYER_A),
				payload.clone(),
				resource_id
			),
			DispatchError::BadOrigin
		);
		// Don't allow root calls
		assert_noop!(
			BridgeTransfer::lottery_handler(Origin::root(), payload, resource_id),
			DispatchError::BadOrigin
		);
	})
}

#[test]
fn transfer() {
	new_test_ext().execute_with(|| {
		// Check inital state
		let bridge_id: u64 = Bridge::account_id();
		let resource_id = BridgeTransfer::bridge_lotteryid();
		assert_eq!(Balances::free_balance(&bridge_id), ENDOWED_BALANCE);
		// Transfer and check result
		assert_ok!(BridgeTransfer::transfer(
			Origin::signed(Bridge::account_id()),
			RELAYER_A,
			10,
			resource_id,
		));
		assert_eq!(Balances::free_balance(&bridge_id), ENDOWED_BALANCE - 10);
		assert_eq!(Balances::free_balance(RELAYER_A), ENDOWED_BALANCE + 10);

		assert_events(vec![Event::Balances(balances::Event::Transfer(
			Bridge::account_id(),
			RELAYER_A,
			10,
		))]);
	})
}

#[test]
fn create_sucessful_transfer_proposal() {
	new_test_ext().execute_with(|| {
		let prop_id = 1;
		let src_id = 1;
		let r_id = bridge::derive_resource_id(src_id, b"transfer");
		let resource = b"BridgeTransfer.transfer".to_vec();
		let proposal = make_transfer_proposal(RELAYER_A, 10);

		assert_ok!(Bridge::set_threshold(Origin::root(), TEST_THRESHOLD,));
		assert_ok!(Bridge::add_relayer(Origin::root(), RELAYER_A));
		assert_ok!(Bridge::add_relayer(Origin::root(), RELAYER_B));
		assert_ok!(Bridge::add_relayer(Origin::root(), RELAYER_C));
		assert_ok!(Bridge::whitelist_chain(Origin::root(), src_id));
		assert_ok!(Bridge::set_resource(Origin::root(), r_id, resource));

		// Create proposal (& vote)
		assert_ok!(Bridge::acknowledge_proposal(
			Origin::signed(RELAYER_A),
			prop_id,
			src_id,
			r_id,
			Box::new(proposal.clone())
		));
		let prop = Bridge::votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
		let expected = bridge::ProposalVotes {
			votes_for: vec![RELAYER_A],
			votes_against: vec![],
			status: bridge::ProposalStatus::Initiated,
			expiry: ProposalLifetime::get() + 1,
		};
		assert_eq!(prop, expected);

		// Second relayer votes against
		assert_ok!(Bridge::reject_proposal(
			Origin::signed(RELAYER_B),
			prop_id,
			src_id,
			r_id,
			Box::new(proposal.clone())
		));
		let prop = Bridge::votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
		let expected = bridge::ProposalVotes {
			votes_for: vec![RELAYER_A],
			votes_against: vec![RELAYER_B],
			status: bridge::ProposalStatus::Initiated,
			expiry: ProposalLifetime::get() + 1,
		};
		assert_eq!(prop, expected);

		// Third relayer votes in favour
		assert_ok!(Bridge::acknowledge_proposal(
			Origin::signed(RELAYER_C),
			prop_id,
			src_id,
			r_id,
			Box::new(proposal.clone())
		));
		let prop = Bridge::votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
		let expected = bridge::ProposalVotes {
			votes_for: vec![RELAYER_A, RELAYER_C],
			votes_against: vec![RELAYER_B],
			status: bridge::ProposalStatus::Approved,
			expiry: ProposalLifetime::get() + 1,
		};
		assert_eq!(prop, expected);

		assert_eq!(Balances::free_balance(RELAYER_A), ENDOWED_BALANCE + 10);
		assert_eq!(
			Balances::free_balance(Bridge::account_id()),
			ENDOWED_BALANCE - 10
		);

		assert_events(vec![
			Event::Bridge(bridge::RawEvent::VoteFor(src_id, prop_id, RELAYER_A)),
			Event::Bridge(bridge::RawEvent::VoteAgainst(src_id, prop_id, RELAYER_B)),
			Event::Bridge(bridge::RawEvent::VoteFor(src_id, prop_id, RELAYER_C)),
			Event::Bridge(bridge::RawEvent::ProposalApproved(src_id, prop_id)),
			Event::Balances(balances::Event::Transfer(
				Bridge::account_id(),
				RELAYER_A,
				10,
			)),
			Event::Bridge(bridge::RawEvent::ProposalSucceeded(src_id, prop_id)),
		]);
	})
}

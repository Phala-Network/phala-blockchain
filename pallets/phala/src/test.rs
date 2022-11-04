use crate::basepool;
use crate::computation;
use crate::pawnshop;
use crate::poolproxy::*;
use crate::stakepoolv2;
use crate::vault;
use fixed::types::U64F64 as FixedPoint;
use frame_support::{
	assert_noop, assert_ok,
	pallet_prelude::Get,
	traits::tokens::fungibles::{Create, Inspect},
};
use hex_literal::hex;
use sp_runtime::{testing::H256, AccountId32};

use crate::mock::{
	ecdh_pubkey, elapse_cool_down, elapse_seconds, new_test_ext, set_block_1, setup_workers,
	setup_workers_linked_operators, worker_pubkey, Balance, RuntimeOrigin, Test, DOLLARS,
};
// Pallets
use crate::mock::{
	Balances, PhalaBasePool, PhalaComputation, PhalaPawnshop, PhalaRegistry, PhalaStakePoolv2,
	PhalaVault,
};
use pallet_democracy::AccountVote;
use phala_types::{messaging::SettleInfo, WorkerPublicKey};
use rmrk_traits::primitives::NftId;
use sp_runtime::Permill;
use sp_std::collections::vec_deque::VecDeque;

#[test]
fn test_pool_subaccount() {
	let sub_account: AccountId32 =
		stakepoolv2::pool_sub_account(1, &WorkerPublicKey::from_raw([0u8; 32]));
	let expected = AccountId32::new(hex!(
		"73706d2f02ab4d74c86ec3b3997a4fadf33e55e8279650c8539ea67e053c02dc"
	));
	assert_eq!(sub_account, expected, "Incorrect sub account");
}

#[test]
fn test_pawn() {
	new_test_ext().execute_with(|| {
		mock_asset_id();
		let free = Balances::free_balance(&<Test as pawnshop::Config>::PawnShopAccountId::get());
		assert_eq!(free, 0);
		let free = Balances::free_balance(1);
		assert_eq!(free, 1000 * DOLLARS);
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(1), 100 * DOLLARS));
		let free = Balances::free_balance(&<Test as pawnshop::Config>::PawnShopAccountId::get());
		assert_eq!(free, 100 * DOLLARS);
		let free = Balances::free_balance(1);
		assert_eq!(free, 900 * DOLLARS);
		let ppha_free = get_balance(1);
		assert_eq!(ppha_free, 100 * DOLLARS);
	});
}

#[test]
fn test_redeem() {
	new_test_ext().execute_with(|| {
		mock_asset_id();
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(1), 100 * DOLLARS));
		assert_ok!(PhalaPawnshop::redeem(RuntimeOrigin::signed(1), 50 * DOLLARS,));
		let free = Balances::free_balance(1);
		assert_eq!(free, 950 * DOLLARS);
		let free = Balances::free_balance(&<Test as pawnshop::Config>::PawnShopAccountId::get());
		assert_eq!(free, 50 * DOLLARS);
		let ppha_free = get_balance(1);
		assert_eq!(ppha_free, 50 * DOLLARS);
		pawnshop::pallet::StakerAccounts::<Test>::insert(
			1,
			pawnshop::FinanceAccount::<u128> {
				invest_pools: vec![],
				locked: 20 * DOLLARS,
			},
		);
		assert_noop!(
			PhalaPawnshop::redeem(RuntimeOrigin::signed(1), 50 * DOLLARS),
			pawnshop::Error::<Test>::RedeemAmountExceedsAvaliableStake
		);
	});
}

#[test]
fn test_redeem_all() {
	new_test_ext().execute_with(|| {
		mock_asset_id();
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(1), 100 * DOLLARS));
		set_block_1();
		setup_workers(2);
		setup_stake_pool_with_workers(1, &[1, 2]); // pid = 0
		assert_ok!(PhalaStakePoolv2::contribute(
			RuntimeOrigin::signed(1),
			0,
			50 * DOLLARS,
			None
		));
		let free = Balances::free_balance(2);
		assert_eq!(free, 2000 * DOLLARS);
		assert_ok!(PhalaPawnshop::redeem_all(RuntimeOrigin::signed(1)));
		let free = Balances::free_balance(1);
		assert_eq!(free, 950 * DOLLARS);
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(2), 100 * DOLLARS));
		assert_ok!(PhalaPawnshop::redeem_all(RuntimeOrigin::signed(2)));
		let free = Balances::free_balance(2);
		assert_eq!(free, 2000 * DOLLARS);
	});
}

#[test]
fn test_vote() {
	new_test_ext().execute_with(|| {
		mock_asset_id();
		let vote_id = pallet_democracy::pallet::Pallet::<Test>::internal_start_referendum(
			H256::zero(),
			pallet_democracy::VoteThreshold::SimpleMajority,
			1000,
		);
		let vote_id2 = pallet_democracy::pallet::Pallet::<Test>::internal_start_referendum(
			H256::zero(),
			pallet_democracy::VoteThreshold::SimpleMajority,
			1000,
		);
		assert_eq!(vote_id, 0);
		assert_eq!(vote_id2, 1);
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(1), 100 * DOLLARS));
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(2), 100 * DOLLARS));
		assert_noop!(
			PhalaPawnshop::vote(RuntimeOrigin::signed(1), 90 * DOLLARS, 90 * DOLLARS, 0),
			pawnshop::Error::<Test>::VoteAmountLargerThanTotalStakes,
		);
		assert_ok!(PhalaPawnshop::vote(
			RuntimeOrigin::signed(1),
			20 * DOLLARS,
			10 * DOLLARS,
			0
		));
		let account1_status = pawnshop::pallet::StakerAccounts::<Test>::get(1).unwrap();
		assert_eq!(account1_status.locked, 30 * DOLLARS);
		assert_ok!(PhalaPawnshop::vote(
			RuntimeOrigin::signed(1),
			40 * DOLLARS,
			20 * DOLLARS,
			1
		));
		let account1_status = pawnshop::pallet::StakerAccounts::<Test>::get(1).unwrap();
		assert_eq!(account1_status.locked, 60 * DOLLARS);
		assert_ok!(PhalaPawnshop::vote(
			RuntimeOrigin::signed(2),
			20 * DOLLARS,
			30 * DOLLARS,
			0
		));
		let vote = PhalaPawnshop::accumulate_account_vote(0);
		let (aye, nay) = match vote {
			AccountVote::Split { aye, nay } => (aye, nay),
			_ => panic!(),
		};
		assert_eq!(aye, 40 * DOLLARS);
		assert_eq!(nay, 40 * DOLLARS);
		assert_ok!(PhalaPawnshop::vote(
			RuntimeOrigin::signed(1),
			5 * DOLLARS,
			10 * DOLLARS,
			1
		));
		let account1_status = pawnshop::pallet::StakerAccounts::<Test>::get(1).unwrap();
		assert_eq!(account1_status.locked, 30 * DOLLARS);
	});
}

#[test]
fn test_unlock() {
	new_test_ext().execute_with(|| {
		mock_asset_id();
		let _vote_id = pallet_democracy::pallet::Pallet::<Test>::internal_start_referendum(
			H256::zero(),
			pallet_democracy::VoteThreshold::SimpleMajority,
			1000,
		);
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(1), 100 * DOLLARS));
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(2), 100 * DOLLARS));
		assert_ok!(PhalaPawnshop::vote(
			RuntimeOrigin::signed(1),
			20 * DOLLARS,
			10 * DOLLARS,
			0
		));
		assert_ok!(PhalaPawnshop::vote(
			RuntimeOrigin::signed(2),
			20 * DOLLARS,
			10 * DOLLARS,
			0
		));
		assert_noop!(
			PhalaPawnshop::unlock(RuntimeOrigin::signed(3), 0, 1),
			pawnshop::Error::<Test>::ReferendumOngoing,
		);
		pallet_democracy::pallet::Pallet::<Test>::internal_cancel_referendum(0);
		assert_ok!(PhalaPawnshop::unlock(RuntimeOrigin::signed(3), 0, 1));
		let account1_status = pawnshop::pallet::StakerAccounts::<Test>::get(1).unwrap();
		assert_eq!(account1_status.locked, 0 * DOLLARS);
		let account2_status = pawnshop::pallet::StakerAccounts::<Test>::get(2).unwrap();
		assert_eq!(account2_status.locked, 30 * DOLLARS);
		assert_ok!(PhalaPawnshop::unlock(RuntimeOrigin::signed(3), 0, 2));
		let account2_status = pawnshop::pallet::StakerAccounts::<Test>::get(2).unwrap();
		assert_eq!(account2_status.locked, 0 * DOLLARS);
		let vote_id = pallet_democracy::pallet::Pallet::<Test>::internal_start_referendum(
			H256::zero(),
			pallet_democracy::VoteThreshold::SimpleMajority,
			1000,
		);
		assert_ok!(PhalaPawnshop::vote(
			RuntimeOrigin::signed(1),
			20 * DOLLARS,
			10 * DOLLARS,
			vote_id
		));
		assert_ok!(PhalaPawnshop::vote(
			RuntimeOrigin::signed(2),
			20 * DOLLARS,
			10 * DOLLARS,
			vote_id
		));
		pallet_democracy::pallet::Pallet::<Test>::internal_cancel_referendum(1);
		assert_ok!(PhalaPawnshop::unlock(RuntimeOrigin::signed(3), 1, 2));
		let account1_status = pawnshop::pallet::StakerAccounts::<Test>::get(1).unwrap();
		assert_eq!(account1_status.locked, 0 * DOLLARS);
		let account2_status = pawnshop::pallet::StakerAccounts::<Test>::get(2).unwrap();
		assert_eq!(account2_status.locked, 0 * DOLLARS);
	});
}

#[test]
fn test_mint_nft() {
	new_test_ext().execute_with(|| {
		set_block_1();
		setup_workers(2);
		setup_stake_pool_with_workers(1, &[1, 2]); // pid = 0
		let pool_info = ensure_stake_pool::<Test>(0).unwrap();
		assert_ok!(PhalaBasePool::mint_nft(
			pool_info.basepool.cid,
			1,
			1000 * DOLLARS,
		));
		assert_ok!(PhalaBasePool::mint_nft(
			pool_info.basepool.cid,
			2,
			500 * DOLLARS,
		));
		{
			assert_ok!(PhalaBasePool::get_nft_attr_guard(pool_info.basepool.cid, 0));
			assert_noop!(
				PhalaBasePool::get_nft_attr_guard(pool_info.basepool.cid, 0),
				basepool::Error::<Test>::AttrLocked
			);
		}
		let nft_attr = PhalaBasePool::get_nft_attr_guard(pool_info.basepool.cid, 0)
			.unwrap()
			.attr
			.clone();
		assert_eq!(nft_attr.shares, 1000 * DOLLARS);
		let nft_attr = PhalaBasePool::get_nft_attr_guard(pool_info.basepool.cid, 1)
			.unwrap()
			.attr
			.clone();
		assert_eq!(nft_attr.shares, 500 * DOLLARS);
	});
}

#[test]
fn test_merge_or_init_nft() {
	new_test_ext().execute_with(|| {
		set_block_1();
		setup_workers(2);
		setup_stake_pool_with_workers(1, &[1, 2]); // pid = 0
		let pool_info = ensure_stake_pool::<Test>(0).unwrap();
		assert_ok!(PhalaBasePool::mint_nft(
			pool_info.basepool.cid,
			1,
			1000 * DOLLARS,
		));
		assert_ok!(PhalaBasePool::mint_nft(
			pool_info.basepool.cid,
			1,
			2000 * DOLLARS,
		));
		let nftid_arr: Vec<NftId> = pallet_rmrk_core::Nfts::<Test>::iter_key_prefix(0).collect();
		assert_eq!(nftid_arr.len(), 2);
		assert_ok!(PhalaBasePool::merge_or_init_nft_for_staker(
			pool_info.basepool.cid,
			1
		));
		let nftid_arr: Vec<NftId> = pallet_rmrk_core::Nfts::<Test>::iter_key_prefix(0).collect();
		assert_eq!(nftid_arr.len(), 1);
		{
			let nft_attr = PhalaBasePool::get_nft_attr_guard(pool_info.basepool.cid, nftid_arr[0])
				.unwrap()
				.attr
				.clone();
			assert_eq!(nft_attr.shares, 3000 * DOLLARS);
		}
		assert_ok!(PhalaBasePool::merge_or_init_nft_for_staker(
			pool_info.basepool.cid,
			2
		));
		let mut nftid_arr: Vec<NftId> =
			pallet_rmrk_core::Nfts::<Test>::iter_key_prefix(0).collect();
		nftid_arr.retain(|x| {
			let nft = pallet_rmrk_core::Nfts::<Test>::get(0, x).unwrap();
			nft.owner == rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(2)
		});
		assert_eq!(nftid_arr.len(), 1);
		{
			let nft_attr = PhalaBasePool::get_nft_attr_guard(pool_info.basepool.cid, nftid_arr[0])
				.unwrap()
				.attr
				.clone();
			assert_eq!(nft_attr.shares, 0 * DOLLARS);
		}
	});
}

#[test]
fn test_set_nft_attr() {
	new_test_ext().execute_with(|| {
		set_block_1();
		setup_workers(2);
		setup_stake_pool_with_workers(1, &[1, 2]); // pid = 0
		let pool_info = ensure_stake_pool::<Test>(0).unwrap();
		assert_ok!(PhalaBasePool::mint_nft(
			pool_info.basepool.cid,
			1,
			1000 * DOLLARS,
		));
		{
			let mut nft_attr_guard =
				PhalaBasePool::get_nft_attr_guard(pool_info.basepool.cid, 0).unwrap();
			let mut nft_attr = nft_attr_guard.attr.clone();
			nft_attr.shares = 5000 * DOLLARS;
			nft_attr_guard.attr = nft_attr;
			assert_ok!(nft_attr_guard.save());
		}
		{
			let nft_attr = PhalaBasePool::get_nft_attr_guard(pool_info.basepool.cid, 0)
				.unwrap()
				.attr
				.clone();
			assert_eq!(nft_attr.shares, 5000 * DOLLARS);
		}
	});
}

#[test]
fn test_remove_stake_from_nft() {
	new_test_ext().execute_with(|| {
		mock_asset_id();
		set_block_1();
		setup_workers(2);
		setup_stake_pool_with_workers(1, &[1, 2]); // pid = 0
		let _pool_info = ensure_stake_pool::<Test>(0).unwrap();
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(1), 100 * DOLLARS));
		assert_ok!(PhalaStakePoolv2::contribute(
			RuntimeOrigin::signed(1),
			0,
			50 * DOLLARS,
			None,
		));
		let mut nftid_arr: Vec<NftId> =
			pallet_rmrk_core::Nfts::<Test>::iter_key_prefix(0).collect();
		nftid_arr.retain(|x| {
			let nft = pallet_rmrk_core::Nfts::<Test>::get(0, x).unwrap();
			nft.owner == rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(1)
		});
		assert_eq!(nftid_arr.len(), 1);
		let mut pool = ensure_stake_pool::<Test>(0).unwrap();
		let mut nft_attr = PhalaBasePool::get_nft_attr_guard(pool.basepool.cid, nftid_arr[0])
			.unwrap()
			.attr
			.clone();
		assert_eq!(pool.basepool.share_price().unwrap(), 1);
		match PhalaBasePool::remove_stake_from_nft(
			&mut pool.basepool,
			40 * DOLLARS,
			&mut nft_attr,
			&1,
		) {
			Some((_amount, _removed_shares)) => return,
			_ => panic!(),
		}
	});
}

#[test]
fn test_create_stakepool() {
	new_test_ext().execute_with(|| {
		set_block_1();
		assert_ok!(PhalaStakePoolv2::create(RuntimeOrigin::signed(1)));
		assert_ok!(PhalaStakePoolv2::create(RuntimeOrigin::signed(2)));
		assert_eq!(
			basepool::Pools::<Test>::get(0),
			Some(PoolProxy::<u64, Balance>::StakePool(StakePool::<
				u64,
				Balance,
			> {
				basepool: basepool::BasePool {
					pid: 0,
					owner: 1,
					total_shares: 0,
					total_value: 0,
					withdraw_queue: VecDeque::new(),
					value_subscribers: VecDeque::new(),
					cid: 0,
					pool_account_id: 16637257129592320098,
				},
				payout_commission: None,
				lock_account: 16637257129592319091,
				cap: None,
				workers: VecDeque::new(),
				cd_workers: VecDeque::new(),
				owner_reward_account: 16637257129592319859,
			})),
		);
		assert_eq!(basepool::PoolCount::<Test>::get(), 2);
	});
}

#[test]
fn test_create_vault() {
	new_test_ext().execute_with(|| {
		set_block_1();
		assert_ok!(PhalaVault::create(RuntimeOrigin::signed(1)));
		assert_ok!(PhalaVault::create(RuntimeOrigin::signed(2)));
		assert_eq!(
			basepool::Pools::<Test>::get(0),
			Some(PoolProxy::Vault(Vault::<u64, Balance> {
				basepool: basepool::BasePool {
					pid: 0,
					owner: 1,
					total_shares: 0,
					total_value: 0,
					withdraw_queue: VecDeque::new(),
					value_subscribers: VecDeque::new(),
					cid: 0,
					pool_account_id: 16637257129592320098,
				},
				last_share_price_checkpoint: 0,
				commission: None,
				owner_shares: 0,
				invest_pools: VecDeque::new(),
			})),
		);
		assert_eq!(basepool::PoolCount::<Test>::get(), 2);
	});
}

#[test]
fn test_contribute() {
	new_test_ext().execute_with(|| {
		mock_asset_id();
		set_block_1();
		setup_workers(2);
		setup_stake_pool_with_workers(1, &[1, 2]); // pid = 0
		assert_noop!(
			PhalaStakePoolv2::contribute(RuntimeOrigin::signed(1), 0, 50 * DOLLARS, None,),
			stakepoolv2::Error::<Test>::InsufficientBalance,
		);

		let pool = ensure_stake_pool::<Test>(0).unwrap();
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(1), 500 * DOLLARS));
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(2), 500 * DOLLARS));
		assert_ok!(PhalaStakePoolv2::contribute(
			RuntimeOrigin::signed(1),
			0,
			50 * DOLLARS,
			None
		));
		assert_ok!(PhalaStakePoolv2::contribute(
			RuntimeOrigin::signed(2),
			0,
			50 * DOLLARS,
			None
		));
		assert_ok!(PhalaStakePoolv2::contribute(
			RuntimeOrigin::signed(1),
			0,
			30 * DOLLARS,
			None
		));

		let mut nftid_arr: Vec<NftId> =
			pallet_rmrk_core::Nfts::<Test>::iter_key_prefix(0).collect();
		nftid_arr.retain(|x| {
			let nft = pallet_rmrk_core::Nfts::<Test>::get(0, x).unwrap();
			nft.owner == rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(1)
		});
		assert_eq!(nftid_arr.len(), 1);
		{
			let nft_attr = PhalaBasePool::get_nft_attr_guard(pool.basepool.cid, nftid_arr[0])
				.unwrap()
				.attr
				.clone();
			assert_eq!(nft_attr.shares, 80 * DOLLARS);
		}
		let mut nftid_arr: Vec<NftId> =
			pallet_rmrk_core::Nfts::<Test>::iter_key_prefix(0).collect();
		nftid_arr.retain(|x| {
			let nft = pallet_rmrk_core::Nfts::<Test>::get(0, x).unwrap();
			nft.owner == rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(2)
		});
		assert_eq!(nftid_arr.len(), 1);
		{
			let nft_attr = PhalaBasePool::get_nft_attr_guard(pool.basepool.cid, nftid_arr[0])
				.unwrap()
				.attr
				.clone();
			assert_eq!(nft_attr.shares, 50 * DOLLARS);
		}
		let pool = ensure_stake_pool::<Test>(0).unwrap();
		assert_eq!(pool.basepool.total_shares, 130 * DOLLARS);
		assert_eq!(pool.basepool.total_value, 130 * DOLLARS);
		let free = get_balance(pool.basepool.pool_account_id);
		assert_eq!(free, 130 * DOLLARS);

		assert_ok!(PhalaVault::create(RuntimeOrigin::signed(1)));
		assert_ok!(PhalaVault::contribute(RuntimeOrigin::signed(2), 1, 200 * DOLLARS,));
		let pool = ensure_vault::<Test>(1).unwrap();
		assert_eq!(pool.basepool.total_shares, 200 * DOLLARS);
		assert_eq!(pool.basepool.total_value, 200 * DOLLARS);
		let free = get_balance(pool.basepool.pool_account_id);
		assert_eq!(free, 200 * DOLLARS);

		assert_noop!(
			PhalaStakePoolv2::contribute(RuntimeOrigin::signed(2), 0, 10 * DOLLARS, Some(1),),
			stakepoolv2::Error::<Test>::UnauthorizedPoolOwner,
		);
		assert_ok!(PhalaStakePoolv2::contribute(
			RuntimeOrigin::signed(1),
			0,
			100 * DOLLARS,
			Some(1)
		));
		let pool = ensure_stake_pool::<Test>(0).unwrap();
		assert_eq!(pool.basepool.total_shares, 230 * DOLLARS);
		assert_eq!(pool.basepool.total_value, 230 * DOLLARS);
		let mut buf = VecDeque::new();
		buf.push_back(1);
		assert_eq!(pool.basepool.value_subscribers, buf);
		let pool = ensure_vault::<Test>(1).unwrap();
		assert_eq!(pool.basepool.total_shares, 200 * DOLLARS);
		assert_eq!(pool.basepool.total_value, 200 * DOLLARS);
		let free = get_balance(pool.basepool.pool_account_id);
		assert_eq!(free, 100 * DOLLARS);
		let mut buf = VecDeque::new();
		buf.push_back(0);
		assert_eq!(pool.invest_pools, buf);
	});
}

#[test]
fn test_set_pool_description() {
	new_test_ext().execute_with(|| {
		set_block_1();
		setup_workers(1);
		setup_stake_pool_with_workers(1, &[1]);
		let str_hello: basepool::DescStr = ("hello").as_bytes().to_vec().try_into().unwrap();
		assert_ok!(PhalaBasePool::set_pool_description(
			RuntimeOrigin::signed(1),
			0,
			str_hello.clone(),
		));
		let list = PhalaBasePool::pool_descriptions(0).unwrap();
		assert_eq!(list, str_hello);
		let str_bye: basepool::DescStr = ("bye").as_bytes().to_vec().try_into().unwrap();
		assert_noop!(
			PhalaBasePool::set_pool_description(RuntimeOrigin::signed(2), 0, str_bye,),
			basepool::Error::<Test>::UnauthorizedPoolOwner
		);
	});
}

#[test]
fn test_staker_whitelist() {
	new_test_ext().execute_with(|| {
		mock_asset_id();
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(1), 500 * DOLLARS));
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(2), 500 * DOLLARS));
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(3), 500 * DOLLARS));
		set_block_1();
		setup_workers(1);
		setup_stake_pool_with_workers(1, &[1]);

		assert_ok!(PhalaStakePoolv2::contribute(
			RuntimeOrigin::signed(1),
			0,
			40 * DOLLARS,
			None
		));
		assert_ok!(PhalaStakePoolv2::contribute(
			RuntimeOrigin::signed(2),
			0,
			40 * DOLLARS,
			None
		));
		assert_ok!(PhalaStakePoolv2::contribute(
			RuntimeOrigin::signed(3),
			0,
			40 * DOLLARS,
			None
		));
		assert_ok!(PhalaBasePool::add_staker_to_whitelist(
			RuntimeOrigin::signed(1),
			0,
			2,
		));
		let whitelist = PhalaBasePool::pool_whitelist(0).unwrap();
		assert_eq!(whitelist, [2]);
		assert_ok!(PhalaStakePoolv2::contribute(
			RuntimeOrigin::signed(1),
			0,
			10 * DOLLARS,
			None
		));
		assert_ok!(PhalaStakePoolv2::contribute(
			RuntimeOrigin::signed(2),
			0,
			40 * DOLLARS,
			None
		));
		assert_noop!(
			PhalaStakePoolv2::contribute(RuntimeOrigin::signed(3), 0, 40 * DOLLARS, None),
			basepool::Error::<Test>::NotInContributeWhitelist
		);
		assert_ok!(PhalaBasePool::add_staker_to_whitelist(
			RuntimeOrigin::signed(1),
			0,
			3,
		));
		let whitelist = PhalaBasePool::pool_whitelist(0).unwrap();
		assert_eq!(whitelist, [2, 3]);
		assert_ok!(PhalaStakePoolv2::contribute(
			RuntimeOrigin::signed(3),
			0,
			20 * DOLLARS,
			None
		));
		assert_ok!(PhalaBasePool::remove_staker_from_whitelist(
			RuntimeOrigin::signed(1),
			0,
			2
		));
		let whitelist = PhalaBasePool::pool_whitelist(0).unwrap();
		assert_eq!(whitelist, [3]);
		assert_noop!(
			PhalaStakePoolv2::contribute(RuntimeOrigin::signed(2), 0, 20 * DOLLARS, None),
			basepool::Error::<Test>::NotInContributeWhitelist
		);
		assert_ok!(PhalaBasePool::remove_staker_from_whitelist(
			RuntimeOrigin::signed(1),
			0,
			3
		));
		assert!(PhalaBasePool::pool_whitelist(0).is_none());
		assert_ok!(PhalaStakePoolv2::contribute(
			RuntimeOrigin::signed(3),
			0,
			20 * DOLLARS,
			None
		));
	});
}

#[test]
fn test_pool_cap() {
	new_test_ext().execute_with(|| {
		mock_asset_id();
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(1), 500 * DOLLARS));
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(2), 500 * DOLLARS));
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(3), 500 * DOLLARS));
		set_block_1();
		setup_workers(1);
		setup_stake_pool_with_workers(1, &[1]); // pid = 0

		assert_eq!(ensure_stake_pool::<Test>(0).unwrap().cap, None);
		// Pool existence
		assert_noop!(
			PhalaStakePoolv2::set_cap(RuntimeOrigin::signed(2), 100, 1),
			basepool::Error::<Test>::PoolDoesNotExist,
		);
		// Owner only
		assert_noop!(
			PhalaStakePoolv2::set_cap(RuntimeOrigin::signed(2), 0, 1),
			stakepoolv2::Error::<Test>::UnauthorizedPoolOwner,
		);
		// Cap to 1000 PHA
		assert_ok!(PhalaStakePoolv2::set_cap(
			RuntimeOrigin::signed(1),
			0,
			100 * DOLLARS
		));
		assert_eq!(
			ensure_stake_pool::<Test>(0).unwrap().cap,
			Some(100 * DOLLARS)
		);
		// Check cap shouldn't be less than the current stake
		assert_ok!(PhalaStakePoolv2::contribute(
			RuntimeOrigin::signed(1),
			0,
			10 * DOLLARS,
			None
		));
		assert_noop!(
			PhalaStakePoolv2::set_cap(RuntimeOrigin::signed(1), 0, 9 * DOLLARS),
			stakepoolv2::Error::<Test>::InadequateCapacity,
		);
		// Stake to the cap
		assert_ok!(PhalaStakePoolv2::contribute(
			RuntimeOrigin::signed(1),
			0,
			90 * DOLLARS,
			None
		));
		// Exceed the cap
		assert_noop!(
			PhalaStakePoolv2::contribute(RuntimeOrigin::signed(2), 0, 90 * DOLLARS, None),
			stakepoolv2::Error::<Test>::StakeExceedsCapacity,
		);

		// Can stake exceed the cap to swap the withdrawing stake out, as long as the cap
		// can be maintained after the contribution
		assert_ok!(PhalaStakePoolv2::start_computing(
			RuntimeOrigin::signed(1),
			0,
			worker_pubkey(1),
			100 * DOLLARS
		));
		assert_ok!(PhalaStakePoolv2::withdraw(
			RuntimeOrigin::signed(1),
			0,
			100 * DOLLARS,
			None
		));
		assert_noop!(
			PhalaStakePoolv2::contribute(RuntimeOrigin::signed(2), 0, 101 * DOLLARS, None),
			stakepoolv2::Error::<Test>::StakeExceedsCapacity
		);
		assert_ok!(PhalaStakePoolv2::contribute(
			RuntimeOrigin::signed(2),
			0,
			100 * DOLLARS,
			None
		));
	});
}

#[test]
fn test_add_worker() {
	new_test_ext().execute_with(|| {
		set_block_1();
		let worker1 = worker_pubkey(1);
		let worker2 = worker_pubkey(2);

		assert_ok!(PhalaRegistry::force_register_worker(
			RuntimeOrigin::root(),
			worker1.clone(),
			ecdh_pubkey(1),
			Some(1)
		));

		// Create a pool (pid = 0)
		assert_ok!(PhalaStakePoolv2::create(RuntimeOrigin::signed(1)));
		// Bad inputs
		assert_noop!(
			PhalaStakePoolv2::add_worker(RuntimeOrigin::signed(1), 1, worker2.clone()),
			stakepoolv2::Error::<Test>::WorkerNotRegistered
		);
		assert_noop!(
			PhalaStakePoolv2::add_worker(RuntimeOrigin::signed(2), 0, worker1.clone()),
			stakepoolv2::Error::<Test>::UnauthorizedOperator
		);
		assert_noop!(
			PhalaStakePoolv2::add_worker(RuntimeOrigin::signed(1), 0, worker1.clone()),
			stakepoolv2::Error::<Test>::BenchmarkMissing
		);
		// Add benchmark and retry
		PhalaRegistry::internal_set_benchmark(&worker1, Some(1));
		assert_ok!(PhalaStakePoolv2::add_worker(
			RuntimeOrigin::signed(1),
			0,
			worker1.clone()
		));
		// Check binding
		let subaccount = stakepoolv2::pool_sub_account(0, &worker_pubkey(1));
		assert_eq!(
			PhalaComputation::ensure_worker_bound(&worker_pubkey(1)).unwrap(),
			subaccount,
		);
		assert_eq!(
			PhalaComputation::ensure_session_bound(&subaccount).unwrap(),
			worker_pubkey(1),
		);
		// Check assignments
		assert_eq!(
			stakepoolv2::pallet::WorkerAssignments::<Test>::get(&worker_pubkey(1)),
			Some(0)
		);
		// Other bad cases
		assert_noop!(
			PhalaStakePoolv2::add_worker(RuntimeOrigin::signed(1), 100, worker1.clone()),
			basepool::Error::<Test>::PoolDoesNotExist
		);
		// Bind one worker to antoher pool (pid = 1)
		assert_ok!(PhalaStakePoolv2::create(RuntimeOrigin::signed(1)));
		assert_noop!(
			PhalaStakePoolv2::add_worker(RuntimeOrigin::signed(1), 1, worker1.clone()),
			stakepoolv2::Error::<Test>::FailedToBindSessionAndWorker
		);
	});
}
#[test]
fn test_start_computing() {
	new_test_ext().execute_with(|| {
		mock_asset_id();
		set_block_1();
		assert_ok!(PhalaStakePoolv2::create(RuntimeOrigin::signed(1)));
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(1), 500 * DOLLARS));
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(99), 50000 * DOLLARS));
		// Cannot start computing without a bound worker
		assert_noop!(
			PhalaStakePoolv2::start_computing(RuntimeOrigin::signed(1), 0, worker_pubkey(1), 0),
			stakepoolv2::Error::<Test>::WorkerDoesNotExist
		);
		// Basic setup
		setup_workers(2);
		assert_ok!(PhalaStakePoolv2::add_worker(
			RuntimeOrigin::signed(1),
			0,
			worker_pubkey(1)
		));
		assert_ok!(PhalaStakePoolv2::contribute(
			RuntimeOrigin::signed(1),
			0,
			100 * DOLLARS,
			None
		));
		// No enough stake
		assert_noop!(
			PhalaStakePoolv2::start_computing(RuntimeOrigin::signed(1), 0, worker_pubkey(1), 0),
			computation::Error::<Test>::InsufficientStake
		);
		// Too much stake
		assert_ok!(PhalaStakePoolv2::contribute(
			RuntimeOrigin::signed(99),
			0,
			30000 * DOLLARS,
			None
		));
		assert_noop!(
			PhalaStakePoolv2::start_computing(
				RuntimeOrigin::signed(1),
				0,
				worker_pubkey(1),
				30000 * DOLLARS
			),
			computation::Error::<Test>::TooMuchStake
		);
		// Can start computing normally
		assert_ok!(PhalaStakePoolv2::start_computing(
			RuntimeOrigin::signed(1),
			0,
			worker_pubkey(1),
			100 * DOLLARS
		));
		assert_eq!(PhalaComputation::online_workers(), 1);
		let pool = ensure_stake_pool::<Test>(0).unwrap();
		let balance = get_balance(pool.basepool.pool_account_id);
		let lock = get_balance(pool.lock_account);
		assert_eq!((balance, lock), (30000 * DOLLARS, 100 * DOLLARS));
	});
}

#[test]
fn test_force_unbind() {
	new_test_ext().execute_with(|| {
		mock_asset_id();
		set_block_1();
		setup_workers_linked_operators(2);
		setup_stake_pool_with_workers(1, &[1]); // pid = 0
		setup_stake_pool_with_workers(2, &[2]); // pid = 1
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(1), 500 * DOLLARS));
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(2), 500 * DOLLARS));
		assert_ok!(PhalaStakePoolv2::contribute(
			RuntimeOrigin::signed(1),
			0,
			100 * DOLLARS,
			None
		));
		assert_ok!(PhalaStakePoolv2::contribute(
			RuntimeOrigin::signed(2),
			1,
			100 * DOLLARS,
			None
		));
		// Pool0: Change the operator to account101 and force unbind (not computing)
		assert_ok!(PhalaRegistry::force_register_worker(
			RuntimeOrigin::root(),
			worker_pubkey(1),
			ecdh_pubkey(1),
			Some(101)
		));
		let sub_account = stakepoolv2::pool_sub_account(0, &worker_pubkey(1));
		assert_ok!(PhalaComputation::unbind(RuntimeOrigin::signed(101), sub_account));
		// Check worker assignments cleared, and the worker removed from the pool
		assert!(!stakepoolv2::pallet::WorkerAssignments::<Test>::contains_key(&worker_pubkey(1)));
		let pool = ensure_stake_pool::<Test>(0).unwrap();
		assert_eq!(pool.workers.contains(&worker_pubkey(1)), false);
		// Check the computing is ready
		let worker = PhalaComputation::sessions(&sub_account).unwrap();
		assert_eq!(worker.state, computation::WorkerState::Ready);
		let pool = ensure_stake_pool::<Test>(1).unwrap();
		let balance = get_balance(pool.basepool.pool_account_id);
		let lock = get_balance(pool.lock_account);
		assert_eq!((balance, lock), (100 * DOLLARS, 0 * DOLLARS));
		// Pool1: Change the operator to account102 and force unbind (computing)
		assert_ok!(PhalaStakePoolv2::start_computing(
			RuntimeOrigin::signed(2),
			1,
			worker_pubkey(2),
			100 * DOLLARS
		));
		let pool = ensure_stake_pool::<Test>(1).unwrap();
		let balance = get_balance(pool.basepool.pool_account_id);
		let lock = get_balance(pool.lock_account);
		assert_eq!((balance, lock), (0 * DOLLARS, 100 * DOLLARS));
		assert_ok!(PhalaRegistry::force_register_worker(
			RuntimeOrigin::root(),
			worker_pubkey(2),
			ecdh_pubkey(2),
			Some(102)
		));
		let sub_account = stakepoolv2::pool_sub_account(1, &worker_pubkey(2));
		assert_ok!(PhalaComputation::unbind(RuntimeOrigin::signed(102), sub_account));
		// Check worker assignments cleared, and the worker removed from the pool
		assert!(!stakepoolv2::WorkerAssignments::<Test>::contains_key(
			&worker_pubkey(2)
		));
		let pool = ensure_stake_pool::<Test>(1).unwrap();
		assert_eq!(pool.workers.contains(&worker_pubkey(2)), false);
		// Check the computing is stopped
		let worker = PhalaComputation::sessions(&sub_account).unwrap();
		assert_eq!(worker.state, computation::WorkerState::WorkerCoolingDown);
	});
}

#[test]
fn test_stop_computing() {
	new_test_ext().execute_with(|| {
		mock_asset_id();
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(1), 500 * DOLLARS));
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(2), 500 * DOLLARS));
		set_block_1();
		assert_ok!(PhalaStakePoolv2::create(RuntimeOrigin::signed(1)));
		// Cannot start computing without a bound worker
		assert_noop!(
			PhalaStakePoolv2::start_computing(RuntimeOrigin::signed(1), 0, worker_pubkey(1), 0),
			stakepoolv2::Error::<Test>::WorkerDoesNotExist
		);
		// Basic setup
		setup_workers(2);
		assert_ok!(PhalaStakePoolv2::add_worker(
			RuntimeOrigin::signed(1),
			0,
			worker_pubkey(1)
		));
		assert_ok!(PhalaStakePoolv2::contribute(
			RuntimeOrigin::signed(1),
			0,
			100 * DOLLARS,
			None
		));
		let pool = ensure_stake_pool::<Test>(0).unwrap();
		let balance = get_balance(pool.basepool.pool_account_id);
		let lock = get_balance(pool.lock_account);
		assert_eq!((balance, lock), (100 * DOLLARS, 0 * DOLLARS));
		assert_ok!(PhalaStakePoolv2::start_computing(
			RuntimeOrigin::signed(1),
			0,
			worker_pubkey(1),
			100 * DOLLARS,
		));
		let pool = ensure_stake_pool::<Test>(0).unwrap();
		let balance = get_balance(pool.basepool.pool_account_id);
		let lock = get_balance(pool.lock_account);
		assert_eq!((balance, lock), (0 * DOLLARS, 100 * DOLLARS));
		assert_ok!(PhalaStakePoolv2::stop_computing(
			RuntimeOrigin::signed(1),
			0,
			worker_pubkey(1),
		));
		let pool = ensure_stake_pool::<Test>(0).unwrap();
		let balance = get_balance(pool.basepool.pool_account_id);
		let lock = get_balance(pool.lock_account);
		assert_eq!((balance, lock), (0 * DOLLARS, 100 * DOLLARS));
		assert_eq!(pool.cd_workers, [worker_pubkey(1)]);
	});
}

#[test]
fn test_reclaim() {
	new_test_ext().execute_with(|| {
		mock_asset_id();
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(1), 500 * DOLLARS));
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(2), 500 * DOLLARS));
		set_block_1();
		assert_ok!(PhalaStakePoolv2::create(RuntimeOrigin::signed(1)));
		// Basic setup
		setup_workers(2);
		assert_ok!(PhalaStakePoolv2::add_worker(
			RuntimeOrigin::signed(1),
			0,
			worker_pubkey(1)
		));
		assert_ok!(PhalaStakePoolv2::contribute(
			RuntimeOrigin::signed(1),
			0,
			100 * DOLLARS,
			None
		));
		assert_ok!(PhalaStakePoolv2::start_computing(
			RuntimeOrigin::signed(1),
			0,
			worker_pubkey(1),
			100 * DOLLARS,
		));
		assert_ok!(PhalaStakePoolv2::stop_computing(
			RuntimeOrigin::signed(1),
			0,
			worker_pubkey(1),
		));
		elapse_cool_down();
		assert_ok!(PhalaStakePoolv2::reclaim_pool_worker(
			RuntimeOrigin::signed(1),
			0,
			worker_pubkey(1),
		));
		let pool = ensure_stake_pool::<Test>(0).unwrap();
		let balance = get_balance(pool.basepool.pool_account_id);
		let lock = get_balance(pool.lock_account);
		assert_eq!((balance, lock), (100 * DOLLARS, 0 * DOLLARS));
	});
}

#[test]
fn restart_computing_should_work() {
	new_test_ext().execute_with(|| {
		mock_asset_id();
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(1), 500 * DOLLARS));
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(2), 500 * DOLLARS));
		setup_workers(1);
		setup_stake_pool_with_workers(1, &[1]); // pid=0
		assert_ok!(PhalaStakePoolv2::contribute(
			RuntimeOrigin::signed(2),
			0,
			200 * DOLLARS,
			None
		));
		assert_ok!(PhalaStakePoolv2::start_computing(
			RuntimeOrigin::signed(1),
			0,
			worker_pubkey(1),
			150 * DOLLARS
		));
		// Bad cases
		assert_noop!(
			PhalaStakePoolv2::restart_computing(
				RuntimeOrigin::signed(1),
				0,
				worker_pubkey(1),
				50 * DOLLARS
			),
			stakepoolv2::Error::<Test>::CannotRestartWithLessStake
		);
		assert_noop!(
			PhalaStakePoolv2::restart_computing(
				RuntimeOrigin::signed(1),
				0,
				worker_pubkey(1),
				150 * DOLLARS
			),
			stakepoolv2::Error::<Test>::CannotRestartWithLessStake
		);
		// Happy path
		let pool0 = ensure_stake_pool::<Test>(0).unwrap();
		assert_eq!(get_balance(pool0.basepool.pool_account_id), 50 * DOLLARS);
		assert_ok!(PhalaStakePoolv2::restart_computing(
			RuntimeOrigin::signed(1),
			0,
			worker_pubkey(1),
			151 * DOLLARS
		));
		let pool0 = ensure_stake_pool::<Test>(0).unwrap();
		assert_eq!(get_balance(pool0.basepool.pool_account_id), 49 * DOLLARS);
	});
}

#[test]
fn test_for_cdworkers() {
	new_test_ext().execute_with(|| {
		mock_asset_id();
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(1), 500 * DOLLARS));
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(2), 500 * DOLLARS));
		set_block_1();
		assert_ok!(PhalaStakePoolv2::create(RuntimeOrigin::signed(1)));
		// Cannot start computing without a bound worker
		assert_noop!(
			PhalaStakePoolv2::start_computing(RuntimeOrigin::signed(1), 0, worker_pubkey(1), 0),
			stakepoolv2::Error::<Test>::WorkerDoesNotExist
		);
		// Basic setup
		setup_workers(2);
		assert_ok!(PhalaStakePoolv2::add_worker(
			RuntimeOrigin::signed(1),
			0,
			worker_pubkey(1)
		));
		assert_ok!(PhalaStakePoolv2::contribute(
			RuntimeOrigin::signed(1),
			0,
			100 * DOLLARS,
			None
		));
		assert_ok!(PhalaStakePoolv2::start_computing(
			RuntimeOrigin::signed(1),
			0,
			worker_pubkey(1),
			100 * DOLLARS
		));
		assert_noop!(
			PhalaStakePoolv2::remove_worker(RuntimeOrigin::signed(1), 0, worker_pubkey(1)),
			stakepoolv2::Error::<Test>::WorkerIsNotReady,
		);
		let pool = ensure_stake_pool::<Test>(0).unwrap();
		assert_eq!(pool.cd_workers, []);
		assert_ok!(PhalaStakePoolv2::stop_computing(
			RuntimeOrigin::signed(1),
			0,
			worker_pubkey(1),
		));
		elapse_cool_down();
		assert_ok!(PhalaStakePoolv2::reclaim_pool_worker(
			RuntimeOrigin::signed(1),
			0,
			worker_pubkey(1),
		));
		assert_ok!(PhalaStakePoolv2::remove_worker(
			RuntimeOrigin::signed(1),
			0,
			worker_pubkey(1),
		));
		let pool = ensure_stake_pool::<Test>(0).unwrap();
		assert_eq!(pool.cd_workers, []);
	});
}

#[test]
fn test_on_reward_for_vault() {
	use crate::computation::pallet::OnReward;
	new_test_ext().execute_with(|| {
		mock_asset_id();
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(1), 500 * DOLLARS));
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(2), 500 * DOLLARS));
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(3), 500 * DOLLARS));
		set_block_1();
		setup_workers(1);
		setup_vault(3); // pid = 0
		setup_stake_pool_with_workers(1, &[1]);
		assert_ok!(PhalaStakePoolv2::set_payout_pref(
			RuntimeOrigin::signed(1),
			1,
			Some(Permill::from_percent(50))
		));
		assert_ok!(PhalaVault::contribute(RuntimeOrigin::signed(3), 0, 100 * DOLLARS));
		assert_ok!(PhalaStakePoolv2::contribute(
			RuntimeOrigin::signed(3),
			1,
			50 * DOLLARS,
			Some(0)
		));
		// Staker2 contribute 1000 PHA and start computing
		assert_ok!(PhalaStakePoolv2::contribute(
			RuntimeOrigin::signed(2),
			1,
			50 * DOLLARS,
			None
		));
		assert_ok!(PhalaStakePoolv2::start_computing(
			RuntimeOrigin::signed(1),
			1,
			worker_pubkey(1),
			100 * DOLLARS
		));
		PhalaStakePoolv2::on_reward(&vec![SettleInfo {
			pubkey: worker_pubkey(1),
			v: FixedPoint::from_num(1u32).to_bits(),
			payout: FixedPoint::from_num(100u32).to_bits(),
			treasury: 0,
		}]);
		let pool = ensure_stake_pool::<Test>(1).unwrap();
		assert_eq!(get_balance(pool.owner_reward_account), 50 * DOLLARS);
		assert_eq!(get_balance(pool.basepool.pool_account_id), 50 * DOLLARS);
		assert_eq!(pool.basepool.total_value, 150 * DOLLARS);
		assert_eq!(pool.basepool.total_shares, 100 * DOLLARS);
		let vault_info = ensure_vault::<Test>(0).unwrap();
		assert_eq!(vault_info.basepool.total_value, 125 * DOLLARS);
		assert_eq!(
			get_balance(vault_info.basepool.pool_account_id),
			50 * DOLLARS
		);
		assert_eq!(vault_info.basepool.total_shares, 100 * DOLLARS);
		let free = Balances::free_balance(&<Test as pawnshop::Config>::PawnShopAccountId::get());
		assert_eq!(free, 1600 * DOLLARS);
	});
}

#[test]
fn test_claim_owner_rewards() {
	use crate::computation::pallet::OnReward;
	new_test_ext().execute_with(|| {
		mock_asset_id();
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(1), 500 * DOLLARS));
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(2), 500 * DOLLARS));
		set_block_1();
		setup_workers(1);
		setup_stake_pool_with_workers(1, &[1]); // pid = 0
		assert_ok!(PhalaStakePoolv2::set_payout_pref(
			RuntimeOrigin::signed(1),
			0,
			Some(Permill::from_percent(50))
		));
		assert_ok!(PhalaStakePoolv2::contribute(
			RuntimeOrigin::signed(1),
			0,
			100 * DOLLARS,
			None
		));
		assert_ok!(PhalaStakePoolv2::contribute(
			RuntimeOrigin::signed(2),
			0,
			400 * DOLLARS,
			None
		));
		PhalaStakePoolv2::on_reward(&vec![SettleInfo {
			pubkey: worker_pubkey(1),
			v: FixedPoint::from_num(1u32).to_bits(),
			payout: FixedPoint::from_num(1000u32).to_bits(),
			treasury: 0,
		}]);
		let pool = ensure_stake_pool::<Test>(0).unwrap();
		assert_eq!(get_balance(pool.owner_reward_account), 500 * DOLLARS);
		assert_ok!(PhalaStakePoolv2::claim_owner_rewards(
			RuntimeOrigin::signed(1),
			0,
			1
		));
		let pool = ensure_stake_pool::<Test>(0).unwrap();
		assert_eq!(get_balance(pool.owner_reward_account), 0 * DOLLARS);
		assert_eq!(get_balance(1), 900 * DOLLARS);
	});
}

#[test]
fn test_vault_owner_shares() {
	use crate::computation::pallet::OnReward;
	new_test_ext().execute_with(|| {
		mock_asset_id();
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(1), 500 * DOLLARS));
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(2), 500 * DOLLARS));
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(3), 500 * DOLLARS));
		set_block_1();
		setup_workers(1);
		setup_vault(3); // pid = 0
		assert_ok!(PhalaVault::set_payout_pref(
			RuntimeOrigin::signed(3),
			0,
			Some(Permill::from_percent(50))
		));
		setup_stake_pool_with_workers(1, &[1]);
		assert_ok!(PhalaVault::contribute(RuntimeOrigin::signed(3), 0, 100 * DOLLARS));
		assert_ok!(PhalaStakePoolv2::contribute(
			RuntimeOrigin::signed(3),
			1,
			50 * DOLLARS,
			Some(0)
		));
		let vault_info = ensure_vault::<Test>(0).unwrap();
		assert_eq!(vault_info.commission.unwrap(), Permill::from_percent(50));
		assert_ok!(PhalaVault::maybe_gain_owner_shares(RuntimeOrigin::signed(3), 0));
		let vault_info = ensure_vault::<Test>(0).unwrap();
		assert_eq!(vault_info.owner_shares, 0);
		assert_ok!(PhalaStakePoolv2::contribute(
			RuntimeOrigin::signed(2),
			1,
			50 * DOLLARS,
			None
		));
		assert_ok!(PhalaStakePoolv2::start_computing(
			RuntimeOrigin::signed(1),
			1,
			worker_pubkey(1),
			100 * DOLLARS
		));
		PhalaStakePoolv2::on_reward(&vec![SettleInfo {
			pubkey: worker_pubkey(1),
			v: FixedPoint::from_num(1u32).to_bits(),
			payout: FixedPoint::from_num(100u32).to_bits(),
			treasury: 0,
		}]);
		let pool = ensure_stake_pool::<Test>(1).unwrap();
		assert_eq!(get_balance(pool.basepool.pool_account_id), 100 * DOLLARS);
		assert_eq!(pool.basepool.total_value, 200 * DOLLARS);
		let vault_info = ensure_vault::<Test>(0).unwrap();
		assert_eq!(vault_info.basepool.total_value, 150 * DOLLARS);
		assert_eq!(
			get_balance(vault_info.basepool.pool_account_id),
			50 * DOLLARS
		);
		assert_eq!(vault_info.basepool.total_shares, 100 * DOLLARS);
		assert_ok!(PhalaVault::maybe_gain_owner_shares(RuntimeOrigin::signed(3), 0));
		let vault_info = ensure_vault::<Test>(0).unwrap();
		assert_eq!(vault_info.owner_shares, 20 * DOLLARS);
		assert_eq!(vault_info.basepool.total_shares, 120 * DOLLARS);
		assert_noop!(
			PhalaVault::claim_owner_shares(RuntimeOrigin::signed(3), 0, 4, 50 * DOLLARS),
			vault::Error::<Test>::InvaildWithdrawSharesAmount
		);
		assert_ok!(PhalaVault::claim_owner_shares(
			RuntimeOrigin::signed(3),
			0,
			4,
			10 * DOLLARS
		));
		let vault_info = ensure_vault::<Test>(0).unwrap();
		assert_eq!(vault_info.owner_shares, 10 * DOLLARS);
		let mut nftid_arr: Vec<NftId> =
			pallet_rmrk_core::Nfts::<Test>::iter_key_prefix(vault_info.basepool.cid).collect();
		nftid_arr.retain(|x| {
			let nft = pallet_rmrk_core::Nfts::<Test>::get(vault_info.basepool.cid, x).unwrap();
			nft.owner == rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(4)
		});
		assert_eq!(nftid_arr.len(), 1);
		{
			let nft_attr = PhalaBasePool::get_nft_attr_guard(vault_info.basepool.cid, nftid_arr[0])
				.unwrap()
				.attr
				.clone();
			assert_eq!(nft_attr.shares, 10 * DOLLARS);
		}
	});
}

#[test]
fn test_withdraw() {
	new_test_ext().execute_with(|| {
		mock_asset_id();
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(1), 500 * DOLLARS));
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(2), 500 * DOLLARS));
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(3), 500 * DOLLARS));
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(99), 5000 * DOLLARS));
		set_block_1();
		setup_workers(2);
		setup_stake_pool_with_workers(1, &[1, 2]); // pid = 0
		assert_ok!(PhalaStakePoolv2::contribute(
			RuntimeOrigin::signed(2),
			0,
			300 * DOLLARS,
			None
		));
		assert_ok!(PhalaStakePoolv2::contribute(
			RuntimeOrigin::signed(1),
			0,
			300 * DOLLARS,
			None
		));
		assert_ok!(PhalaStakePoolv2::contribute(
			RuntimeOrigin::signed(3),
			0,
			300 * DOLLARS,
			None
		));
		assert_ok!(PhalaStakePoolv2::start_computing(
			RuntimeOrigin::signed(1),
			0,
			worker_pubkey(1),
			400 * DOLLARS
		));
		assert_ok!(PhalaStakePoolv2::start_computing(
			RuntimeOrigin::signed(1),
			0,
			worker_pubkey(2),
			300 * DOLLARS
		));
		assert_ok!(PhalaStakePoolv2::withdraw(
			RuntimeOrigin::signed(2),
			0,
			300 * DOLLARS,
			None
		));
		let pool = ensure_stake_pool::<Test>(0).unwrap();
		let item = pool
			.basepool
			.withdraw_queue
			.clone()
			.into_iter()
			.find(|x| x.user == 2);
		{
			let nft_attr =
				PhalaBasePool::get_nft_attr_guard(pool.basepool.cid, item.unwrap().nft_id)
					.unwrap()
					.attr
					.clone();
			assert_eq!(nft_attr.shares, 100 * DOLLARS);
		}
		let mut nftid_arr: Vec<NftId> =
			pallet_rmrk_core::Nfts::<Test>::iter_key_prefix(0).collect();
		nftid_arr.retain(|x| {
			let nft = pallet_rmrk_core::Nfts::<Test>::get(0, x).unwrap();
			nft.owner == rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(2)
		});
		assert_eq!(nftid_arr.len(), 1);
		{
			let user_nft_attr = PhalaBasePool::get_nft_attr_guard(pool.basepool.cid, nftid_arr[0])
				.unwrap()
				.attr
				.clone();
			assert_eq!(user_nft_attr.shares, 0 * DOLLARS);
		}
		assert_eq!(get_balance(2), 400 * DOLLARS);
		assert_ok!(PhalaStakePoolv2::contribute(
			RuntimeOrigin::signed(99),
			0,
			300 * DOLLARS,
			None
		));
		let pool = ensure_stake_pool::<Test>(0).unwrap();
		assert_eq!(pool.basepool.withdraw_queue.len(), 0);
		assert_eq!(get_balance(2), 500 * DOLLARS);
		let mut nftid_arr: Vec<NftId> =
			pallet_rmrk_core::Nfts::<Test>::iter_key_prefix(0).collect();
		nftid_arr.retain(|x| {
			let nft = pallet_rmrk_core::Nfts::<Test>::get(0, x).unwrap();
			nft.owner == rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(2)
		});
		assert_eq!(nftid_arr.len(), 1);
		{
			let user_nft_attr = PhalaBasePool::get_nft_attr_guard(pool.basepool.cid, nftid_arr[0])
				.unwrap()
				.attr
				.clone();
			assert_eq!(user_nft_attr.shares, 0 * DOLLARS);
		}
		assert_ok!(PhalaStakePoolv2::withdraw(
			RuntimeOrigin::signed(1),
			0,
			200 * DOLLARS,
			None
		));
		let pool = ensure_stake_pool::<Test>(0).unwrap();
		assert_eq!(pool.basepool.withdraw_queue.len(), 0);
		assert_eq!(get_balance(1), 400 * DOLLARS);
		let mut nftid_arr: Vec<NftId> =
			pallet_rmrk_core::Nfts::<Test>::iter_key_prefix(0).collect();
		nftid_arr.retain(|x| {
			let nft = pallet_rmrk_core::Nfts::<Test>::get(0, x).unwrap();
			nft.owner == rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(1)
		});
		assert_eq!(nftid_arr.len(), 1);
		{
			let user_nft_attr = PhalaBasePool::get_nft_attr_guard(pool.basepool.cid, nftid_arr[0])
				.unwrap()
				.attr
				.clone();
			assert_eq!(user_nft_attr.shares, 100 * DOLLARS);
		}
		let _pid = setup_vault(99);
		assert_ok!(PhalaVault::contribute(RuntimeOrigin::signed(1), 1, 300 * DOLLARS,));
		assert_ok!(PhalaVault::contribute(RuntimeOrigin::signed(99), 1, 300 * DOLLARS,));
		assert_ok!(PhalaStakePoolv2::contribute(
			RuntimeOrigin::signed(99),
			0,
			500 * DOLLARS,
			Some(1)
		));
		assert_ok!(PhalaVault::withdraw(RuntimeOrigin::signed(1), 1, 200 * DOLLARS,));
		let pool = ensure_vault::<Test>(1).unwrap();
		let item = pool
			.basepool
			.withdraw_queue
			.clone()
			.into_iter()
			.find(|x| x.user == 1);
		{
			let nft_attr =
				PhalaBasePool::get_nft_attr_guard(pool.basepool.cid, item.unwrap().nft_id)
					.unwrap()
					.attr
					.clone();
			assert_eq!(nft_attr.shares, 100 * DOLLARS);
		}
		let mut nftid_arr: Vec<NftId> =
			pallet_rmrk_core::Nfts::<Test>::iter_key_prefix(1).collect();
		nftid_arr.retain(|x| {
			let nft = pallet_rmrk_core::Nfts::<Test>::get(1, x).unwrap();
			nft.owner == rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(1)
		});
		assert_eq!(nftid_arr.len(), 1);
		{
			let user_nft_attr = PhalaBasePool::get_nft_attr_guard(pool.basepool.cid, nftid_arr[0])
				.unwrap()
				.attr
				.clone();
			assert_eq!(user_nft_attr.shares, 100 * DOLLARS);
		}
		assert_eq!(get_balance(1), 200 * DOLLARS);
		assert_ok!(PhalaStakePoolv2::withdraw(
			RuntimeOrigin::signed(3),
			0,
			200 * DOLLARS,
			None
		));
		let pool = ensure_stake_pool::<Test>(0).unwrap();
		assert_eq!(get_balance(pool.basepool.pool_account_id), 300 * DOLLARS);
		assert_ok!(PhalaStakePoolv2::withdraw(
			RuntimeOrigin::signed(99),
			0,
			400 * DOLLARS,
			Some(1)
		));
		let pool = ensure_stake_pool::<Test>(0).unwrap();
		let vault = ensure_vault::<Test>(1).unwrap();
		let item = pool
			.basepool
			.withdraw_queue
			.clone()
			.into_iter()
			.find(|x| x.user == vault.basepool.pool_account_id);
		{
			let nft_attr =
				PhalaBasePool::get_nft_attr_guard(pool.basepool.cid, item.unwrap().nft_id)
					.unwrap()
					.attr
					.clone();
			assert_eq!(nft_attr.shares, 100 * DOLLARS);
		}
		let mut nftid_arr: Vec<NftId> =
			pallet_rmrk_core::Nfts::<Test>::iter_key_prefix(0).collect();
		nftid_arr.retain(|x| {
			let nft = pallet_rmrk_core::Nfts::<Test>::get(0, x).unwrap();
			nft.owner
				== rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(
					vault.basepool.pool_account_id,
				)
		});
		assert_eq!(nftid_arr.len(), 1);
		{
			let user_nft_attr = PhalaBasePool::get_nft_attr_guard(pool.basepool.cid, nftid_arr[0])
				.unwrap()
				.attr
				.clone();
			assert_eq!(user_nft_attr.shares, 100 * DOLLARS);
		}
		assert_eq!(get_balance(vault.basepool.pool_account_id), 300 * DOLLARS);
		let mut nftid_arr: Vec<NftId> =
			pallet_rmrk_core::Nfts::<Test>::iter_key_prefix(1).collect();
		nftid_arr.retain(|x| {
			let nft = pallet_rmrk_core::Nfts::<Test>::get(1, x).unwrap();
			nft.owner == rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(1)
		});
		assert_eq!(nftid_arr.len(), 1);
		{
			let user_nft_attr = PhalaBasePool::get_nft_attr_guard(vault.basepool.cid, nftid_arr[0])
				.unwrap()
				.attr
				.clone();
			assert_eq!(user_nft_attr.shares, 100 * DOLLARS);
		}
	});
}

#[test]
fn test_check_and_maybe_force_withdraw() {
	new_test_ext().execute_with(|| {
		mock_asset_id();
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(1), 500 * DOLLARS));
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(2), 500 * DOLLARS));
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(3), 500 * DOLLARS));
		assert_ok!(PhalaPawnshop::pawn(RuntimeOrigin::signed(99), 500 * DOLLARS));
		set_block_1();
		setup_workers(2);
		setup_stake_pool_with_workers(1, &[1, 2]); // pid = 0
		assert_ok!(PhalaStakePoolv2::contribute(
			RuntimeOrigin::signed(2),
			0,
			300 * DOLLARS,
			None
		));
		assert_ok!(PhalaStakePoolv2::start_computing(
			RuntimeOrigin::signed(1),
			0,
			worker_pubkey(1),
			100 * DOLLARS
		));
		assert_ok!(PhalaStakePoolv2::start_computing(
			RuntimeOrigin::signed(1),
			0,
			worker_pubkey(2),
			100 * DOLLARS
		));
		assert_ok!(PhalaStakePoolv2::withdraw(
			RuntimeOrigin::signed(2),
			0,
			300 * DOLLARS,
			None
		));
		assert_ok!(PhalaStakePoolv2::stop_computing(
			RuntimeOrigin::signed(1),
			0,
			worker_pubkey(1),
		));
		elapse_seconds(864000);
		let pool = ensure_stake_pool::<Test>(0).unwrap();
		assert_eq!(get_balance(pool.basepool.pool_account_id), 0 * DOLLARS);
		assert_eq!(pool.cd_workers, [worker_pubkey(1)]);
		assert_ok!(PhalaStakePoolv2::reclaim_pool_worker(
			RuntimeOrigin::signed(3),
			0,
			worker_pubkey(1)
		));
		let pool = ensure_stake_pool::<Test>(0).unwrap();
		assert_eq!(get_balance(pool.basepool.pool_account_id), 0 * DOLLARS);
		assert_eq!(pool.cd_workers, []);
		let item = pool
			.basepool
			.withdraw_queue
			.clone()
			.into_iter()
			.find(|x| x.user == 2);
		{
			let nft_attr =
				PhalaBasePool::get_nft_attr_guard(pool.basepool.cid, item.unwrap().nft_id)
					.unwrap()
					.attr
					.clone();
			assert_eq!(nft_attr.shares, 100 * DOLLARS);
		}
		let mut nftid_arr: Vec<NftId> =
			pallet_rmrk_core::Nfts::<Test>::iter_key_prefix(0).collect();
		nftid_arr.retain(|x| {
			let nft = pallet_rmrk_core::Nfts::<Test>::get(0, x).unwrap();
			nft.owner == rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(2)
		});
		assert_eq!(nftid_arr.len(), 1);
		{
			let user_nft_attr = PhalaBasePool::get_nft_attr_guard(pool.basepool.cid, nftid_arr[0])
				.unwrap()
				.attr
				.clone();
			assert_eq!(user_nft_attr.shares, 0 * DOLLARS);
		}
		assert_eq!(get_balance(2), 400 * DOLLARS);
		assert_ok!(PhalaStakePoolv2::check_and_maybe_force_withdraw(
			RuntimeOrigin::signed(3),
			0
		));
		let pool = ensure_stake_pool::<Test>(0).unwrap();
		assert_eq!(pool.cd_workers, [worker_pubkey(2)]);
		elapse_seconds(864000);
		assert_ok!(PhalaStakePoolv2::reclaim_pool_worker(
			RuntimeOrigin::signed(3),
			0,
			worker_pubkey(2)
		));
		let pid = setup_vault(99);

		assert_ok!(PhalaVault::contribute(RuntimeOrigin::signed(3), 1, 400 * DOLLARS,));
		assert_ok!(PhalaStakePoolv2::contribute(
			RuntimeOrigin::signed(99),
			0,
			300 * DOLLARS,
			Some(pid)
		));
		assert_ok!(PhalaStakePoolv2::start_computing(
			RuntimeOrigin::signed(1),
			0,
			worker_pubkey(1),
			100 * DOLLARS
		));
		assert_ok!(PhalaStakePoolv2::start_computing(
			RuntimeOrigin::signed(1),
			0,
			worker_pubkey(2),
			100 * DOLLARS
		));
		assert_ok!(PhalaVault::withdraw(RuntimeOrigin::signed(3), 1, 400 * DOLLARS,));
		assert_ok!(PhalaStakePoolv2::withdraw(
			RuntimeOrigin::signed(99),
			0,
			100 * DOLLARS,
			Some(pid)
		));
		assert_ok!(PhalaVault::check_and_maybe_force_withdraw(
			RuntimeOrigin::signed(4),
			pid
		));
		let vault = ensure_vault::<Test>(1).unwrap();
		assert_eq!(get_balance(vault.basepool.pool_account_id), 0 * DOLLARS);
		let item = vault
			.basepool
			.withdraw_queue
			.clone()
			.into_iter()
			.find(|x| x.user == 3);
		{
			let nft_attr =
				PhalaBasePool::get_nft_attr_guard(vault.basepool.cid, item.unwrap().nft_id)
					.unwrap()
					.attr
					.clone();
			assert_eq!(nft_attr.shares, 200 * DOLLARS);
		}
		elapse_seconds(864000);
		assert_ok!(PhalaVault::check_and_maybe_force_withdraw(
			RuntimeOrigin::signed(4),
			pid
		));
		let pool = ensure_stake_pool::<Test>(0).unwrap();
		let item = pool
			.basepool
			.withdraw_queue
			.clone()
			.into_iter()
			.find(|x| x.user == vault.basepool.pool_account_id);
		{
			let nft_attr =
				PhalaBasePool::get_nft_attr_guard(pool.basepool.cid, item.unwrap().nft_id)
					.unwrap()
					.attr
					.clone();
			assert_eq!(nft_attr.shares, 200 * DOLLARS);
		}
		assert_noop!(
			PhalaStakePoolv2::withdraw(RuntimeOrigin::signed(99), 0, 100 * DOLLARS, Some(pid)),
			stakepoolv2::Error::<Test>::VaultIsLocked,
		);
		assert_ok!(PhalaVault::contribute(RuntimeOrigin::signed(99), 1, 400 * DOLLARS,));
		assert_ok!(PhalaVault::check_and_maybe_force_withdraw(
			RuntimeOrigin::signed(4),
			pid
		));
		assert_ok!(PhalaStakePoolv2::withdraw(
			RuntimeOrigin::signed(99),
			0,
			100 * DOLLARS,
			Some(pid)
		),);
	});
}

fn mock_asset_id() {
	<pallet_assets::pallet::Pallet<Test> as Create<u64>>::create(
		<Test as pawnshop::Config>::PPhaAssetId::get(),
		1,
		true,
		1,
	)
	.expect("create should success .qed");
}

fn get_balance(account_id: u64) -> u128 {
	<pallet_assets::pallet::Pallet<Test> as Inspect<u64>>::balance(
		<Test as pawnshop::Config>::PPhaAssetId::get(),
		&account_id,
	)
}

fn setup_stake_pool_with_workers(owner: u64, workers: &[u8]) -> u64 {
	let pid = PhalaBasePool::pool_count();
	assert_ok!(PhalaStakePoolv2::create(RuntimeOrigin::signed(owner)));
	for id in workers {
		assert_ok!(PhalaStakePoolv2::add_worker(
			RuntimeOrigin::signed(owner),
			pid,
			worker_pubkey(*id),
		));
	}
	pid
}

fn setup_vault(owner: u64) -> u64 {
	let pid = PhalaBasePool::pool_count();
	assert_ok!(PhalaVault::create(RuntimeOrigin::signed(owner)));
	pid
}

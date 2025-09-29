// Copyright 2025 Moonbeam foundation.
// This file is part of Moonbeam.

// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

//! Tests for the crowdloan-rewards pallet

use super::*;
use crate::mock::*;
use crate::pallet::{EndVestingBlock, InitVestingBlock, Initialized};
use frame_support::{assert_noop, assert_ok, traits::Currency};
use sp_core::crypto::AccountId32;
use sp_runtime::traits::AccountIdConversion;

// Helper function to create a test account
fn account(id: u8) -> AccountId32 {
	AccountId32::from([id; 32])
}

// Helper function to fund the pallet account
fn fund_pallet(amount: Balance) {
	let _ = Balances::make_free_balance_be(&CrowdloanRewards::account_id(), amount);
}

// Helper function to setup initial reward data using direct storage
fn setup_reward_data(
	relay_account: AccountId32,
	reward_account: Option<AccountId32>,
	total_reward: Balance,
) {
	let claimed_reward = if reward_account.is_some() {
		InitializationPayment::get() * total_reward
	} else {
		0
	};

	let reward_info = RewardInfo {
		total_reward,
		claimed_reward,
		contributed_relay_addresses: vec![relay_account.clone()],
	};

	if let Some(reward_account) = reward_account {
		// Associated contribution
		AccountsPayable::<Test>::insert(&reward_account, &reward_info);
		ClaimedRelayChainIds::<Test>::insert(&relay_account, ());
	} else {
		// Unassociated contribution
		UnassociatedContributions::<Test>::insert(&relay_account, &reward_info);
	}

	// Set up vesting blocks
	InitVestingBlock::<Test>::put(1u32);
	EndVestingBlock::<Test>::put(100u32);

	// Mark as initialized
	Initialized::<Test>::put(true);
}

#[test]
fn test_claim_works_with_full_vesting() {
	new_test_ext().execute_with(|| {
		let reward_account = account(1);
		let relay_account = account(10);
		let total_reward = 10_000u128;

		// Fund the pallet
		fund_pallet(1_000_000);

		// Setup reward data
		setup_reward_data(
			relay_account.clone(),
			Some(reward_account.clone()),
			total_reward,
		);

		// Move to end of vesting period
		run_to_block(100);

		let initial_balance = Balances::free_balance(&reward_account);
		let pallet_initial_balance = Balances::free_balance(&CrowdloanRewards::account_id());

		// Claim rewards
		assert_ok!(CrowdloanRewards::claim(RuntimeOrigin::signed(
			reward_account.clone()
		)));

		// Check that rewards were paid
		let reward_info = AccountsPayable::<Test>::get(&reward_account).unwrap();
		assert_eq!(reward_info.claimed_reward, total_reward);

		// Check balances
		let final_balance = Balances::free_balance(&reward_account);
		let pallet_final_balance = Balances::free_balance(&CrowdloanRewards::account_id());

		// Should have received remaining vested amount
		let initialization_payment = InitializationPayment::get() * total_reward;
		let expected_claim = total_reward - initialization_payment;
		assert_eq!(final_balance, initial_balance + expected_claim);
		assert_eq!(
			pallet_final_balance,
			pallet_initial_balance - expected_claim
		);

		// Note: Event checking could be added here if needed
	});
}

#[test]
fn test_claim_works_with_partial_vesting() {
	new_test_ext().execute_with(|| {
		let reward_account = account(1);
		let relay_account = account(10);
		let total_reward = 10_000u128;

		// Fund the pallet
		fund_pallet(1_000_000);

		// Setup reward data
		setup_reward_data(
			relay_account.clone(),
			Some(reward_account.clone()),
			total_reward,
		);

		// Move to 50% of vesting period (block 50 out of 100)
		run_to_block(50);

		let initial_balance = Balances::free_balance(&reward_account);

		// Claim rewards
		assert_ok!(CrowdloanRewards::claim(RuntimeOrigin::signed(
			reward_account.clone()
		)));

		// Check that partial rewards were paid
		let reward_info = AccountsPayable::<Test>::get(&reward_account).unwrap();

		// Calculate expected vested amount
		let initialization_payment = InitializationPayment::get() * total_reward;
		let remaining_to_vest = total_reward - initialization_payment;
		let vesting_period = 100u32 - 1u32; // 99 blocks
		let elapsed_period = 50u32 - 1u32; // 49 blocks
		let expected_vested = remaining_to_vest * elapsed_period as u128 / vesting_period as u128;
		let expected_total_claimed = initialization_payment + expected_vested;

		assert_eq!(reward_info.claimed_reward, expected_total_claimed);

		// Check balance increased by the vested amount
		let final_balance = Balances::free_balance(&reward_account);
		assert_eq!(final_balance, initial_balance + expected_vested);
	});
}

#[test]
fn test_claim_fails_when_no_rewards() {
	new_test_ext().execute_with(|| {
		let reward_account = account(1);

		// Try to claim without having any rewards
		assert_noop!(
			CrowdloanRewards::claim(RuntimeOrigin::signed(reward_account)),
			Error::<Test>::NoAssociatedClaim
		);
	});
}

#[test]
fn test_claim_fails_when_not_initialized() {
	new_test_ext().execute_with(|| {
		let reward_account = account(1);
		let relay_account = account(10);
		let total_reward = 10_000u128;

		// Setup reward data but mark as not initialized
		setup_reward_data(
			relay_account.clone(),
			Some(reward_account.clone()),
			total_reward,
		);
		Initialized::<Test>::put(false);

		// Try to claim
		assert_noop!(
			CrowdloanRewards::claim(RuntimeOrigin::signed(reward_account)),
			Error::<Test>::RewardVecNotFullyInitializedYet
		);
	});
}

#[test]
fn test_claim_fails_when_all_rewards_claimed() {
	new_test_ext().execute_with(|| {
		let reward_account = account(1);
		let relay_account = account(10);
		let total_reward = 10_000u128;

		// Fund the pallet
		fund_pallet(1_000_000);

		// Setup reward data with all rewards already claimed
		let reward_info = RewardInfo {
			total_reward,
			claimed_reward: total_reward, // All claimed
			contributed_relay_addresses: vec![relay_account.clone()],
		};

		AccountsPayable::<Test>::insert(&reward_account, &reward_info);
		ClaimedRelayChainIds::<Test>::insert(&relay_account, ());
		InitVestingBlock::<Test>::put(1u32);
		EndVestingBlock::<Test>::put(100u32);
		Initialized::<Test>::put(true);

		// Try to claim
		assert_noop!(
			CrowdloanRewards::claim(RuntimeOrigin::signed(reward_account)),
			Error::<Test>::RewardsAlreadyClaimed
		);
	});
}

#[test]
fn test_update_reward_address_works() {
	new_test_ext().execute_with(|| {
		let old_reward_account = account(1);
		let new_reward_account = account(2);
		let relay_account = account(10);
		let total_reward = 10_000u128;

		// Setup reward data
		setup_reward_data(
			relay_account.clone(),
			Some(old_reward_account.clone()),
			total_reward,
		);

		// Update reward address
		assert_ok!(CrowdloanRewards::update_reward_address(
			RuntimeOrigin::signed(old_reward_account.clone()),
			new_reward_account.clone()
		));

		// Check that old account no longer has rewards
		assert!(AccountsPayable::<Test>::get(&old_reward_account).is_none());

		// Check that new account has the rewards
		let reward_info = AccountsPayable::<Test>::get(&new_reward_account).unwrap();
		assert_eq!(reward_info.total_reward, total_reward);

		// Note: Event checking could be added here if needed
	});
}

#[test]
fn test_update_reward_address_fails_when_no_rewards() {
	new_test_ext().execute_with(|| {
		let old_reward_account = account(1);
		let new_reward_account = account(2);

		// Try to update address without having rewards
		assert_noop!(
			CrowdloanRewards::update_reward_address(
				RuntimeOrigin::signed(old_reward_account),
				new_reward_account
			),
			Error::<Test>::NoAssociatedClaim
		);
	});
}

#[test]
fn test_update_reward_address_fails_when_new_account_already_has_rewards() {
	new_test_ext().execute_with(|| {
		let old_reward_account = account(1);
		let new_reward_account = account(2);
		let relay_account1 = account(10);
		let relay_account2 = account(11);
		let total_reward = 10_000u128;

		// Setup reward data for both accounts
		setup_reward_data(
			relay_account1.clone(),
			Some(old_reward_account.clone()),
			total_reward,
		);
		setup_reward_data(
			relay_account2.clone(),
			Some(new_reward_account.clone()),
			total_reward,
		);

		// Try to update address to an account that already has rewards
		assert_noop!(
			CrowdloanRewards::update_reward_address(
				RuntimeOrigin::signed(old_reward_account),
				new_reward_account
			),
			Error::<Test>::AlreadyAssociated
		);
	});
}

#[test]
fn test_pot_returns_correct_balance() {
	new_test_ext().execute_with(|| {
		let expected_balance = 1_000_000_000u128;
		assert_eq!(CrowdloanRewards::pot(), expected_balance);
	});
}

#[test]
fn test_account_id_returns_correct_account() {
	new_test_ext().execute_with(|| {
		let expected_account = CrowdloanPalletId::get().into_account_truncating();
		assert_eq!(CrowdloanRewards::account_id(), expected_account);
	});
}

#[test]
fn test_vesting_calculation_with_zero_period() {
	new_test_ext().execute_with(|| {
		let reward_account = account(1);
		let relay_account = account(10);
		let total_reward = 10_000u128;

		// Fund the pallet
		fund_pallet(1_000_000);

		// Setup with zero vesting period
		let reward_info = RewardInfo {
			total_reward,
			claimed_reward: InitializationPayment::get() * total_reward,
			contributed_relay_addresses: vec![relay_account.clone()],
		};

		AccountsPayable::<Test>::insert(&reward_account, &reward_info);
		ClaimedRelayChainIds::<Test>::insert(&relay_account, ());
		InitVestingBlock::<Test>::put(1u32);
		EndVestingBlock::<Test>::put(1u32); // Same as init = zero period
		Initialized::<Test>::put(true);

		run_to_block(10);

		let initial_balance = Balances::free_balance(&reward_account);

		// Claim with zero vesting period should pay everything immediately
		assert_ok!(CrowdloanRewards::claim(RuntimeOrigin::signed(
			reward_account.clone()
		)));

		let final_balance = Balances::free_balance(&reward_account);
		let initialization_payment = InitializationPayment::get() * total_reward;
		let remaining_reward = total_reward - initialization_payment;

		assert_eq!(final_balance, initial_balance + remaining_reward);
	});
}

#[test]
fn test_multiple_claims_during_vesting() {
	new_test_ext().execute_with(|| {
		let reward_account = account(1);
		let relay_account = account(10);
		let total_reward = 10_000u128;

		// Fund the pallet
		fund_pallet(1_000_000);

		// Setup reward data
		setup_reward_data(
			relay_account.clone(),
			Some(reward_account.clone()),
			total_reward,
		);

		let initialization_payment = InitializationPayment::get() * total_reward;
		let initial_balance = Balances::free_balance(&reward_account);

		// First claim at 25% vesting
		run_to_block(25);
		assert_ok!(CrowdloanRewards::claim(RuntimeOrigin::signed(
			reward_account.clone()
		)));

		let balance_after_first = Balances::free_balance(&reward_account);

		// Second claim at 50% vesting
		run_to_block(50);
		assert_ok!(CrowdloanRewards::claim(RuntimeOrigin::signed(
			reward_account.clone()
		)));

		let balance_after_second = Balances::free_balance(&reward_account);

		// Third claim at 100% vesting
		run_to_block(100);
		assert_ok!(CrowdloanRewards::claim(RuntimeOrigin::signed(
			reward_account.clone()
		)));

		let final_balance = Balances::free_balance(&reward_account);

		// Should have received all rewards by the end
		assert_eq!(
			final_balance,
			initial_balance + total_reward - initialization_payment
		);

		// Each claim should increase balance
		assert!(balance_after_first > initial_balance);
		assert!(balance_after_second > balance_after_first);
		assert!(final_balance > balance_after_second);

		// Final check: all rewards should be claimed
		let reward_info = AccountsPayable::<Test>::get(&reward_account).unwrap();
		assert_eq!(reward_info.claimed_reward, total_reward);
	});
}

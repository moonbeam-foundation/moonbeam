// Copyright 2019-2025 PureStake Inc.
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

use frame_support::assert_ok;

use sp_std::boxed::Box;
use xcm::latest::prelude::{AccountKey20, Location, Parachain, WeightLimit};
use xcm::VersionedLocation;

use xcm_simulator::TestExt;

use crate::{xcm_mock::*, xcm_testing::helpers::*};

#[test]
fn evm_account_receiving_assets_should_handle_sufficients_ref_count() {
	reset_test_environment();

	let mut sufficient_account = [0u8; 20];
	sufficient_account[0..20].copy_from_slice(&evm_account()[..]);

	let evm_account_id = parachain::AccountId::from(sufficient_account);

	// Evm account is self sufficient
	ParaA::execute_with(|| {
		assert_eq!(parachain::System::account(evm_account_id).sufficients, 1);
	});

	register_relay_asset();

	// Actually send relay asset to parachain
	let dest: Location = AccountKey20 {
		network: None,
		key: sufficient_account,
	}
	.into();
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::limited_reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::from(dest.clone()).clone().into()),
			Box::new(([], 123).into()),
			0,
			WeightLimit::Unlimited
		));
	});

	// Evm account sufficient ref count increased by 1.
	ParaA::execute_with(|| {
		// TODO: since the suicided logic was introduced the data of the smart contract is not
		// removed, it will have to be updated in a future release when there is the ability to
		// remove contract data
		// assert_eq!(parachain::System::account(evm_account_id).sufficients, 2);
	});

	ParaA::execute_with(|| {
		// Remove the account from the evm context.
		parachain::EVM::remove_account(&evm_account());
		// Evm account sufficient ref count decreased by 1.
		// TODO: since the suicided logic was introduced the data of the smart contract is not
		// removed, it will have to be updated in a future release when there is the ability to
		// remove contract data
		// assert_eq!(parachain::System::account(evm_account_id).sufficients, 1);
	});
}

#[test]
fn empty_account_should_not_be_reset() {
	reset_test_environment();

	// Test account has nonce 1 on genesis.
	let mut sufficient_account = [0u8; 20];
	sufficient_account[0..20].copy_from_slice(&evm_account()[..]);

	let evm_account_id = parachain::AccountId::from(sufficient_account);

	let source_id = register_relay_asset_non_sufficient();

	// Send native token to evm_account
	ParaA::execute_with(|| {
		assert_ok!(ParaBalances::transfer_allow_death(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			evm_account_id,
			100
		));
	});

	// Actually send relay asset to parachain
	let dest: Location = AccountKey20 {
		network: None,
		key: sufficient_account,
	}
	.into();
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::limited_reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::from(dest.clone()).clone().into()),
			Box::new(([], 123).into()),
			0,
			WeightLimit::Unlimited
		));
	});

	ParaA::execute_with(|| {
		// Empty the assets from the account.
		// As this makes the account go below the `min_balance`, the account is considered dead
		// at eyes of pallet-assets, and the consumer reference is decreased by 1 and is now Zero.
		assert_ok!(parachain::Assets::transfer(
			parachain::RuntimeOrigin::signed(evm_account_id),
			source_id,
			PARAALICE.into(),
			123
		));
		// Verify account asset balance is Zero.
		assert_eq!(
			parachain::Assets::balance(source_id, &evm_account_id.into()),
			0
		);
		// Because we no longer have consumer references, we can set the balance to Zero.
		// This would reset the account if our ED were to be > than Zero.
		assert_ok!(ParaBalances::force_set_balance(
			parachain::RuntimeOrigin::root(),
			evm_account_id,
			0,
		));
		// Verify account native balance is Zero.
		assert_eq!(ParaBalances::free_balance(&evm_account_id), 0);
		// Remove the account from the evm context.
		// This decreases the sufficients reference by 1 and now is Zero.
		parachain::EVM::remove_account(&evm_account());
		// Verify reference count.
		let account = parachain::System::account(evm_account_id);
		assert_eq!(account.sufficients, 0);
		assert_eq!(account.consumers, 0);
		assert_eq!(account.providers, 1);
		// We expect the account to be alive in a Zero ED context.
		assert_eq!(parachain::System::account_nonce(evm_account_id), 1);
	});
}

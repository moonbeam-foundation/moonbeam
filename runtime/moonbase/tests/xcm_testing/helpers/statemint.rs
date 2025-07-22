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

//! Statemint-specific helpers for XCM tests

use crate::xcm_mock::{
	parachain, statemint_like, AssetManager, ParaA, Statemint, StatemintAssets, StatemintBalances,
	StatemintChainPalletXcm, XcmWeightTrader, RELAYALICE,
};
use crate::xcm_testing::add_supported_asset;
use frame_support::{assert_ok, traits::PalletInfoAccess};
use sp_runtime::AccountId32;
use sp_std::boxed::Box;
use xcm::latest::prelude::{GeneralIndex, Location, PalletInstance, Parachain, WeightLimit};
use xcm_executor::traits::ConvertLocation;
use xcm_simulator::TestExt;

use super::assets::setup_relay_asset_for_statemint;
use super::core::{account_key20_location, parachain_location, reset_test_environment};
use super::transfers::execute_relay_to_statemint_transfer;

// Statemint test setup struct and functions

#[derive(Clone)]
struct StatemintTestSetup {
	pub dest_para: Location,
	pub sov_account: crate::xcm_mock::statemint_like::AccountId,
	pub parachain_beneficiary: Location,
	pub statemint_beneficiary: Location,
}

pub fn setup_statemint_test_environment() -> StatemintTestSetup {
	reset_test_environment();

	let dest_para = parachain_location(1);
	let sov = xcm_builder::SiblingParachainConvertsVia::<
		polkadot_parachain::primitives::Sibling,
		crate::xcm_mock::statemint_like::AccountId,
	>::convert_location(&dest_para)
	.unwrap();

	let parachain_beneficiary = account_key20_location(crate::xcm_mock::PARAALICE);
	let statemint_beneficiary: Location = xcm::latest::prelude::Junction::AccountId32 {
		network: None,
		id: crate::xcm_mock::RELAYALICE.into(),
	}
	.into();

	StatemintTestSetup {
		dest_para,
		sov_account: sov,
		parachain_beneficiary,
		statemint_beneficiary,
	}
}

// Statemint asset creation and management helpers

pub fn setup_statemint_asset(
	asset_id: u128,
	mint_amount: u128,
	sov_account: crate::xcm_mock::statemint_like::AccountId,
) {
	Statemint::execute_with(|| {
		// Set prefix
		statemint_like::PrefixChanger::set_prefix(
			PalletInstance(<StatemintAssets as PalletInfoAccess>::index() as u8).into(),
		);

		// Create and mint asset
		assert_ok!(StatemintAssets::create(
			statemint_like::RuntimeOrigin::signed(RELAYALICE),
			asset_id,
			RELAYALICE,
			1
		));

		assert_ok!(StatemintAssets::mint(
			statemint_like::RuntimeOrigin::signed(RELAYALICE),
			asset_id,
			RELAYALICE,
			mint_amount
		));

		// Transfer native tokens for fees
		assert_ok!(StatemintBalances::transfer_allow_death(
			statemint_like::RuntimeOrigin::signed(RELAYALICE),
			sov_account,
			100000000000000
		));
	});
}

pub fn create_statemint_asset_location(
	asset_index: u128,
) -> (
	Location,
	crate::xcm_mock::parachain::AssetType,
	crate::xcm_mock::parachain::AssetId,
) {
	let location = Location::new(
		1,
		[
			Parachain(1000u32),
			PalletInstance(5u8),
			GeneralIndex(asset_index),
		],
	);

	let asset_type: crate::xcm_mock::parachain::AssetType = location
		.clone()
		.try_into()
		.expect("Location conversion to AssetType should succeed");
	let asset_id: crate::xcm_mock::parachain::AssetId = asset_type.clone().into();

	(location, asset_type, asset_id)
}

pub fn register_statemint_asset_on_para(
	asset_location: crate::xcm_mock::parachain::AssetType,
	name: &[u8],
	symbol: &[u8],
) {
	let asset_metadata = parachain::AssetMetadata {
		name: name.to_vec(),
		symbol: symbol.to_vec(),
		decimals: 12,
	};

	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			asset_location.clone(),
			asset_metadata,
			1u128,
			true
		));
		assert_ok!(add_supported_asset(asset_location, 0));
	});
}

// Statemint-specific transfer helpers

pub fn execute_statemint_to_para_dot_transfer(
	setup: &StatemintTestSetup,
	from_account: crate::xcm_mock::statemint_like::AccountId,
	amount: u128,
) {
	Statemint::execute_with(|| {
		assert_ok!(StatemintChainPalletXcm::limited_reserve_transfer_assets(
			statemint_like::RuntimeOrigin::signed(from_account),
			Box::new(setup.dest_para.clone().into()),
			Box::new(xcm::VersionedLocation::from(setup.parachain_beneficiary.clone()).into()),
			Box::new((Location::parent(), amount).into()),
			0,
			WeightLimit::Unlimited
		));
	});
}

pub fn execute_statemint_asset_transfer(
	setup: &StatemintTestSetup,
	from_account: crate::xcm_mock::statemint_like::AccountId,
	asset_id: u128,
	amount: u128,
) {
	Statemint::execute_with(|| {
		assert_ok!(StatemintChainPalletXcm::limited_reserve_transfer_assets(
			statemint_like::RuntimeOrigin::signed(from_account),
			Box::new(setup.dest_para.clone().into()),
			Box::new(xcm::VersionedLocation::from(setup.parachain_beneficiary.clone()).into()),
			Box::new(
				(
					[
						PalletInstance(<StatemintAssets as PalletInfoAccess>::index() as u8),
						GeneralIndex(asset_id),
					],
					amount
				)
					.into()
			),
			0,
			WeightLimit::Unlimited
		));
	});
}

pub fn execute_statemint_to_para_transfer_with_balance_check(
	from_account: [u8; 32],
	to_account: [u8; 20],
	amount: u128,
) {
	Statemint::execute_with(|| {
		let account_id = AccountId32::from(from_account);
		let previous_balance = StatemintBalances::free_balance(&account_id);

		assert_ok!(StatemintChainPalletXcm::limited_reserve_transfer_assets(
			statemint_like::RuntimeOrigin::signed(account_id.clone()),
			Box::new(parachain_location(1).into()),
			Box::new(
				xcm::VersionedLocation::from(account_key20_location(to_account))
					.clone()
					.into()
			),
			Box::new((Location::parent(), amount).into()),
			0,
			WeightLimit::Unlimited
		));

		// Assert that the amount was deducted from sender's account
		assert_eq!(
			StatemintBalances::free_balance(&account_id),
			previous_balance - amount
		);
	});
}

// Complete multi-asset test setup (USDC + DOT scenario)
pub fn setup_multi_asset_statemint_test(
	asset_id: u128,
) -> (
	StatemintTestSetup,
	crate::xcm_mock::parachain::AssetId,
	crate::xcm_mock::parachain::AssetId,
) {
	let setup = setup_statemint_test_environment();
	let (_relay_location, source_relay_id) = setup_relay_asset_for_statemint();

	// Setup USDC asset
	let (_, usdc_location, usdc_asset_id) = create_statemint_asset_location(asset_id);
	register_statemint_asset_on_para(usdc_location.clone(), b"USDC", b"USDC");

	// Setup statemint asset and native token transfers
	setup_statemint_asset(asset_id, 300000000000000, setup.sov_account.clone());

	// Setup ParaA weight trader
	ParaA::execute_with(|| {
		XcmWeightTrader::set_asset_price(Location::parent(), 0u128);
		let statemint_asset = Location::new(
			1,
			[
				Parachain(1000u32),
				PalletInstance(5u8),
				GeneralIndex(asset_id),
			],
		);
		XcmWeightTrader::set_asset_price(statemint_asset, 0u128);
	});

	// Execute relay->statemint transfer
	execute_relay_to_statemint_transfer(setup.statemint_beneficiary.clone(), 200);

	// Initial setup transfers
	Statemint::execute_with(|| {
		// Note: Balance may be less than INITIAL_BALANCE + 200 due to teleport fees
		assert_ok!(StatemintBalances::transfer_allow_death(
			statemint_like::RuntimeOrigin::signed(RELAYALICE),
			setup.sov_account.clone(),
			110000000000000
		));
	});

	// Transfer DOTs to ParaA
	execute_statemint_to_para_dot_transfer(&setup, RELAYALICE, 200);

	// Transfer USDC to ParaA
	execute_statemint_asset_transfer(&setup, RELAYALICE, asset_id, 125);

	(setup, source_relay_id, usdc_asset_id)
}

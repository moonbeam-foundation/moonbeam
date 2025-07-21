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

use frame_support::{assert_ok, traits::PalletInfoAccess, weights::Weight};
use moonbase_runtime::xcm_config::AssetType;

use crate::{
	xcm_mock::{parachain::PolkadotXcm, *},
	xcm_testing::{add_supported_asset, currency_to_asset, helpers::*},
};
use sp_std::boxed::Box;
use xcm::VersionedLocation;
use xcm::{
	latest::prelude::{
		AccountId32, Asset, AssetId, Assets as XcmAssets, Fungibility, GeneralIndex, Junction,
		Location, PalletInstance, Parachain, WeightLimit,
	},
	VersionedAssets,
};
use xcm_executor::traits::ConvertLocation;
use xcm_primitives::split_location_into_chain_part_and_beneficiary;
use xcm_simulator::TestExt;

#[test]
fn test_statemint_like() {
	let setup = setup_statemint_test_environment();
	let (_, source_location, source_id) = create_statemint_asset_location(0);

	// Register asset on ParaA
	register_statemint_asset_on_para(source_location, b"StatemintToken", b"StatemintToken");

	// Setup asset on Statemint and transfer to ParaA
	setup_statemint_asset(0, 300000000000000, setup.sov_account.clone());

	Statemint::execute_with(|| {
		// Send asset to ParaA
		assert_ok!(StatemintChainPalletXcm::limited_reserve_transfer_assets(
			statemint_like::RuntimeOrigin::signed(RELAYALICE),
			Box::new(setup.dest_para.clone().into()),
			Box::new(VersionedLocation::from(setup.parachain_beneficiary.clone()).into()),
			Box::new(
				(
					[
						xcm::latest::prelude::PalletInstance(
							<StatemintAssets as PalletInfoAccess>::index() as u8
						),
						xcm::latest::prelude::GeneralIndex(0),
					],
					123
				)
					.into()
			),
			0,
			WeightLimit::Unlimited
		));
	});

	assert_asset_balance(&PARAALICE, source_id, 123);
}

#[test]
fn send_statemint_asset_from_para_a_to_statemint_with_relay_fee() {
	reset_test_environment();

	// Setup relay asset using helper
	let (_relay_location, source_relay_id) = setup_relay_asset_for_statemint();

	// Statemint asset
	let statemint_asset = Location::new(
		1,
		[
			Parachain(1000u32),
			PalletInstance(5u8),
			GeneralIndex(10u128),
		],
	);
	let statemint_location_asset: AssetType = statemint_asset
		.clone()
		.try_into()
		.expect("Location convertion to AssetType should succeed");
	let source_statemint_asset_id: parachain::AssetId = statemint_location_asset.clone().into();

	let asset_metadata_statemint_asset = parachain::AssetMetadata {
		name: b"USDC".to_vec(),
		symbol: b"USDC".to_vec(),
		decimals: 12,
	};

	let dest_para = parachain_location(1);

	let sov = xcm_builder::SiblingParachainConvertsVia::<
		polkadot_parachain::primitives::Sibling,
		statemint_like::AccountId,
	>::convert_location(&dest_para)
	.unwrap();

	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			statemint_location_asset.clone(),
			asset_metadata_statemint_asset,
			1u128,
			true
		));
		assert_ok!(add_supported_asset(statemint_location_asset, 0));
	});

	let parachain_beneficiary_from_relay = account_key20_location(PARAALICE);

	// Send relay chain asset to Alice in Parachain A
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::limited_reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(
				VersionedLocation::from(parachain_beneficiary_from_relay)
					.clone()
					.into()
			),
			Box::new(([], 200).into()),
			0,
			WeightLimit::Unlimited
		));
	});

	Statemint::execute_with(|| {
		// Set new prefix
		statemint_like::PrefixChanger::set_prefix(
			PalletInstance(<StatemintAssets as PalletInfoAccess>::index() as u8).into(),
		);

		assert_ok!(StatemintAssets::create(
			statemint_like::RuntimeOrigin::signed(RELAYALICE),
			10,
			RELAYALICE,
			1
		));

		assert_ok!(StatemintAssets::mint(
			statemint_like::RuntimeOrigin::signed(RELAYALICE),
			10,
			RELAYALICE,
			300000000000000
		));

		// Send some native statemint tokens to sovereign for fees.
		// We can't pay fees with USDC as the asset is minted as non-sufficient.
		assert_ok!(StatemintBalances::transfer_allow_death(
			statemint_like::RuntimeOrigin::signed(RELAYALICE),
			sov,
			100000000000000
		));

		// Send statemint USDC asset to Alice in Parachain A
		let parachain_beneficiary_from_statemint = account_key20_location(PARAALICE);

		// Send with new prefix
		assert_ok!(StatemintChainPalletXcm::limited_reserve_transfer_assets(
			statemint_like::RuntimeOrigin::signed(RELAYALICE),
			Box::new(parachain_location(1).into()),
			Box::new(
				VersionedLocation::from(parachain_beneficiary_from_statemint)
					.clone()
					.into()
			),
			Box::new(
				(
					[
						xcm::latest::prelude::PalletInstance(
							<StatemintAssets as PalletInfoAccess>::index() as u8
						),
						GeneralIndex(10),
					],
					125
				)
					.into()
			),
			0,
			WeightLimit::Unlimited
		));
	});

	let statemint_beneficiary = Location {
		parents: 1,
		interior: [
			Parachain(1000),
			AccountId32 {
				network: None,
				id: RELAYBOB.into(),
			},
		]
		.into(),
	};

	ParaA::execute_with(|| {
		// Alice has received 125 USDC
		assert_eq!(
			Assets::balance(source_statemint_asset_id, &PARAALICE.into()),
			125
		);

		// Alice has received 200 Relay assets
		assert_eq!(Assets::balance(source_relay_id, &PARAALICE.into()), 200);
	});

	Statemint::execute_with(|| {
		// Check that BOB's balance is empty before the transfer
		assert_eq!(StatemintAssets::account_balances(RELAYBOB), vec![]);
	});

	let (chain_part, beneficiary) =
		split_location_into_chain_part_and_beneficiary(statemint_beneficiary).unwrap();

	// Transfer USDC from Parachain A to Statemint using Relay asset as fee
	ParaA::execute_with(|| {
		let asset = currency_to_asset(
			parachain::CurrencyId::ForeignAsset(source_statemint_asset_id),
			100,
		);
		let asset_fee =
			currency_to_asset(parachain::CurrencyId::ForeignAsset(source_relay_id), 100);
		assert_ok!(PolkadotXcm::transfer_assets(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(VersionedLocation::from(chain_part)),
			Box::new(VersionedLocation::from(beneficiary)),
			Box::new(VersionedAssets::from(vec![asset_fee, asset])),
			0,
			WeightLimit::Limited(Weight::from_parts(80_000_000u64, 100_000u64))
		));
	});

	ParaA::execute_with(|| {
		// Alice has 100 USDC less
		assert_eq!(
			Assets::balance(source_statemint_asset_id, &PARAALICE.into()),
			25
		);

		// Alice has 100 relay asset less
		assert_eq!(Assets::balance(source_relay_id, &PARAALICE.into()), 100);
	});

	Statemint::execute_with(|| {
		// Check that BOB received 100 USDC on statemint
		assert_eq!(StatemintAssets::account_balances(RELAYBOB), vec![(10, 100)]);
	});
}

#[test]
fn send_dot_from_moonbeam_to_statemint_via_xtokens_transfer() {
	let setup = setup_statemint_test_environment();
	let (_relay_location, source_relay_id) = setup_relay_asset_for_statemint();

	ParaA::execute_with(|| {
		XcmWeightTrader::set_asset_price(Location::parent(), 0u128);
	});

	// Execute relay->statemint->para transfer sequence
	execute_relay_to_statemint_transfer(setup.statemint_beneficiary.clone(), 200);

	Statemint::execute_with(|| {
		assert_eq!(
			StatemintBalances::free_balance(RELAYALICE),
			INITIAL_BALANCE + 200
		);
		assert_ok!(StatemintBalances::transfer_allow_death(
			statemint_like::RuntimeOrigin::signed(RELAYALICE),
			setup.sov_account.clone(),
			110000000000000
		));
	});

	execute_statemint_to_para_dot_transfer(&setup, RELAYALICE, 200);
	assert_asset_balance(&PARAALICE, source_relay_id, 200);

	// ParaA to Statemint transfer to RELAYBOB
	let dest = Location::new(
		1,
		[
			Parachain(1000),
			AccountId32 {
				network: None,
				id: RELAYBOB.into(),
			},
		],
	);
	let (chain_part, beneficiary) = split_location_into_chain_part_and_beneficiary(dest).unwrap();

	ParaA::execute_with(|| {
		let asset = currency_to_asset(parachain::CurrencyId::ForeignAsset(source_relay_id), 100);
		assert_ok!(PolkadotXcm::transfer_assets(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(VersionedLocation::from(chain_part)),
			Box::new(VersionedLocation::from(beneficiary)),
			Box::new(VersionedAssets::from(vec![asset])),
			0,
			medium_transfer_weight()
		));
		assert_eq!(Assets::balance(source_relay_id, &PARAALICE.into()), 100);
	});

	Statemint::execute_with(|| {
		assert_eq!(
			StatemintBalances::free_balance(RELAYBOB),
			INITIAL_BALANCE + 100
		);
	});

	// Send back tokens from AH to ParaA from Bob's account
	execute_statemint_to_para_transfer_with_balance_check(RELAYBOB.into(), PARAALICE, 100);

	ParaA::execute_with(|| {
		assert_eq!(Assets::balance(source_relay_id, &PARAALICE.into()), 200);
	});
}

#[test]
fn send_dot_from_moonbeam_to_statemint_via_xtokens_transfer_with_fee() {
	let setup = setup_statemint_test_environment();
	let (_relay_location, source_relay_id) = setup_relay_asset_for_statemint();

	ParaA::execute_with(|| {
		XcmWeightTrader::set_asset_price(Location::parent(), 0u128);
	});

	// Execute relay->statemint->para transfer sequence
	execute_relay_to_statemint_transfer(setup.statemint_beneficiary.clone(), 200);

	Statemint::execute_with(|| {
		assert_eq!(
			StatemintBalances::free_balance(RELAYALICE),
			INITIAL_BALANCE + 200
		);
		assert_ok!(StatemintBalances::transfer_allow_death(
			statemint_like::RuntimeOrigin::signed(RELAYALICE),
			setup.sov_account.clone(),
			110000000000000
		));
	});

	execute_statemint_to_para_dot_transfer(&setup, RELAYALICE, 200);
	assert_asset_balance(&PARAALICE, source_relay_id, 200);

	// ParaA to Statemint transfer with fee
	let dest = Location::new(
		1,
		[
			Parachain(1000),
			AccountId32 {
				network: None,
				id: RELAYBOB.into(),
			},
		],
	);
	let (chain_part, beneficiary) = split_location_into_chain_part_and_beneficiary(dest).unwrap();

	ParaA::execute_with(|| {
		let asset = currency_to_asset(parachain::CurrencyId::ForeignAsset(source_relay_id), 100);
		let asset_fee = currency_to_asset(parachain::CurrencyId::ForeignAsset(source_relay_id), 10);
		assert_ok!(PolkadotXcm::transfer_assets(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(VersionedLocation::from(chain_part)),
			Box::new(VersionedLocation::from(beneficiary)),
			Box::new(VersionedAssets::from(vec![asset_fee, asset])),
			0,
			medium_transfer_weight()
		));
		assert_eq!(Assets::balance(source_relay_id, &PARAALICE.into()), 90);
	});

	Statemint::execute_with(|| {
		assert_eq!(
			StatemintBalances::free_balance(RELAYBOB),
			INITIAL_BALANCE + 110
		);
	});

	execute_statemint_to_para_transfer_with_balance_check(RELAYBOB.into(), PARAALICE, 100);

	ParaA::execute_with(|| {
		assert_eq!(Assets::balance(source_relay_id, &PARAALICE.into()), 190);
	});
}

#[test]
fn send_dot_from_moonbeam_to_statemint_via_xtokens_transfer_multiasset() {
	reset_test_environment();

	// Setup relay asset using helper
	let (_relay_location, source_relay_id) = setup_relay_asset_for_statemint();

	let dest_para = parachain_location(1);

	let sov = xcm_builder::SiblingParachainConvertsVia::<
		polkadot_parachain::primitives::Sibling,
		statemint_like::AccountId,
	>::convert_location(&dest_para)
	.unwrap();

	ParaA::execute_with(|| {
		XcmWeightTrader::set_asset_price(Location::parent(), 0u128);
	});

	let parachain_beneficiary_absolute = account_key20_location(PARAALICE);

	let statemint_beneficiary_absolute: Location = Junction::AccountId32 {
		network: None,
		id: RELAYALICE.into(),
	}
	.into();

	// First we send relay chain asset to Alice in AssetHub (via teleport)
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::limited_teleport_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1000).into()),
			Box::new(
				VersionedLocation::from(statemint_beneficiary_absolute)
					.clone()
					.into()
			),
			Box::new(([], 200).into()),
			0,
			WeightLimit::Unlimited
		));
	});

	// Send DOTs from AssetHub to ParaA (Moonbeam)
	Statemint::execute_with(|| {
		// Check Alice received 200 tokens on AssetHub
		assert_eq!(
			StatemintBalances::free_balance(RELAYALICE),
			INITIAL_BALANCE + 200
		);

		assert_ok!(StatemintBalances::transfer_allow_death(
			statemint_like::RuntimeOrigin::signed(RELAYALICE),
			sov,
			110000000000000
		));

		// Now send those tokens to ParaA
		assert_ok!(StatemintChainPalletXcm::limited_reserve_transfer_assets(
			statemint_like::RuntimeOrigin::signed(RELAYALICE),
			Box::new(parachain_location(1).into()),
			Box::new(
				VersionedLocation::from(parachain_beneficiary_absolute.clone())
					.clone()
					.into()
			),
			Box::new((Location::parent(), 200).into()),
			0,
			WeightLimit::Unlimited
		));
	});

	// Alice should have received the DOTs
	assert_asset_balance(&PARAALICE, source_relay_id, 200);

	let dest = Location::new(
		1,
		[
			Parachain(1000),
			AccountId32 {
				network: None,
				id: RELAYBOB.into(),
			},
		],
	);

	let (chain_part, beneficiary) = split_location_into_chain_part_and_beneficiary(dest).unwrap();
	let asset = Asset {
		id: AssetId(Location::parent()),
		fun: Fungibility::Fungible(100),
	};
	// Finally we test that we are able to send back the DOTs to AssetHub from the ParaA
	ParaA::execute_with(|| {
		assert_ok!(PolkadotXcm::transfer_assets(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(VersionedLocation::from(chain_part)),
			Box::new(VersionedLocation::from(beneficiary)),
			Box::new(VersionedAssets::from(asset)),
			0,
			medium_transfer_weight()
		));

		assert_eq!(Assets::balance(source_relay_id, &PARAALICE.into()), 100);
	});

	Statemint::execute_with(|| {
		// Check that Bob received the tokens back in AssetHub
		assert_eq!(
			StatemintBalances::free_balance(RELAYBOB),
			INITIAL_BALANCE + 100
		);
	});

	// Send back tokens from AH to ParaA from Bob's account
	execute_statemint_to_para_transfer_with_balance_check(RELAYBOB.into(), PARAALICE, 100);

	ParaA::execute_with(|| {
		// Alice should have received 100 DOTs
		assert_eq!(Assets::balance(source_relay_id, &PARAALICE.into()), 200);
	});
}

#[test]
fn send_dot_from_moonbeam_to_statemint_via_xtokens_transfer_multicurrencies() {
	let (_setup, source_relay_id, source_statemint_asset_id) = setup_multi_asset_statemint_test(10);

	// Verify initial balances after setup
	ParaA::execute_with(|| {
		assert_eq!(Assets::balance(source_relay_id, &PARAALICE.into()), 200);
		assert_eq!(
			Assets::balance(source_statemint_asset_id, &PARAALICE.into()),
			125
		);
	});

	// ParaA to Statemint multi-asset transfer
	let dest = Location::new(
		1,
		[
			Parachain(1000),
			AccountId32 {
				network: None,
				id: RELAYBOB.into(),
			},
		],
	);
	let (chain_part, beneficiary) = split_location_into_chain_part_and_beneficiary(dest).unwrap();

	ParaA::execute_with(|| {
		let asset = currency_to_asset(
			parachain::CurrencyId::ForeignAsset(source_statemint_asset_id),
			100,
		);
		let asset_fee =
			currency_to_asset(parachain::CurrencyId::ForeignAsset(source_relay_id), 100);
		assert_ok!(PolkadotXcm::transfer_assets(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(VersionedLocation::from(chain_part)),
			Box::new(VersionedLocation::from(beneficiary)),
			Box::new(VersionedAssets::from(vec![asset_fee, asset])),
			0,
			WeightLimit::Limited(Weight::from_parts(80_000_000u64, 100_000u64))
		));
		assert_eq!(Assets::balance(source_relay_id, &PARAALICE.into()), 100);
	});

	// Verify transfers to Bob
	Statemint::execute_with(|| {
		assert_eq!(
			StatemintBalances::free_balance(RELAYBOB),
			INITIAL_BALANCE + 100
		);
		assert_eq!(StatemintAssets::account_balances(RELAYBOB), vec![(10, 100)]);
	});

	// Return transfer from Statemint to ParaA
	execute_statemint_to_para_transfer_with_balance_check(RELAYBOB.into(), PARAALICE, 100);

	ParaA::execute_with(|| {
		assert_eq!(Assets::balance(source_relay_id, &PARAALICE.into()), 200);
	});
}

#[test]
fn send_dot_from_moonbeam_to_statemint_via_xtokens_transfer_multiassets() {
	let (_setup, source_relay_id, source_statemint_asset_id) = setup_multi_asset_statemint_test(10);

	// Verify initial balances
	ParaA::execute_with(|| {
		assert_eq!(Assets::balance(source_relay_id, &PARAALICE.into()), 200);
		assert_eq!(
			Assets::balance(source_statemint_asset_id, &PARAALICE.into()),
			125
		);
	});

	// Multi-asset transfer using Asset structs
	let dest = Location::new(
		1,
		[
			Parachain(1000),
			AccountId32 {
				network: None,
				id: RELAYBOB.into(),
			},
		],
	);
	let (chain_part, beneficiary) = split_location_into_chain_part_and_beneficiary(dest).unwrap();

	let statemint_asset = Location::new(
		1,
		[
			Parachain(1000u32),
			PalletInstance(5u8),
			GeneralIndex(10u128),
		],
	);
	let statemint_asset_to_send = Asset {
		id: AssetId(statemint_asset),
		fun: Fungibility::Fungible(100),
	};
	let relay_asset_to_send = Asset {
		id: AssetId(Location::parent()),
		fun: Fungibility::Fungible(100),
	};
	let assets_to_send: XcmAssets =
		XcmAssets::from(vec![statemint_asset_to_send, relay_asset_to_send.clone()]);

	// Verify asset ordering for fees
	assert_eq!(assets_to_send.get(0).unwrap(), &relay_asset_to_send);

	ParaA::execute_with(|| {
		assert_ok!(PolkadotXcm::transfer_assets(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(VersionedLocation::from(chain_part)),
			Box::new(VersionedLocation::from(beneficiary)),
			Box::new(VersionedAssets::from(assets_to_send)),
			0,
			WeightLimit::Limited(Weight::from_parts(80_000_000u64, 100_000u64))
		));
		assert_eq!(Assets::balance(source_relay_id, &PARAALICE.into()), 100);
	});

	// Verify Bob received both assets
	Statemint::execute_with(|| {
		assert_eq!(
			StatemintBalances::free_balance(RELAYBOB),
			INITIAL_BALANCE + 100
		);
		assert_eq!(StatemintAssets::account_balances(RELAYBOB), vec![(10, 100)]);
	});

	// Return transfer
	execute_statemint_to_para_transfer_with_balance_check(RELAYBOB.into(), PARAALICE, 100);

	ParaA::execute_with(|| {
		assert_eq!(Assets::balance(source_relay_id, &PARAALICE.into()), 200);
	});
}

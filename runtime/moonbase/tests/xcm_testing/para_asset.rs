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

use crate::xcm_mock::{parachain::PolkadotXcm, *};
use crate::xcm_testing::{add_supported_asset, currency_to_asset, helpers::*};
use frame_support::assert_ok;
use moonbase_runtime::xcm_config::AssetType;
use sp_weights::Weight;
use xcm::{
	latest::prelude::{
		AccountKey20, All, BuyExecution, ClearOrigin, DepositAsset, Limited, Location,
		PalletInstance, Parachain, WeightLimit, WithdrawAsset, Xcm,
	},
	VersionedAssets, VersionedLocation,
};
use xcm_primitives::{split_location_into_chain_part_and_beneficiary, DEFAULT_PROOF_SIZE};
use xcm_simulator::TestExt;

#[test]
fn send_para_a_asset_to_para_b() {
	MockNet::reset();

	// this represents the asset in paraA
	let para_a_balances = Location::new(1, [Parachain(1), PalletInstance(1u8)]);
	let source_location: AssetType = para_a_balances
		.try_into()
		.expect("Location convertion to AssetType should succeed");
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"ParaAToken".to_vec(),
		symbol: b"ParaA".to_vec(),
		decimals: 18,
	};

	// Register asset in paraB. Free execution
	ParaB::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata,
			1u128,
			true
		));
		assert_ok!(add_supported_asset(source_location, 0));
	});

	// Send para A asset from para A to para B using helper
	let asset = currency_to_asset(parachain::CurrencyId::SelfReserve, 100);
	execute_transfer_to_para(
		PARAALICE,
		VersionedAssets::from(vec![asset]),
		2,
		PARAALICE,
		Some(standard_heavy_weight()),
	);

	// Native token is substracted in paraA
	assert_native_balance_decreased_by(&PARAALICE, INITIAL_BALANCE, 100);

	// Asset is minted in paraB
	ParaB::execute_with(|| {
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 100);
	});
}

#[test]
fn send_para_a_asset_from_para_b_to_para_c() {
	MockNet::reset();

	// Represents para A asset
	let para_a_balances = Location::new(1, [Parachain(1), PalletInstance(1u8)]);
	let source_location: AssetType = para_a_balances
		.try_into()
		.expect("Location convertion to AssetType should succeed");
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"ParaAToken".to_vec(),
		symbol: b"ParaA".to_vec(),
		decimals: 18,
	};

	// Register para A asset in parachain B. Free execution
	ParaB::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata.clone(),
			1u128,
			true
		));
		assert_ok!(add_supported_asset(source_location.clone(), 0));
	});

	// Register para A asset in parachain C. Free execution
	ParaC::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata,
			1u128,
			true
		));
		assert_ok!(add_supported_asset(source_location, 0));
	});

	// Send para A asset to para B using helper
	let asset = currency_to_asset(parachain::CurrencyId::SelfReserve, 100);
	execute_transfer_to_para(
		PARAALICE,
		VersionedAssets::from(vec![asset]),
		2,
		PARAALICE,
		Some(standard_transfer_weight()),
	);

	// Para A balances have been substracted
	assert_native_balance_decreased_by(&PARAALICE, INITIAL_BALANCE, 100);

	// Para B balances have been credited
	ParaB::execute_with(|| {
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 100);
	});

	// Send para A asset from para B to para C
	let dest = Location {
		parents: 1,
		interior: [
			Parachain(3),
			AccountKey20 {
				network: None,
				key: PARAALICE.into(),
			},
		]
		.into(),
	};
	let (chain_part, beneficiary) = split_location_into_chain_part_and_beneficiary(dest).unwrap();

	ParaB::execute_with(|| {
		let asset = currency_to_asset(parachain::CurrencyId::ForeignAsset(source_id), 100);
		assert_ok!(PolkadotXcm::transfer_assets(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(VersionedLocation::from(chain_part)),
			Box::new(VersionedLocation::from(beneficiary)),
			Box::new(VersionedAssets::from(vec![asset])),
			0,
			WeightLimit::Limited(Weight::from_parts(80u64, DEFAULT_PROOF_SIZE))
		));
	});

	// The message passed through parachainA so we needed to pay since its the native token
	// The message passed through parachainA so we needed to pay since its the native token
	ParaC::execute_with(|| {
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 96);
	});
}

#[test]
fn send_para_a_asset_to_para_b_and_back_to_para_a() {
	MockNet::reset();

	// Para A asset
	let para_a_balances = Location::new(1, [Parachain(1), PalletInstance(1u8)]);
	let source_location: AssetType = para_a_balances
		.try_into()
		.expect("Location convertion to AssetType should succeed");
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"ParaAToken".to_vec(),
		symbol: b"ParaA".to_vec(),
		decimals: 18,
	};

	// Register para A asset in para B
	ParaB::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata,
			1u128,
			true
		));
		assert_ok!(add_supported_asset(source_location, 0));
	});

	// Send para A asset to para B
	let dest = Location {
		parents: 1,
		interior: [
			Parachain(2),
			AccountKey20 {
				network: None,
				key: PARAALICE.into(),
			},
		]
		.into(),
	};
	let (chain_part, beneficiary) = split_location_into_chain_part_and_beneficiary(dest).unwrap();

	ParaA::execute_with(|| {
		let asset = currency_to_asset(parachain::CurrencyId::SelfReserve, 100);
		assert_ok!(PolkadotXcm::transfer_assets(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(VersionedLocation::from(chain_part)),
			Box::new(VersionedLocation::from(beneficiary)),
			Box::new(VersionedAssets::from(vec![asset])),
			0,
			WeightLimit::Limited(Weight::from_parts(80u64, DEFAULT_PROOF_SIZE))
		));
	});

	// Balances have been subtracted
	ParaA::execute_with(|| {
		assert_eq!(
			ParaBalances::free_balance(&PARAALICE.into()),
			INITIAL_BALANCE - 100
		);
	});

	// Para B balances have been credited
	ParaB::execute_with(|| {
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 100);
	});

	// Send back para A asset to para A
	let dest = Location {
		parents: 1,
		interior: [
			Parachain(1),
			AccountKey20 {
				network: None,
				key: PARAALICE.into(),
			},
		]
		.into(),
	};
	let (chain_part, beneficiary) = split_location_into_chain_part_and_beneficiary(dest).unwrap();

	ParaB::execute_with(|| {
		let asset = currency_to_asset(parachain::CurrencyId::ForeignAsset(source_id), 100);
		assert_ok!(PolkadotXcm::transfer_assets(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(VersionedLocation::from(chain_part)),
			Box::new(VersionedLocation::from(beneficiary)),
			Box::new(VersionedAssets::from(vec![asset])),
			0,
			WeightLimit::Limited(Weight::from_parts(80u64, DEFAULT_PROOF_SIZE))
		));
	});

	ParaA::execute_with(|| {
		// Weight used is 4
		assert_eq!(
			ParaBalances::free_balance(&PARAALICE.into()),
			INITIAL_BALANCE - 4
		);
	});
}

#[test]
fn send_para_a_asset_to_para_b_and_back_to_para_a_with_new_reanchoring() {
	MockNet::reset();

	let para_a_balances = Location::new(1, [Parachain(1), PalletInstance(1u8)]);
	let source_location: AssetType = para_a_balances
		.try_into()
		.expect("Location convertion to AssetType should succeed");
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"ParaAToken".to_vec(),
		symbol: b"ParaA".to_vec(),
		decimals: 18,
	};

	ParaB::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata,
			1u128,
			true
		));
		assert_ok!(add_supported_asset(source_location, 0));
	});

	let dest = Location {
		parents: 1,
		interior: [
			Parachain(2),
			AccountKey20 {
				network: None,
				key: PARAALICE.into(),
			},
		]
		.into(),
	};
	let (chain_part, beneficiary) = split_location_into_chain_part_and_beneficiary(dest).unwrap();

	ParaA::execute_with(|| {
		let asset = currency_to_asset(parachain::CurrencyId::SelfReserve, 100);
		assert_ok!(PolkadotXcm::transfer_assets(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(VersionedLocation::from(chain_part)),
			Box::new(VersionedLocation::from(beneficiary)),
			Box::new(VersionedAssets::from(vec![asset])),
			0,
			WeightLimit::Limited(Weight::from_parts(80u64, DEFAULT_PROOF_SIZE))
		));
	});

	// Para A asset has been credited
	ParaA::execute_with(|| {
		assert_eq!(
			ParaBalances::free_balance(&PARAALICE.into()),
			INITIAL_BALANCE - 100
		);
	});

	ParaB::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 100);
	});

	// This time we will force the new reanchoring by manually sending the
	// Message through polkadotXCM pallet

	let dest = Location {
		parents: 1,
		interior: [Parachain(1)].into(),
	};

	let reanchored_para_a_balances = Location::new(0, [PalletInstance(1u8)]);

	let message = xcm::VersionedXcm::<()>::V5(Xcm(vec![
		WithdrawAsset((reanchored_para_a_balances.clone(), 100).into()),
		ClearOrigin,
		BuyExecution {
			fees: (reanchored_para_a_balances, 100).into(),
			weight_limit: Limited(80.into()),
		},
		DepositAsset {
			assets: All.into(),
			beneficiary: Location::new(
				0,
				[AccountKey20 {
					network: None,
					key: PARAALICE,
				}],
			),
		},
	]));
	ParaB::execute_with(|| {
		// Send a message to the sovereign account in ParaA to withdraw
		// and deposit asset
		assert_ok!(ParachainPalletXcm::send(
			parachain::RuntimeOrigin::root(),
			Box::new(dest.into()),
			Box::new(message),
		));
	});

	ParaA::execute_with(|| {
		// Weight used is 4
		assert_eq!(
			ParaBalances::free_balance(&PARAALICE.into()),
			INITIAL_BALANCE - 4
		);
	});
}

#[test]
fn send_para_a_asset_to_para_b_with_trader() {
	MockNet::reset();

	let para_a_balances = Location::new(1, [Parachain(1), PalletInstance(1u8)]);
	let source_location: AssetType = para_a_balances
		.try_into()
		.expect("Location convertion to AssetType should succeed");
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"ParaAToken".to_vec(),
		symbol: b"ParaA".to_vec(),
		decimals: 18,
	};

	ParaB::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata,
			1u128,
			true
		));
		assert_ok!(add_supported_asset(source_location, 2500000000000));
	});

	let dest = Location {
		parents: 1,
		interior: [
			Parachain(2),
			AccountKey20 {
				network: None,
				key: PARAALICE.into(),
			},
		]
		.into(),
	};
	let (chain_part, beneficiary) = split_location_into_chain_part_and_beneficiary(dest).unwrap();

	// In destination chain, we only need 4 weight
	// We put 10 weight, 6 of which should be refunded and 4 of which should go to treasury
	ParaA::execute_with(|| {
		let asset = currency_to_asset(parachain::CurrencyId::SelfReserve, 100);
		assert_ok!(PolkadotXcm::transfer_assets(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(VersionedLocation::from(chain_part)),
			Box::new(VersionedLocation::from(beneficiary)),
			Box::new(VersionedAssets::from(vec![asset])),
			0,
			WeightLimit::Limited(Weight::from_parts(10u64, DEFAULT_PROOF_SIZE))
		));
	});
	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(
			ParaBalances::free_balance(&PARAALICE.into()),
			INITIAL_BALANCE - 100
		);
	});

	// We are sending 100 tokens from para A.
	// Amount spent in fees is Units per second * weight / 1_000_000_000_000 (weight per second)
	// weight is 4 since we are executing 4 instructions with a unitweightcost of 1.
	// Units per second should be 2_500_000_000_000_000
	// Since we set 10 weight in destination chain, 25 will be charged upfront
	// 15 of those will be refunded, while 10 will go to treasury as the true weight used
	// will be 4
	ParaB::execute_with(|| {
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 90);
		// Fee should have been received by treasury
		assert_eq!(Assets::balance(source_id, &Treasury::account_id()), 10);
	});
}

#[test]
fn send_para_a_asset_to_para_b_with_trader_and_fee() {
	MockNet::reset();

	let para_a_balances = Location::new(1, [Parachain(1), PalletInstance(1u8)]);
	let source_location: AssetType = para_a_balances
		.try_into()
		.expect("Location convertion to AssetType should succeed");
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"ParaAToken".to_vec(),
		symbol: b"ParaA".to_vec(),
		decimals: 18,
	};

	ParaB::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata,
			1u128,
			true
		));
		// With these units per second, 80K weight convrets to 1 asset unit
		assert_ok!(add_supported_asset(source_location, 12500000));
	});

	let dest = Location {
		parents: 1,
		interior: [
			Parachain(2),
			AccountKey20 {
				network: None,
				key: PARAALICE.into(),
			},
		]
		.into(),
	};
	let (chain_part, beneficiary) = split_location_into_chain_part_and_beneficiary(dest).unwrap();
	// we use transfer_with_fee
	ParaA::execute_with(|| {
		let asset = currency_to_asset(parachain::CurrencyId::SelfReserve, 100);
		let asset_fee = currency_to_asset(parachain::CurrencyId::SelfReserve, 1);
		assert_ok!(PolkadotXcm::transfer_assets(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(VersionedLocation::from(chain_part)),
			Box::new(VersionedLocation::from(beneficiary)),
			Box::new(VersionedAssets::from(vec![asset_fee, asset])),
			0,
			WeightLimit::Limited(Weight::from_parts(800000u64, DEFAULT_PROOF_SIZE))
		));
	});
	ParaA::execute_with(|| {
		// 100 tokens transferred plus 1 taken from fees
		assert_eq!(
			ParaBalances::free_balance(&PARAALICE.into()),
			INITIAL_BALANCE - 100 - 1
		);
	});

	ParaB::execute_with(|| {
		// free execution, full amount received because the xcm instruction does not cost
		// what it is specified
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 101);
	});
}

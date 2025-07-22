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

use crate::xcm_mock::*;
use crate::{
	xcm_mock::parachain::PolkadotXcm,
	xcm_testing::{add_supported_asset, currency_to_asset},
};
use frame_support::{assert_ok, weights::Weight};
use moonbase_runtime::xcm_config::AssetType;
use sp_std::boxed::Box;
use xcm::{
	latest::prelude::{
		AccountKey20, Location, PalletInstance, Parachain, QueryResponse, Response, WeightLimit,
		Xcm,
	},
	VersionedAssets,
};
use xcm::{VersionedLocation, WrapVersion};
use xcm_primitives::{split_location_into_chain_part_and_beneficiary, DEFAULT_PROOF_SIZE};
use xcm_simulator::TestExt;

#[test]
fn test_automatic_versioning_on_runtime_upgrade_with_relay() {
	MockNet::reset();

	let source_location = parachain::AssetType::Xcm(xcm::v3::Location::parent());
	let asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};
	// register relay asset in parachain A and set XCM version to 1
	ParaA::execute_with(|| {
		parachain::XcmVersioner::set_version(1);
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata,
			1u128,
			true
		));
		assert_ok!(add_supported_asset(source_location, 0));
	});

	let response = Response::Version(2);
	let querier: Location = [].into();

	// This is irrelevant, nothing will be done with this message,
	// but we need to pass a message as an argument to trigger the storage change
	let mock_message: Xcm<()> = Xcm(vec![QueryResponse {
		query_id: 0,
		response,
		max_weight: Weight::zero(),
		querier: Some(querier),
	}]);
	// The router is mocked, and we cannot use WrapVersion in ChildParachainRouter. So we will force
	// it directly here
	// Actually send relay asset to parachain
	let dest: Location = AccountKey20 {
		network: None,
		key: PARAALICE,
	}
	.into();

	Relay::execute_with(|| {
		// This sets the default version, for not known destinations
		assert_ok!(RelayChainPalletXcm::force_default_xcm_version(
			relay_chain::RuntimeOrigin::root(),
			Some(3)
		));

		// Wrap version, which sets VersionedStorage
		// This is necessary because the mock router does not use wrap_version, but
		// this is not necessary in prod
		assert_ok!(<RelayChainPalletXcm as WrapVersion>::wrap_version(
			&Parachain(1).into(),
			mock_message
		));

		// Transfer assets. Since it is an unknown destination, it will query for version
		assert_ok!(RelayChainPalletXcm::limited_reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::from(dest).clone()),
			Box::new(([], 123).into()),
			0,
			WeightLimit::Unlimited
		));

		// Let's advance the relay. This should trigger the subscription message
		relay_chain::relay_roll_to(2);

		// queries should have been updated
		assert!(RelayChainPalletXcm::query(0).is_some());
	});

	let expected_supported_version: relay_chain::RuntimeEvent =
		pallet_xcm::Event::SupportedVersionChanged {
			location: Location {
				parents: 0,
				interior: [Parachain(1)].into(),
			},
			version: 1,
		}
		.into();

	Relay::execute_with(|| {
		// Assert that the events vector contains the version change
		assert!(relay_chain::relay_events().contains(&expected_supported_version));
	});

	// ParaA changes version to 2, and calls on_runtime_upgrade. This should notify the targets
	// of the new version change
	ParaA::execute_with(|| {
		// Set version
		parachain::XcmVersioner::set_version(2);
		// Do runtime upgrade
		parachain::on_runtime_upgrade();
		// Initialize block, to call on_initialize and notify targets
		parachain::para_roll_to(2);
		// Expect the event in the parachain
		assert!(parachain::para_events().iter().any(|e| matches!(
			e,
			parachain::RuntimeEvent::PolkadotXcm(pallet_xcm::Event::VersionChangeNotified {
				result: 2,
				..
			})
		)));
	});

	// This event should have been seen in the relay
	let expected_supported_version_2: relay_chain::RuntimeEvent =
		pallet_xcm::Event::SupportedVersionChanged {
			location: Location {
				parents: 0,
				interior: [Parachain(1)].into(),
			},
			version: 2,
		}
		.into();

	Relay::execute_with(|| {
		// Assert that the events vector contains the new version change
		assert!(relay_chain::relay_events().contains(&expected_supported_version_2));
	});
}

#[test]
fn test_automatic_versioning_on_runtime_upgrade_with_para_b() {
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
	let response = Response::Version(2);
	let querier: Location = [].into();

	// This is irrelevant, nothing will be done with this message,
	// but we need to pass a message as an argument to trigger the storage change
	let mock_message: Xcm<()> = Xcm(vec![QueryResponse {
		query_id: 0,
		response,
		max_weight: Weight::zero(),
		querier: Some(querier),
	}]);

	ParaA::execute_with(|| {
		// advertised version
		parachain::XcmVersioner::set_version(2);
	});

	ParaB::execute_with(|| {
		// Let's try with v0
		parachain::XcmVersioner::set_version(0);

		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata,
			1u128,
			true
		));
		assert_ok!(add_supported_asset(source_location, 0));
	});

	ParaA::execute_with(|| {
		// This sets the default version, for not known destinations
		assert_ok!(ParachainPalletXcm::force_default_xcm_version(
			parachain::RuntimeOrigin::root(),
			Some(3)
		));
		// Wrap version, which sets VersionedStorage
		assert_ok!(<ParachainPalletXcm as WrapVersion>::wrap_version(
			&Location::new(1, [Parachain(2)]).into(),
			mock_message
		));

		parachain::para_roll_to(2);

		// queries should have been updated
		assert!(ParachainPalletXcm::query(0).is_some());
	});

	let expected_supported_version: parachain::RuntimeEvent =
		pallet_xcm::Event::SupportedVersionChanged {
			location: Location {
				parents: 1,
				interior: [Parachain(2)].into(),
			},
			version: 0,
		}
		.into();

	ParaA::execute_with(|| {
		// Assert that the events vector contains the version change
		assert!(parachain::para_events().contains(&expected_supported_version));
	});

	// Let's ensure talking in v0 works
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
	}
	.into();
	let (chain_part, beneficiary) = split_location_into_chain_part_and_beneficiary(dest).unwrap();
	ParaA::execute_with(|| {
		let asset = currency_to_asset(parachain::CurrencyId::SelfReserve, 100);
		// free execution, full amount received
		assert_ok!(PolkadotXcm::transfer_assets(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(VersionedLocation::from(chain_part)),
			Box::new(VersionedLocation::from(beneficiary)),
			Box::new(VersionedAssets::from(vec![asset])),
			0,
			WeightLimit::Limited(Weight::from_parts(80u64, DEFAULT_PROOF_SIZE))
		));
		// free execution, full amount received
		assert_eq!(
			ParaBalances::free_balance(&PARAALICE.into()),
			INITIAL_BALANCE - 100
		);
	});

	ParaB::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 100);
	});

	// ParaB changes version to 2, and calls on_runtime_upgrade. This should notify the targets
	// of the new version change
	ParaB::execute_with(|| {
		// Set version
		parachain::XcmVersioner::set_version(2);
		// Do runtime upgrade
		parachain::on_runtime_upgrade();
		// Initialize block, to call on_initialize and notify targets
		parachain::para_roll_to(2);
		// Expect the event in the parachain
		assert!(parachain::para_events().iter().any(|e| matches!(
			e,
			parachain::RuntimeEvent::PolkadotXcm(pallet_xcm::Event::VersionChangeNotified {
				result: 2,
				..
			})
		)));
	});

	// This event should have been seen in para A
	let expected_supported_version_2: parachain::RuntimeEvent =
		pallet_xcm::Event::SupportedVersionChanged {
			location: Location {
				parents: 1,
				interior: [Parachain(2)].into(),
			},
			version: 2,
		}
		.into();

	// Para A should have received the version change
	ParaA::execute_with(|| {
		// Assert that the events vector contains the new version change
		assert!(parachain::para_events().contains(&expected_supported_version_2));
	});
}

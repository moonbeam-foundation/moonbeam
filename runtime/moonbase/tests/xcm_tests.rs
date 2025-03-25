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

//! Moonbase Runtime Xcm Tests

mod xcm_mock;
use frame_support::{
	assert_ok,
	traits::{ConstU32, PalletInfo, PalletInfoAccess},
	weights::constants::WEIGHT_REF_TIME_PER_SECOND,
	weights::Weight,
	BoundedVec,
};
use pallet_xcm_transactor::{
	Currency, CurrencyPayment, HrmpInitParams, HrmpOperation, TransactWeights,
};
use sp_runtime::traits::{Convert, MaybeEquivalence};
use sp_std::boxed::Box;
use xcm::{
	latest::prelude::{
		AccountId32, AccountKey20, All, Asset, AssetId, Assets as XcmAssets, BuyExecution,
		ClearOrigin, DepositAsset, Fungibility, GeneralIndex, Junction, Junctions, Limited,
		Location, OriginKind, PalletInstance, Parachain, QueryResponse, Reanchorable, Response,
		WeightLimit, WithdrawAsset, Xcm,
	},
	VersionedAssets,
};
use xcm::{IntoVersion, VersionedLocation, WrapVersion};
use xcm_executor::traits::ConvertLocation;
use xcm_mock::*;
use xcm_primitives::{
	split_location_into_chain_part_and_beneficiary, UtilityEncodeCall, DEFAULT_PROOF_SIZE,
};
use xcm_simulator::TestExt;
mod common;
use cumulus_primitives_core::relay_chain::HrmpChannelId;
use parachain::PolkadotXcm;

fn add_supported_asset(asset_type: parachain::AssetType, units_per_second: u128) -> Result<(), ()> {
	let parachain::AssetType::Xcm(location_v3) = asset_type;
	let VersionedLocation::V5(location_v5) = VersionedLocation::V3(location_v3)
		.into_version(5)
		.map_err(|_| ())?
	else {
		return Err(());
	};
	use frame_support::weights::WeightToFee as _;
	let native_amount_per_second: u128 =
		<parachain::Runtime as pallet_xcm_weight_trader::Config>::WeightToFee::weight_to_fee(
			&Weight::from_parts(
				frame_support::weights::constants::WEIGHT_REF_TIME_PER_SECOND,
				0,
			),
		)
		.try_into()
		.map_err(|_| ())?;
	let precision_factor = 10u128.pow(pallet_xcm_weight_trader::RELATIVE_PRICE_DECIMALS);
	let relative_price: u128 = if units_per_second > 0u128 {
		native_amount_per_second
			.saturating_mul(precision_factor)
			.saturating_div(units_per_second)
	} else {
		0u128
	};
	pallet_xcm_weight_trader::SupportedAssets::<parachain::Runtime>::insert(
		location_v5,
		(true, relative_price),
	);
	Ok(())
}

fn currency_to_asset(currency_id: parachain::CurrencyId, amount: u128) -> Asset {
	Asset {
		id: AssetId(
			<parachain::Runtime as pallet_xcm_transactor::Config>::CurrencyIdToLocation::convert(
				currency_id,
			)
			.unwrap(),
		),
		fun: Fungibility::Fungible(amount),
	}
}

// Send a relay asset (like DOT) to a parachain A
#[test]
fn receive_relay_asset_from_relay() {
	MockNet::reset();

	let source_location = parachain::AssetType::Xcm(xcm::v3::Location::parent());
	let source_id: parachain::AssetId = source_location.clone().into();
	let asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};
	// register relay asset in parachain A
	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata,
			1u128,
			true
		));
		assert_ok!(add_supported_asset(source_location.clone(), 0));
	});

	// Actually send relay asset to parachain
	let dest: Location = AccountKey20 {
		network: None,
		key: PARAALICE,
	}
	.into();
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::limited_reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::from(dest).clone().into()),
			Box::new(([], 123).into()),
			0,
			WeightLimit::Unlimited
		));
	});

	// Verify that parachain received the asset
	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 123);
	});
}

// Send relay asset (like DOT) back from Parachain A to relaychain
#[test]
fn send_relay_asset_to_relay() {
	MockNet::reset();

	let source_location = parachain::AssetType::Xcm(xcm::v3::Location::parent());
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};

	// Register relay asset in paraA
	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata,
			1u128,
			true
		));
		// Free execution
		assert_ok!(add_supported_asset(source_location, 0));
	});

	let dest: Location = Junction::AccountKey20 {
		network: None,
		key: PARAALICE,
	}
	.into();

	// First send relay chain asset to Parachain like in previous test
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::limited_reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::from(dest.clone()).into()),
			Box::new(([], 123).into()),
			0,
			WeightLimit::Unlimited
		));
	});

	ParaA::execute_with(|| {
		// Free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 123);
	});

	// Lets gather the balance before sending back money
	let mut balance_before_sending = 0;
	Relay::execute_with(|| {
		balance_before_sending = RelayBalances::free_balance(&RELAYALICE);
	});

	// We now send back some money to the relay
	let dest = Location {
		parents: 1,
		interior: [AccountId32 {
			network: None,
			id: RELAYALICE.into(),
		}]
		.into(),
	};
	let (chain_part, beneficiary) = split_location_into_chain_part_and_beneficiary(dest).unwrap();

	ParaA::execute_with(|| {
		let asset = currency_to_asset(parachain::CurrencyId::ForeignAsset(source_id), 123);
		// free execution, full amount received
		assert_ok!(PolkadotXcm::transfer_assets(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(VersionedLocation::from(chain_part)),
			Box::new(VersionedLocation::from(beneficiary)),
			Box::new(VersionedAssets::from(asset.into())),
			0,
			WeightLimit::Limited(Weight::from_parts(40000u64, DEFAULT_PROOF_SIZE))
		));
	});

	// The balances in paraAlice should have been substracted
	ParaA::execute_with(|| {
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 0);
	});

	// Balances in the relay should have been received
	Relay::execute_with(|| {
		// Free execution,x	 full amount received
		assert!(RelayBalances::free_balance(&RELAYALICE) > balance_before_sending);
	});
}

#[test]
fn send_relay_asset_to_para_b() {
	MockNet::reset();

	let source_location = parachain::AssetType::Xcm(xcm::v3::Location::parent());
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};

	// Register asset in paraA. Free execution
	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata.clone(),
			1u128,
			true
		));
		assert_ok!(add_supported_asset(source_location.clone(), 0));
	});

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

	// First send relay chain asset to Parachain A like in previous test
	let dest: Location = Junction::AccountKey20 {
		network: None,
		key: PARAALICE,
	}
	.into();
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::limited_reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::from(dest).clone().into()),
			Box::new(([], 123).into()),
			0,
			WeightLimit::Unlimited
		));
	});

	ParaA::execute_with(|| {
		// Free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 123);
	});

	// Now send relay asset from para A to para B
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
		let asset = currency_to_asset(parachain::CurrencyId::ForeignAsset(source_id), 100);
		assert_ok!(PolkadotXcm::transfer_assets(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(VersionedLocation::from(chain_part)),
			Box::new(VersionedLocation::from(beneficiary)),
			Box::new(VersionedAssets::from(vec![asset].into())),
			0,
			WeightLimit::Limited(Weight::from_parts(40000u64, DEFAULT_PROOF_SIZE))
		));
	});

	// Para A balances should have been substracted
	ParaA::execute_with(|| {
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 23);
	});

	// Para B balances should have been credited
	ParaB::execute_with(|| {
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 100);
	});
}

#[test]
fn send_para_a_asset_to_para_b() {
	MockNet::reset();

	// this represents the asset in paraA
	let para_a_balances = Location::new(1, [Parachain(1), PalletInstance(1u8)]);
	let source_location = para_a_balances.try_into().expect("convert to v3");
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

	// Send para A asset from para A to para B
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
		// Free execution, full amount received
		assert_ok!(PolkadotXcm::transfer_assets(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(VersionedLocation::from(chain_part)),
			Box::new(VersionedLocation::from(beneficiary)),
			Box::new(VersionedAssets::from(vec![asset].into())),
			0,
			WeightLimit::Limited(Weight::from_parts(800000u64, DEFAULT_PROOF_SIZE))
		));
	});

	// Native token is substracted in paraA
	ParaA::execute_with(|| {
		assert_eq!(
			ParaBalances::free_balance(&PARAALICE.into()),
			INITIAL_BALANCE - 100
		);
	});

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
	let source_location = para_a_balances.try_into().expect("convert to v3");
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
		// free execution, full amount received
		assert_ok!(PolkadotXcm::transfer_assets(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(VersionedLocation::from(chain_part)),
			Box::new(VersionedLocation::from(beneficiary)),
			Box::new(VersionedAssets::from(vec![asset].into())),
			0,
			WeightLimit::Limited(Weight::from_parts(80u64, DEFAULT_PROOF_SIZE))
		));
	});

	// Para A balances have been substracted
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
			Box::new(VersionedAssets::from(vec![asset].into())),
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
	let source_location = para_a_balances.try_into().expect("convert to v3");
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
			Box::new(VersionedAssets::from(vec![asset].into())),
			0,
			WeightLimit::Limited(Weight::from_parts(80u64, DEFAULT_PROOF_SIZE))
		));
	});

	// Balances have been substracted
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
			Box::new(VersionedAssets::from(vec![asset].into())),
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
	let source_location = para_a_balances.try_into().expect("convert to v3");
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
			Box::new(VersionedAssets::from(vec![asset].into())),
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
fn receive_relay_asset_with_trader() {
	MockNet::reset();

	let source_location = parachain::AssetType::Xcm(xcm::v3::Location::parent());
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};

	// This time we are gonna put a rather high number of units per second
	// we know later we will divide by 1e12
	// Lets put 1e6 as units per second
	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata,
			1u128,
			true
		));
		assert_ok!(add_supported_asset(source_location, 2_500_000_000_000));
	});

	let dest: Location = Junction::AccountKey20 {
		network: None,
		key: PARAALICE,
	}
	.into();
	// We are sending 100 tokens from relay.
	// Amount spent in fees is Units per second * weight / 1_000_000_000_000 (weight per second)
	// weight is 4 since we are executing 4 instructions with a unitweightcost of 1.
	// Units per second should be 2_500_000_000_000_000
	// Therefore with no refund, we should receive 10 tokens less
	// Native trader fails for this, and we use the asset trader
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::limited_reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::from(dest).clone().into()),
			Box::new(([], 100).into()),
			0,
			WeightLimit::Unlimited
		));
	});

	ParaA::execute_with(|| {
		// non-free execution, not full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 90);
		// Fee should have been received by treasury
		assert_eq!(Assets::balance(source_id, &Treasury::account_id()), 10);
	});
}

#[test]
fn send_para_a_asset_to_para_b_with_trader() {
	MockNet::reset();

	let para_a_balances = Location::new(1, [Parachain(1), PalletInstance(1u8)]);
	let source_location = para_a_balances.try_into().expect("convert to v3");
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
			Box::new(VersionedAssets::from(vec![asset].into())),
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
	let source_location = para_a_balances.try_into().expect("convert to v3");
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
			Box::new(VersionedAssets::from(vec![asset_fee, asset].into())),
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

#[test]
fn error_when_not_paying_enough() {
	MockNet::reset();

	let source_location = parachain::AssetType::Xcm(xcm::v3::Location::parent());
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};

	let dest: Location = Junction::AccountKey20 {
		network: None,
		key: PARAALICE,
	}
	.into();
	// This time we are gonna put a rather high number of units per second
	// we know later we will divide by 1e12
	// Lets put 1e6 as units per second
	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata,
			1u128,
			true
		));
		assert_ok!(add_supported_asset(source_location, 2500000000000));
	});

	ParaA::execute_with(|| {
		// amount not received as it is not paying enough
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 0);
	});

	// We are sending 100 tokens from relay.
	// If we set the dest weight to be 1e7, we know the buy_execution will spend 1e7*1e6/1e12 = 10
	// Therefore with no refund, we should receive 10 tokens less
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::limited_reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::from(dest).clone().into()),
			Box::new(([], 5).into()),
			0,
			WeightLimit::Unlimited
		));
	});

	ParaA::execute_with(|| {
		// amount not received as it is not paying enough
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 0);
	});
}

#[test]
fn transact_through_derivative_multilocation() {
	MockNet::reset();

	let source_location = parachain::AssetType::Xcm(xcm::v3::Location::parent());
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};

	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata,
			1u128,
			true
		));
		assert_ok!(add_supported_asset(source_location, 1));

		// Root can set transact info
		assert_ok!(XcmTransactor::set_transact_info(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::from(Location::parent())),
			// Relay charges 1000 for every instruction, and we have 3, so 3000
			3000.into(),
			20000000000.into(),
			None
		));
		// Root can set transact info
		assert_ok!(XcmTransactor::set_fee_per_second(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::from(Location::parent())),
			WEIGHT_REF_TIME_PER_SECOND as u128,
		));
	});

	// Let's construct the call to know how much weight it is going to require

	let dest: Location = AccountKey20 {
		network: None,
		key: PARAALICE,
	}
	.into();
	Relay::execute_with(|| {
		// 4000000000 transact + 3000 correspond to 4000003000 tokens. 100 more for the transfer call
		assert_ok!(RelayChainPalletXcm::limited_reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::from(dest).clone().into()),
			Box::new(([], 4000003100u128).into()),
			0,
			WeightLimit::Unlimited
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 4000003100);
	});

	// Register address
	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::register(
			parachain::RuntimeOrigin::root(),
			PARAALICE.into(),
			0,
		));
	});

	// Send to registered address

	let registered_address = derivative_account_id(para_a_account(), 0);
	let dest = Location {
		parents: 1,
		interior: [AccountId32 {
			network: None,
			id: registered_address.clone().into(),
		}]
		.into(),
	};
	let (chain_part, beneficiary) = split_location_into_chain_part_and_beneficiary(dest).unwrap();

	ParaA::execute_with(|| {
		let asset = currency_to_asset(parachain::CurrencyId::ForeignAsset(source_id), 100);
		// free execution, full amount received
		assert_ok!(PolkadotXcm::transfer_assets(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(VersionedLocation::from(chain_part)),
			Box::new(VersionedLocation::from(beneficiary)),
			Box::new(VersionedAssets::from(vec![asset].into())),
			0,
			WeightLimit::Limited(Weight::from_parts(40000u64, DEFAULT_PROOF_SIZE))
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 4000003000);
	});

	// What we will do now is transfer this relay tokens from the derived account to the sovereign
	// again
	Relay::execute_with(|| {
		// free execution,x	 full amount received
		assert!(RelayBalances::free_balance(&para_a_account()) == 4000003000);
	});

	// Encode the call. Balances transact to para_a_account
	// First index
	let mut encoded: Vec<u8> = Vec::new();
	let index = <relay_chain::Runtime as frame_system::Config>::PalletInfo::index::<
		relay_chain::Balances,
	>()
	.unwrap() as u8;

	encoded.push(index);

	// Then call bytes
	let mut call_bytes = pallet_balances::Call::<relay_chain::Runtime>::transfer_allow_death {
		dest: para_a_account(),
		value: 100u32.into(),
	}
	.encode();
	encoded.append(&mut call_bytes);

	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_derivative(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			parachain::MockTransactors::Relay,
			0,
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					Location::parent()
				))),
				fee_amount: None
			},
			// 4000000000 + 3000 we should have taken out 4000003000 tokens from the caller
			encoded,
			TransactWeights {
				transact_required_weight_at_most: 4000000000.into(),
				overall_weight: None
			},
			false
		));
		let event_found: Option<parachain::RuntimeEvent> = parachain::para_events()
			.iter()
			.find_map(|event| match event.clone() {
				parachain::RuntimeEvent::PolkadotXcm(pallet_xcm::Event::AssetsTrapped {
					..
				}) => Some(event.clone()),
				_ => None,
			});
		// Assert that the events do not contain the assets being trapped
		assert!(event_found.is_none());
	});

	Relay::execute_with(|| {
		// free execution,x	 full amount received
		assert!(RelayBalances::free_balance(&para_a_account()) == 100);

		assert!(RelayBalances::free_balance(&registered_address) == 0);
	});
}

#[test]
fn transact_through_derivative_with_custom_fee_weight() {
	MockNet::reset();

	let source_location = parachain::AssetType::Xcm(xcm::v3::Location::parent());
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};

	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata,
			1u128,
			true
		));
		assert_ok!(add_supported_asset(source_location, 1));
	});

	// Let's construct the call to know how much weight it is going to require

	let dest: Location = AccountKey20 {
		network: None,
		key: PARAALICE,
	}
	.into();
	Relay::execute_with(|| {
		// 4000000000 transact + 3000 correspond to 4000003000 tokens. 100 more for the transfer call
		assert_ok!(RelayChainPalletXcm::limited_reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::from(dest).clone().into()),
			Box::new(([], 4000003100u128).into()),
			0,
			WeightLimit::Unlimited
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 4000003100);
	});

	// Register address
	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::register(
			parachain::RuntimeOrigin::root(),
			PARAALICE.into(),
			0,
		));
	});

	// Send to registered address

	let registered_address = derivative_account_id(para_a_account(), 0);
	let dest = Location {
		parents: 1,
		interior: [AccountId32 {
			network: None,
			id: registered_address.clone().into(),
		}]
		.into(),
	};
	let (chain_part, beneficiary) = split_location_into_chain_part_and_beneficiary(dest).unwrap();

	ParaA::execute_with(|| {
		let asset = currency_to_asset(parachain::CurrencyId::ForeignAsset(source_id), 100);
		// free execution, full amount received
		assert_ok!(PolkadotXcm::transfer_assets(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(VersionedLocation::from(chain_part)),
			Box::new(VersionedLocation::from(beneficiary)),
			Box::new(VersionedAssets::from(vec![asset].into())),
			0,
			WeightLimit::Limited(Weight::from_parts(40000u64, DEFAULT_PROOF_SIZE))
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 4000003000);
	});

	// What we will do now is transfer this relay tokens from the derived account to the sovereign
	// again
	Relay::execute_with(|| {
		// free execution,x	 full amount received
		assert!(RelayBalances::free_balance(&para_a_account()) == 4000003000);
	});

	// Encode the call. Balances transact to para_a_account
	// First index
	let mut encoded: Vec<u8> = Vec::new();
	let index = <relay_chain::Runtime as frame_system::Config>::PalletInfo::index::<
		relay_chain::Balances,
	>()
	.unwrap() as u8;

	encoded.push(index);

	// Then call bytes
	let mut call_bytes = pallet_balances::Call::<relay_chain::Runtime>::transfer_allow_death {
		dest: para_a_account(),
		value: 100u32.into(),
	}
	.encode();
	encoded.append(&mut call_bytes);

	let overall_weight = 4000003000u64;
	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_derivative(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			parachain::MockTransactors::Relay,
			0,
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					Location::parent()
				))),
				// 1-1 fee weight mapping
				fee_amount: Some(overall_weight as u128)
			},
			// 4000000000 + 3000 we should have taken out 4000003000 tokens from the caller
			encoded,
			TransactWeights {
				transact_required_weight_at_most: 4000000000.into(),
				overall_weight: Some(Limited(overall_weight.into()))
			},
			false
		));
		let event_found: Option<parachain::RuntimeEvent> = parachain::para_events()
			.iter()
			.find_map(|event| match event.clone() {
				parachain::RuntimeEvent::PolkadotXcm(pallet_xcm::Event::AssetsTrapped {
					..
				}) => Some(event.clone()),
				_ => None,
			});
		// Assert that the events do not contain the assets being trapped
		assert!(event_found.is_none());
	});

	Relay::execute_with(|| {
		// free execution,x	 full amount received
		assert!(RelayBalances::free_balance(&para_a_account()) == 100);

		assert!(RelayBalances::free_balance(&registered_address) == 0);
	});
}

#[test]
fn transact_through_derivative_with_custom_fee_weight_refund() {
	MockNet::reset();

	let source_location = parachain::AssetType::Xcm(xcm::v3::Location::parent());
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};

	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata,
			1u128,
			true
		));
		assert_ok!(add_supported_asset(source_location, 1));
	});

	// Let's construct the call to know how much weight it is going to require

	let dest: Location = AccountKey20 {
		network: None,
		key: PARAALICE,
	}
	.into();
	Relay::execute_with(|| {
		// 4000000000 transact + 9000 correspond to 4000009000 tokens. 100 more for the transfer call
		assert_ok!(RelayChainPalletXcm::limited_reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::from(dest).clone().into()),
			Box::new(([], 4000009100u128).into()),
			0,
			WeightLimit::Unlimited
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 4000009100);
	});

	// Register address
	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::register(
			parachain::RuntimeOrigin::root(),
			PARAALICE.into(),
			0,
		));
	});

	// Send to registered address

	let registered_address = derivative_account_id(para_a_account(), 0);
	let dest = Location {
		parents: 1,
		interior: [AccountId32 {
			network: None,
			id: registered_address.clone().into(),
		}]
		.into(),
	};
	let (chain_part, beneficiary) = split_location_into_chain_part_and_beneficiary(dest).unwrap();

	ParaA::execute_with(|| {
		let asset = currency_to_asset(parachain::CurrencyId::ForeignAsset(source_id), 100);
		// free execution, full amount received
		assert_ok!(PolkadotXcm::transfer_assets(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(VersionedLocation::from(chain_part)),
			Box::new(VersionedLocation::from(beneficiary)),
			Box::new(VersionedAssets::from(vec![asset].into())),
			0,
			WeightLimit::Limited(Weight::from_parts(40000u64, DEFAULT_PROOF_SIZE))
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 4000009000);
	});

	// What we will do now is transfer this relay tokens from the derived account to the sovereign
	// again
	Relay::execute_with(|| {
		// free execution,x	 full amount received
		assert!(RelayBalances::free_balance(&para_a_account()) == 4000009000);
	});

	// Encode the call. Balances transact to para_a_account
	// First index
	let mut encoded: Vec<u8> = Vec::new();
	let index = <relay_chain::Runtime as frame_system::Config>::PalletInfo::index::<
		relay_chain::Balances,
	>()
	.unwrap() as u8;

	encoded.push(index);

	// Then call bytes
	let mut call_bytes = pallet_balances::Call::<relay_chain::Runtime>::transfer_allow_death {
		dest: para_a_account(),
		value: 100u32.into(),
	}
	.encode();
	encoded.append(&mut call_bytes);

	let overall_weight = 4000009000u64;
	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_derivative(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			parachain::MockTransactors::Relay,
			0,
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					Location::parent()
				))),
				// 1-1 fee weight mapping
				fee_amount: Some(overall_weight as u128)
			},
			encoded,
			TransactWeights {
				transact_required_weight_at_most: 4000000000.into(),
				overall_weight: Some(Limited(overall_weight.into()))
			},
			true
		));
		let event_found: Option<parachain::RuntimeEvent> = parachain::para_events()
			.iter()
			.find_map(|event| match event.clone() {
				parachain::RuntimeEvent::PolkadotXcm(pallet_xcm::Event::AssetsTrapped {
					..
				}) => Some(event.clone()),
				_ => None,
			});
		// Assert that the events do not contain the assets being trapped
		assert!(event_found.is_none());
	});

	Relay::execute_with(|| {
		// free execution,x	 full amount received
		// 4000005186 refunded + 100 transferred = 4000005286
		assert_eq!(RelayBalances::free_balance(&para_a_account()), 4000005286);
		assert_eq!(RelayBalances::free_balance(&registered_address), 0);
	});
}

#[test]
fn transact_through_sovereign() {
	MockNet::reset();

	let source_location = parachain::AssetType::Xcm(xcm::v3::Location::parent());
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};

	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata,
			1u128,
			true
		));
		assert_ok!(add_supported_asset(source_location, 1));

		// Root can set transact info
		assert_ok!(XcmTransactor::set_transact_info(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::from(Location::parent())),
			// Relay charges 1000 for every instruction, and we have 3, so 3000
			3000.into(),
			20000000000.into(),
			None
		));
		// Root can set transact info
		assert_ok!(XcmTransactor::set_fee_per_second(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::from(Location::parent())),
			WEIGHT_REF_TIME_PER_SECOND as u128,
		));
	});

	let dest: Location = AccountKey20 {
		network: None,
		key: PARAALICE,
	}
	.into();
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::limited_reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::from(dest).clone().into()),
			Box::new(([], 4000003100u128).into()),
			0,
			WeightLimit::Unlimited
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 4000003100);
	});

	// Register address
	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::register(
			parachain::RuntimeOrigin::root(),
			PARAALICE.into(),
			0,
		));
	});

	// Send to registered address
	let registered_address = derivative_account_id(para_a_account(), 0);
	let dest = Location {
		parents: 1,
		interior: [AccountId32 {
			network: None,
			id: registered_address.clone().into(),
		}]
		.into(),
	};
	let (chain_part, beneficiary) = split_location_into_chain_part_and_beneficiary(dest).unwrap();

	ParaA::execute_with(|| {
		let asset = currency_to_asset(parachain::CurrencyId::ForeignAsset(source_id), 100);
		// free execution, full amount received
		assert_ok!(PolkadotXcm::transfer_assets(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(VersionedLocation::from(chain_part)),
			Box::new(VersionedLocation::from(beneficiary)),
			Box::new(VersionedAssets::from(vec![asset].into())),
			0,
			WeightLimit::Limited(Weight::from_parts(40000u64, DEFAULT_PROOF_SIZE))
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 4000003000);
	});

	// What we will do now is transfer this relay tokens from the derived account to the sovereign
	// again
	Relay::execute_with(|| {
		// free execution,x	 full amount received
		assert!(RelayBalances::free_balance(&para_a_account()) == 4000003000);
		0
	});

	// We send the xcm transact operation to parent
	let dest = Location {
		parents: 1,
		interior: [].into(),
	};

	// Encode the call. Balances transact to para_a_account
	// First index
	let mut encoded: Vec<u8> = Vec::new();
	let index = <relay_chain::Runtime as frame_system::Config>::PalletInfo::index::<
		relay_chain::Balances,
	>()
	.unwrap() as u8;

	encoded.push(index);

	// Then call bytes
	let mut call_bytes = pallet_balances::Call::<relay_chain::Runtime>::transfer_allow_death {
		dest: para_a_account(),
		value: 100u32.into(),
	}
	.encode();
	encoded.append(&mut call_bytes);

	let utility_bytes = parachain::MockTransactors::Relay.encode_call(
		xcm_primitives::UtilityAvailableCalls::AsDerivative(0, encoded),
	);

	// Root can directly pass the execution byes to the sovereign
	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_sovereign(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::from(dest)),
			Some(PARAALICE.into()),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					Location::parent()
				))),
				fee_amount: None
			},
			utility_bytes,
			OriginKind::SovereignAccount,
			TransactWeights {
				transact_required_weight_at_most: 4000000000.into(),
				overall_weight: None
			},
			false
		));
	});

	Relay::execute_with(|| {
		// free execution,x	 full amount received
		assert!(RelayBalances::free_balance(&para_a_account()) == 100);

		assert!(RelayBalances::free_balance(&registered_address) == 0);
	});
}

#[test]
fn transact_through_sovereign_fee_payer_none() {
	MockNet::reset();

	ParaA::execute_with(|| {
		// Root can set transact info
		assert_ok!(XcmTransactor::set_transact_info(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::from(Location::parent())),
			// Relay charges 1000 for every instruction, and we have 3, so 3000
			3000.into(),
			20000000000.into(),
			None
		));
		// Root can set transact info
		assert_ok!(XcmTransactor::set_fee_per_second(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::from(Location::parent())),
			WEIGHT_REF_TIME_PER_SECOND as u128,
		));
	});

	let derivative_address = derivative_account_id(para_a_account(), 0);

	Relay::execute_with(|| {
		// Transfer 100 tokens to derivative_address on the relay
		assert_ok!(RelayBalances::transfer_keep_alive(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			derivative_address.clone(),
			100u128
		));

		// Transfer the XCM execution fee amount to ParaA's sovereign account
		assert_ok!(RelayBalances::transfer_keep_alive(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			para_a_account(),
			4000003000u128
		));
	});

	// Check balances before the transact call
	Relay::execute_with(|| {
		assert_eq!(RelayBalances::free_balance(&para_a_account()), 4000003000);
		assert_eq!(RelayBalances::free_balance(&derivative_address), 100);
		assert_eq!(RelayBalances::free_balance(&RELAYBOB), 0);
	});

	// Encode the call. Balances transfer of 100 relay tokens to RELAYBOB
	let mut encoded: Vec<u8> = Vec::new();
	let index = <relay_chain::Runtime as frame_system::Config>::PalletInfo::index::<
		relay_chain::Balances,
	>()
	.unwrap() as u8;

	encoded.push(index);

	let mut call_bytes = pallet_balances::Call::<relay_chain::Runtime>::transfer_allow_death {
		dest: RELAYBOB,
		value: 100u32.into(),
	}
	.encode();
	encoded.append(&mut call_bytes);

	// The final call will be an AsDerivative using index 0
	let utility_bytes = parachain::MockTransactors::Relay.encode_call(
		xcm_primitives::UtilityAvailableCalls::AsDerivative(0, encoded),
	);

	// We send the xcm transact operation to parent
	let dest = Location {
		parents: 1,
		interior: /* Here */ [].into(),
	};

	// Root can directly pass the execution byes to the sovereign
	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_sovereign(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::from(dest)),
			// No fee_payer here. The sovereign account will pay the fees on destination.
			None,
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					Location::parent()
				))),
				fee_amount: None
			},
			utility_bytes,
			OriginKind::SovereignAccount,
			TransactWeights {
				transact_required_weight_at_most: 4000000000.into(),
				overall_weight: None
			},
			false
		));
	});

	// Check balances after the transact call are correct
	Relay::execute_with(|| {
		assert_eq!(RelayBalances::free_balance(&para_a_account()), 0);
		assert_eq!(RelayBalances::free_balance(&derivative_address), 0);
		assert_eq!(RelayBalances::free_balance(&RELAYBOB), 100);
	});
}

#[test]
fn transact_through_sovereign_with_custom_fee_weight() {
	MockNet::reset();

	let source_location = parachain::AssetType::Xcm(xcm::v3::Location::parent());
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};

	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata,
			1u128,
			true
		));
		assert_ok!(add_supported_asset(source_location, 1));
	});

	let dest: Location = AccountKey20 {
		network: None,
		key: PARAALICE,
	}
	.into();
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::limited_reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::from(dest).clone().into()),
			Box::new(([], 4000003100u128).into()),
			0,
			WeightLimit::Unlimited
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 4000003100);
	});

	// Register address
	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::register(
			parachain::RuntimeOrigin::root(),
			PARAALICE.into(),
			0,
		));
	});

	// Send to registered address
	let registered_address = derivative_account_id(para_a_account(), 0);
	let dest = Location {
		parents: 1,
		interior: [AccountId32 {
			network: None,
			id: registered_address.clone().into(),
		}]
		.into(),
	};
	let (chain_part, beneficiary) = split_location_into_chain_part_and_beneficiary(dest).unwrap();

	ParaA::execute_with(|| {
		let asset = currency_to_asset(parachain::CurrencyId::ForeignAsset(source_id), 100);
		// free execution, full amount received
		assert_ok!(PolkadotXcm::transfer_assets(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(VersionedLocation::from(chain_part)),
			Box::new(VersionedLocation::from(beneficiary)),
			Box::new(VersionedAssets::from(vec![asset].into())),
			0,
			WeightLimit::Limited(Weight::from_parts(40000u64, DEFAULT_PROOF_SIZE))
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 4000003000);
	});

	// What we will do now is transfer this relay tokens from the derived account to the sovereign
	// again
	Relay::execute_with(|| {
		// free execution,x	 full amount received
		assert!(RelayBalances::free_balance(&para_a_account()) == 4000003000);
		0
	});

	// We send the xcm transact operation to parent
	let dest = Location {
		parents: 1,
		interior: [].into(),
	};

	// Encode the call. Balances transact to para_a_account
	// First index
	let mut encoded: Vec<u8> = Vec::new();
	let index = <relay_chain::Runtime as frame_system::Config>::PalletInfo::index::<
		relay_chain::Balances,
	>()
	.unwrap() as u8;

	encoded.push(index);

	// Then call bytes
	let mut call_bytes = pallet_balances::Call::<relay_chain::Runtime>::transfer_allow_death {
		dest: para_a_account(),
		value: 100u32.into(),
	}
	.encode();
	encoded.append(&mut call_bytes);

	let utility_bytes = parachain::MockTransactors::Relay.encode_call(
		xcm_primitives::UtilityAvailableCalls::AsDerivative(0, encoded),
	);

	let total_weight = 4000003000u64;
	// Root can directly pass the execution byes to the sovereign
	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_sovereign(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::from(dest)),
			Some(PARAALICE.into()),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					Location::parent()
				))),
				// 1-1 fee-weight mapping
				fee_amount: Some(total_weight as u128)
			},
			utility_bytes,
			OriginKind::SovereignAccount,
			TransactWeights {
				transact_required_weight_at_most: 4000000000.into(),
				overall_weight: Some(Limited(total_weight.into()))
			},
			false
		));
	});

	Relay::execute_with(|| {
		// free execution,x	 full amount received
		assert!(RelayBalances::free_balance(&para_a_account()) == 100);

		assert!(RelayBalances::free_balance(&registered_address) == 0);
	});
}

#[test]
fn transact_through_sovereign_with_custom_fee_weight_refund() {
	MockNet::reset();

	let source_location = parachain::AssetType::Xcm(xcm::v3::Location::parent());
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};

	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata,
			1u128,
			true
		));
		assert_ok!(add_supported_asset(source_location, 1));
	});

	let dest: Location = AccountKey20 {
		network: None,
		key: PARAALICE,
	}
	.into();
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::limited_reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::from(dest).clone().into()),
			Box::new(([], 4000009100u128).into()),
			0,
			WeightLimit::Unlimited
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 4000009100);
	});

	// Register address
	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::register(
			parachain::RuntimeOrigin::root(),
			PARAALICE.into(),
			0,
		));
	});

	// Send to registered address
	let registered_address = derivative_account_id(para_a_account(), 0);
	let dest = Location {
		parents: 1,
		interior: [AccountId32 {
			network: None,
			id: registered_address.clone().into(),
		}]
		.into(),
	};
	let (chain_part, beneficiary) = split_location_into_chain_part_and_beneficiary(dest).unwrap();

	ParaA::execute_with(|| {
		let asset = currency_to_asset(parachain::CurrencyId::ForeignAsset(source_id), 100);
		// free execution, full amount received
		assert_ok!(PolkadotXcm::transfer_assets(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(VersionedLocation::from(chain_part)),
			Box::new(VersionedLocation::from(beneficiary)),
			Box::new(VersionedAssets::from(vec![asset].into())),
			0,
			WeightLimit::Limited(Weight::from_parts(40000u64, DEFAULT_PROOF_SIZE))
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 4000009000);
	});

	// What we will do now is transfer this relay tokens from the derived account to the sovereign
	// again
	Relay::execute_with(|| {
		// free execution,x	 full amount received
		assert!(RelayBalances::free_balance(&para_a_account()) == 4000009000);
		0
	});

	// We send the xcm transact operation to parent
	let dest = Location {
		parents: 1,
		interior: [].into(),
	};

	// Encode the call. Balances transact to para_a_account
	// First index
	let mut encoded: Vec<u8> = Vec::new();
	let index = <relay_chain::Runtime as frame_system::Config>::PalletInfo::index::<
		relay_chain::Balances,
	>()
	.unwrap() as u8;

	encoded.push(index);

	// Then call bytes
	let mut call_bytes = pallet_balances::Call::<relay_chain::Runtime>::transfer_allow_death {
		dest: para_a_account(),
		value: 100u32.into(),
	}
	.encode();
	encoded.append(&mut call_bytes);

	let utility_bytes = parachain::MockTransactors::Relay.encode_call(
		xcm_primitives::UtilityAvailableCalls::AsDerivative(0, encoded),
	);

	let total_weight = 4000009000u64;
	// Root can directly pass the execution byes to the sovereign
	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_sovereign(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::from(dest)),
			Some(PARAALICE.into()),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					Location::parent()
				))),
				// 1-1 fee-weight mapping
				fee_amount: Some(total_weight as u128)
			},
			utility_bytes,
			OriginKind::SovereignAccount,
			TransactWeights {
				transact_required_weight_at_most: 4000000000.into(),
				overall_weight: Some(Limited(total_weight.into()))
			},
			true
		));
	});

	Relay::execute_with(|| {
		// free execution,x	 full amount received
		// 4000005186 refunded + 100 transferred = 4000005286
		assert_eq!(RelayBalances::free_balance(&para_a_account()), 4000005286);

		assert_eq!(RelayBalances::free_balance(&registered_address), 0);
	});
}

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
			Box::new(VersionedLocation::from(dest).clone().into()),
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
	let source_location = para_a_balances.try_into().expect("convert to v3");
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
			Box::new(VersionedAssets::from(vec![asset].into())),
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

#[test]
fn receive_asset_with_no_sufficients_not_possible_if_non_existent_account() {
	MockNet::reset();

	let fresh_account = [2u8; 20];
	let source_location = parachain::AssetType::Xcm(xcm::v3::Location::parent());
	let source_id: parachain::AssetId = source_location.clone().into();
	let asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};
	// register relay asset in parachain A
	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata,
			1u128,
			false
		));
		assert_ok!(add_supported_asset(source_location, 0));
	});

	// Actually send relay asset to parachain
	let dest: Location = AccountKey20 {
		network: None,
		key: fresh_account,
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

	// parachain should not have received assets
	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &fresh_account.into()), 0);
	});

	// Send native token to fresh_account
	ParaA::execute_with(|| {
		assert_ok!(ParaBalances::transfer_allow_death(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			fresh_account.into(),
			100
		));
	});

	// Re-send tokens
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::limited_reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::from(dest).clone().into()),
			Box::new(([], 123).into()),
			0,
			WeightLimit::Unlimited
		));
	});

	// parachain should have received assets
	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &fresh_account.into()), 123);
	});
}

#[test]
fn receive_assets_with_sufficients_true_allows_non_funded_account_to_receive_assets() {
	MockNet::reset();

	let fresh_account = [2u8; 20];
	let source_location = parachain::AssetType::Xcm(xcm::v3::Location::parent());
	let source_id: parachain::AssetId = source_location.clone().into();
	let asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};
	// register relay asset in parachain A
	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata,
			1u128,
			true
		));
		assert_ok!(add_supported_asset(source_location, 0));
	});

	// Actually send relay asset to parachain
	let dest: Location = AccountKey20 {
		network: None,
		key: fresh_account,
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

	// parachain should have received assets
	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &fresh_account.into()), 123);
	});
}

#[test]
fn evm_account_receiving_assets_should_handle_sufficients_ref_count() {
	MockNet::reset();

	let mut sufficient_account = [0u8; 20];
	sufficient_account[0..20].copy_from_slice(&evm_account()[..]);

	let evm_account_id = parachain::AccountId::from(sufficient_account);

	// Evm account is self sufficient
	ParaA::execute_with(|| {
		assert_eq!(parachain::System::account(evm_account_id).sufficients, 1);
	});

	let source_location = parachain::AssetType::Xcm(xcm::v3::Location::parent());
	let asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};
	// register relay asset in parachain A
	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata,
			1u128,
			true
		));
		assert_ok!(add_supported_asset(source_location, 0));
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
	MockNet::reset();

	// Test account has nonce 1 on genesis.
	let mut sufficient_account = [0u8; 20];
	sufficient_account[0..20].copy_from_slice(&evm_account()[..]);

	let evm_account_id = parachain::AccountId::from(sufficient_account);

	let source_location = parachain::AssetType::Xcm(xcm::v3::Location::parent());
	let source_id: parachain::AssetId = source_location.clone().into();
	let asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};
	// register relay asset in parachain A
	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata,
			1u128,
			false
		));
		assert_ok!(add_supported_asset(source_location, 0));
	});

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

#[test]
fn test_statemint_like() {
	MockNet::reset();

	let dest_para = Location::new(1, [Parachain(1)]);

	let sov = xcm_builder::SiblingParachainConvertsVia::<
		polkadot_parachain::primitives::Sibling,
		statemint_like::AccountId,
	>::convert_location(&dest_para)
	.unwrap();

	let statemint_asset_a_balances = Location::new(
		1,
		[
			Parachain(1000),
			PalletInstance(5),
			xcm::latest::prelude::GeneralIndex(0u128),
		],
	);
	let source_location = statemint_asset_a_balances
		.try_into()
		.expect("convert to v3");
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"StatemintToken".to_vec(),
		symbol: b"StatemintToken".to_vec(),
		decimals: 12,
	};

	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata.clone(),
			1u128,
			true
		));
		assert_ok!(add_supported_asset(source_location, 0));
	});

	Statemint::execute_with(|| {
		// Set new prefix
		statemint_like::PrefixChanger::set_prefix(
			PalletInstance(<StatemintAssets as PalletInfoAccess>::index() as u8).into(),
		);
		assert_ok!(StatemintAssets::create(
			statemint_like::RuntimeOrigin::signed(RELAYALICE),
			0,
			RELAYALICE,
			1
		));

		assert_ok!(StatemintAssets::mint(
			statemint_like::RuntimeOrigin::signed(RELAYALICE),
			0,
			RELAYALICE,
			300000000000000
		));

		// This is needed, since the asset is created as non-sufficient
		assert_ok!(StatemintBalances::transfer_allow_death(
			statemint_like::RuntimeOrigin::signed(RELAYALICE),
			sov,
			100000000000000
		));

		// Actually send relay asset to parachain
		let dest: Location = AccountKey20 {
			network: None,
			key: PARAALICE,
		}
		.into();

		// Send asset with previous prefix
		assert_ok!(StatemintChainPalletXcm::limited_reserve_transfer_assets(
			statemint_like::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Location::new(1, [Parachain(1)]).into()),
			Box::new(VersionedLocation::from(dest).clone().into()),
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

	ParaA::execute_with(|| {
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 123);
	});
}

#[test]
fn send_statemint_asset_from_para_a_to_statemint_with_relay_fee() {
	MockNet::reset();

	// Relay asset
	let relay_location = parachain::AssetType::Xcm(xcm::v3::Location::parent());
	let source_relay_id: parachain::AssetId = relay_location.clone().into();

	let relay_asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};

	// Statemint asset
	let statemint_asset = Location::new(
		1,
		[
			Parachain(1000u32),
			PalletInstance(5u8),
			GeneralIndex(10u128),
		],
	);
	let statemint_location_asset = statemint_asset.try_into().expect("convert to v3");
	let source_statemint_asset_id: parachain::AssetId = statemint_location_asset.clone().into();

	let asset_metadata_statemint_asset = parachain::AssetMetadata {
		name: b"USDC".to_vec(),
		symbol: b"USDC".to_vec(),
		decimals: 12,
	};

	let dest_para = Location::new(1, [Parachain(1)]);

	let sov = xcm_builder::SiblingParachainConvertsVia::<
		polkadot_parachain::primitives::Sibling,
		statemint_like::AccountId,
	>::convert_location(&dest_para)
	.unwrap();

	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			relay_location.clone(),
			relay_asset_metadata,
			1u128,
			true
		));
		assert_ok!(add_supported_asset(relay_location, 0));

		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			statemint_location_asset.clone(),
			asset_metadata_statemint_asset,
			1u128,
			true
		));
		assert_ok!(add_supported_asset(statemint_location_asset, 0));
	});

	let parachain_beneficiary_from_relay: Location = Junction::AccountKey20 {
		network: None,
		key: PARAALICE,
	}
	.into();

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
		let parachain_beneficiary_from_statemint: Location = AccountKey20 {
			network: None,
			key: PARAALICE,
		}
		.into();

		// Send with new prefix
		assert_ok!(StatemintChainPalletXcm::limited_reserve_transfer_assets(
			statemint_like::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Location::new(1, [Parachain(1)]).into()),
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
			Box::new(VersionedAssets::from(vec![asset_fee, asset].into())),
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
	MockNet::reset();

	// Relay asset
	let relay_location = parachain::AssetType::Xcm(xcm::v3::Location::parent());
	let source_relay_id: parachain::AssetId = relay_location.clone().into();

	let relay_asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};

	let dest_para = Location::new(1, [Parachain(1)]);

	let sov = xcm_builder::SiblingParachainConvertsVia::<
		polkadot_parachain::primitives::Sibling,
		statemint_like::AccountId,
	>::convert_location(&dest_para)
	.unwrap();

	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			relay_location.clone(),
			relay_asset_metadata,
			1u128,
			true
		));
		XcmWeightTrader::set_asset_price(Location::parent(), 0u128);
	});

	let parachain_beneficiary_absolute: Location = Junction::AccountKey20 {
		network: None,
		key: PARAALICE,
	}
	.into();

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
			Box::new(Location::new(1, [Parachain(1)]).into()),
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

	ParaA::execute_with(|| {
		// Alice should have received the DOTs
		assert_eq!(Assets::balance(source_relay_id, &PARAALICE.into()), 200);
	});

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

	// Finally we test that we are able to send back the DOTs to AssetHub from the ParaA
	ParaA::execute_with(|| {
		let asset = currency_to_asset(parachain::CurrencyId::ForeignAsset(source_relay_id), 100);
		assert_ok!(PolkadotXcm::transfer_assets(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(VersionedLocation::from(chain_part)),
			Box::new(VersionedLocation::from(beneficiary)),
			Box::new(VersionedAssets::from(vec![asset].into())),
			0,
			WeightLimit::Limited(Weight::from_parts(40000u64, DEFAULT_PROOF_SIZE))
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
	Statemint::execute_with(|| {
		// Now send those tokens to ParaA
		assert_ok!(StatemintChainPalletXcm::limited_reserve_transfer_assets(
			statemint_like::RuntimeOrigin::signed(RELAYBOB),
			Box::new(Location::new(1, [Parachain(1)]).into()),
			Box::new(
				VersionedLocation::from(parachain_beneficiary_absolute)
					.clone()
					.into()
			),
			Box::new((Location::parent(), 100).into()),
			0,
			WeightLimit::Unlimited
		));

		// 100 DOTs were deducted from Bob's account
		assert_eq!(StatemintBalances::free_balance(RELAYBOB), INITIAL_BALANCE);
	});

	ParaA::execute_with(|| {
		// Alice should have received 100 DOTs
		assert_eq!(Assets::balance(source_relay_id, &PARAALICE.into()), 200);
	});
}

#[test]
fn send_dot_from_moonbeam_to_statemint_via_xtokens_transfer_with_fee() {
	MockNet::reset();

	// Relay asset
	let relay_location = parachain::AssetType::Xcm(xcm::v3::Location::parent());
	let source_relay_id: parachain::AssetId = relay_location.clone().into();

	let relay_asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};

	let dest_para = Location::new(1, [Parachain(1)]);

	let sov = xcm_builder::SiblingParachainConvertsVia::<
		polkadot_parachain::primitives::Sibling,
		statemint_like::AccountId,
	>::convert_location(&dest_para)
	.unwrap();

	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			relay_location.clone(),
			relay_asset_metadata,
			1u128,
			true
		));
		XcmWeightTrader::set_asset_price(Location::parent(), 0u128);
	});

	let parachain_beneficiary_absolute: Location = Junction::AccountKey20 {
		network: None,
		key: PARAALICE,
	}
	.into();

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
			Box::new(Location::new(1, [Parachain(1)]).into()),
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

	ParaA::execute_with(|| {
		// Alice should have received the DOTs
		assert_eq!(Assets::balance(source_relay_id, &PARAALICE.into()), 200);
	});

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

	// Finally we test that we are able to send back the DOTs to AssetHub from the ParaA
	ParaA::execute_with(|| {
		let asset = currency_to_asset(parachain::CurrencyId::ForeignAsset(source_relay_id), 100);
		let asset_fee = currency_to_asset(parachain::CurrencyId::ForeignAsset(source_relay_id), 10);
		assert_ok!(PolkadotXcm::transfer_assets(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(VersionedLocation::from(chain_part)),
			Box::new(VersionedLocation::from(beneficiary)),
			Box::new(VersionedAssets::from(vec![asset_fee, asset].into())),
			0,
			WeightLimit::Limited(Weight::from_parts(40000u64, DEFAULT_PROOF_SIZE))
		));

		assert_eq!(Assets::balance(source_relay_id, &PARAALICE.into()), 90);
	});

	Statemint::execute_with(|| {
		// Free execution: check that Bob received the tokens back in AssetHub
		assert_eq!(
			StatemintBalances::free_balance(RELAYBOB),
			INITIAL_BALANCE + 110
		);
	});

	// Send back tokens from AH to ParaA from Bob's account
	Statemint::execute_with(|| {
		// Now send those tokens to ParaA
		assert_ok!(StatemintChainPalletXcm::limited_reserve_transfer_assets(
			statemint_like::RuntimeOrigin::signed(RELAYBOB),
			Box::new(Location::new(1, [Parachain(1)]).into()),
			Box::new(
				VersionedLocation::from(parachain_beneficiary_absolute)
					.clone()
					.into()
			),
			Box::new((Location::parent(), 100).into()),
			0,
			WeightLimit::Unlimited
		));

		// 100 DOTs were deducted from Bob's account
		assert_eq!(
			StatemintBalances::free_balance(RELAYBOB),
			INITIAL_BALANCE + 10
		);
	});

	ParaA::execute_with(|| {
		// Alice should have received 100 DOTs
		assert_eq!(Assets::balance(source_relay_id, &PARAALICE.into()), 190);
	});
}

#[test]
fn send_dot_from_moonbeam_to_statemint_via_xtokens_transfer_multiasset() {
	MockNet::reset();

	// Relay asset
	let relay_location = parachain::AssetType::Xcm(xcm::v3::Location::parent());
	let source_relay_id: parachain::AssetId = relay_location.clone().into();

	let relay_asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};

	let dest_para = Location::new(1, [Parachain(1)]);

	let sov = xcm_builder::SiblingParachainConvertsVia::<
		polkadot_parachain::primitives::Sibling,
		statemint_like::AccountId,
	>::convert_location(&dest_para)
	.unwrap();

	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			relay_location.clone(),
			relay_asset_metadata,
			1u128,
			true
		));
		XcmWeightTrader::set_asset_price(Location::parent(), 0u128);
	});

	let parachain_beneficiary_absolute: Location = Junction::AccountKey20 {
		network: None,
		key: PARAALICE,
	}
	.into();

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
			Box::new(Location::new(1, [Parachain(1)]).into()),
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

	ParaA::execute_with(|| {
		// Alice should have received the DOTs
		assert_eq!(Assets::balance(source_relay_id, &PARAALICE.into()), 200);
	});

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
			Box::new(VersionedAssets::from(asset.into())),
			0,
			WeightLimit::Limited(Weight::from_parts(40000u64, DEFAULT_PROOF_SIZE))
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
	Statemint::execute_with(|| {
		// Now send those tokens to ParaA
		assert_ok!(StatemintChainPalletXcm::limited_reserve_transfer_assets(
			statemint_like::RuntimeOrigin::signed(RELAYBOB),
			Box::new(Location::new(1, [Parachain(1)]).into()),
			Box::new(
				VersionedLocation::from(parachain_beneficiary_absolute)
					.clone()
					.into()
			),
			Box::new((Location::parent(), 100).into()),
			0,
			WeightLimit::Unlimited
		));

		// 100 DOTs were deducted from Bob's account
		assert_eq!(StatemintBalances::free_balance(RELAYBOB), INITIAL_BALANCE);
	});

	ParaA::execute_with(|| {
		// Alice should have received 100 DOTs
		assert_eq!(Assets::balance(source_relay_id, &PARAALICE.into()), 200);
	});
}

#[test]
fn send_dot_from_moonbeam_to_statemint_via_xtokens_transfer_multicurrencies() {
	MockNet::reset();

	// Relay asset
	let relay_location = parachain::AssetType::Xcm(xcm::v3::Location::parent());
	let source_relay_id: parachain::AssetId = relay_location.clone().into();

	let relay_asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};

	// Statemint asset
	let statemint_asset = Location::new(
		1,
		[
			Parachain(1000u32),
			PalletInstance(5u8),
			GeneralIndex(10u128),
		],
	);
	let statemint_location_asset = statemint_asset.try_into().expect("convert to v3");
	let source_statemint_asset_id: parachain::AssetId = statemint_location_asset.clone().into();

	let asset_metadata_statemint_asset = parachain::AssetMetadata {
		name: b"USDC".to_vec(),
		symbol: b"USDC".to_vec(),
		decimals: 12,
	};

	let dest_para = Location::new(1, [Parachain(1)]);

	let sov = xcm_builder::SiblingParachainConvertsVia::<
		polkadot_parachain::primitives::Sibling,
		statemint_like::AccountId,
	>::convert_location(&dest_para)
	.unwrap();

	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			relay_location.clone(),
			relay_asset_metadata,
			1u128,
			true
		));
		XcmWeightTrader::set_asset_price(Location::parent(), 0u128);

		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			statemint_location_asset.clone(),
			asset_metadata_statemint_asset,
			1u128,
			true
		));
		XcmWeightTrader::set_asset_price(statemint_asset, 0u128);
	});

	let parachain_beneficiary_absolute: Location = Junction::AccountKey20 {
		network: None,
		key: PARAALICE,
	}
	.into();

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

	// Send DOTs and USDC from AssetHub to ParaA (Moonbeam)
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

		// Now send relay tokens to ParaA
		assert_ok!(StatemintChainPalletXcm::limited_reserve_transfer_assets(
			statemint_like::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Location::new(1, [Parachain(1)]).into()),
			Box::new(
				VersionedLocation::from(parachain_beneficiary_absolute.clone())
					.clone()
					.into()
			),
			Box::new((Location::parent(), 200).into()),
			0,
			WeightLimit::Unlimited
		));

		// Send USDC
		assert_ok!(StatemintChainPalletXcm::limited_reserve_transfer_assets(
			statemint_like::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Location::new(1, [Parachain(1)]).into()),
			Box::new(
				VersionedLocation::from(parachain_beneficiary_absolute.clone())
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

	ParaA::execute_with(|| {
		// Alice should have received the DOTs
		assert_eq!(Assets::balance(source_relay_id, &PARAALICE.into()), 200);

		// Alice has received 125 USDC
		assert_eq!(
			Assets::balance(source_statemint_asset_id, &PARAALICE.into()),
			125
		);
	});

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
	// Finally we test that we are able to send back the DOTs to AssetHub from the ParaA
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
			Box::new(VersionedAssets::from(vec![asset_fee, asset].into())),
			0,
			WeightLimit::Limited(Weight::from_parts(80_000_000u64, 100_000u64))
		));

		assert_eq!(Assets::balance(source_relay_id, &PARAALICE.into()), 100);
	});

	Statemint::execute_with(|| {
		// Check that Bob received relay tokens back in AssetHub
		assert_eq!(
			StatemintBalances::free_balance(RELAYBOB),
			INITIAL_BALANCE + 100
		);

		// Check that BOB received 100 USDC on AssetHub
		assert_eq!(StatemintAssets::account_balances(RELAYBOB), vec![(10, 100)]);
	});

	// Send back tokens from AH to ParaA from Bob's account
	Statemint::execute_with(|| {
		let bob_previous_balance = StatemintBalances::free_balance(RELAYBOB);

		// Now send those tokens to ParaA
		assert_ok!(StatemintChainPalletXcm::limited_reserve_transfer_assets(
			statemint_like::RuntimeOrigin::signed(RELAYBOB),
			Box::new(Location::new(1, [Parachain(1)]).into()),
			Box::new(
				VersionedLocation::from(parachain_beneficiary_absolute)
					.clone()
					.into()
			),
			Box::new((Location::parent(), 100).into()),
			0,
			WeightLimit::Unlimited
		));

		// 100 DOTs were deducted from Bob's account
		assert_eq!(
			StatemintBalances::free_balance(RELAYBOB),
			bob_previous_balance - 100
		);
	});

	ParaA::execute_with(|| {
		// Alice should have received 100 DOTs
		assert_eq!(Assets::balance(source_relay_id, &PARAALICE.into()), 200);
	});
}

#[test]
fn send_dot_from_moonbeam_to_statemint_via_xtokens_transfer_multiassets() {
	MockNet::reset();

	// Relay asset
	let relay_location = parachain::AssetType::Xcm(xcm::v3::Location::parent());
	let source_relay_id: parachain::AssetId = relay_location.clone().into();

	let relay_asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};

	// Statemint asset
	let statemint_asset = Location::new(
		1,
		[
			Parachain(1000u32),
			PalletInstance(5u8),
			GeneralIndex(10u128),
		],
	);
	let statemint_location_asset = statemint_asset.try_into().expect("convert to v3");
	let source_statemint_asset_id: parachain::AssetId = statemint_location_asset.clone().into();

	let asset_metadata_statemint_asset = parachain::AssetMetadata {
		name: b"USDC".to_vec(),
		symbol: b"USDC".to_vec(),
		decimals: 12,
	};

	let dest_para = Location::new(1, [Parachain(1)]);

	let sov = xcm_builder::SiblingParachainConvertsVia::<
		polkadot_parachain::primitives::Sibling,
		statemint_like::AccountId,
	>::convert_location(&dest_para)
	.unwrap();

	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			relay_location.clone(),
			relay_asset_metadata,
			1u128,
			true
		));
		XcmWeightTrader::set_asset_price(Location::parent(), 0u128);

		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			statemint_location_asset.clone(),
			asset_metadata_statemint_asset,
			1u128,
			true
		));
		XcmWeightTrader::set_asset_price(statemint_asset.clone(), 0u128);
	});

	let parachain_beneficiary_absolute: Location = Junction::AccountKey20 {
		network: None,
		key: PARAALICE,
	}
	.into();

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

	// Send DOTs and USDC from AssetHub to ParaA (Moonbeam)
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

		// Now send relay tokens to ParaA
		assert_ok!(StatemintChainPalletXcm::limited_reserve_transfer_assets(
			statemint_like::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Location::new(1, [Parachain(1)]).into()),
			Box::new(
				VersionedLocation::from(parachain_beneficiary_absolute.clone())
					.clone()
					.into()
			),
			Box::new((Location::parent(), 200).into()),
			0,
			WeightLimit::Unlimited
		));

		// Send USDC
		assert_ok!(StatemintChainPalletXcm::limited_reserve_transfer_assets(
			statemint_like::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Location::new(1, [Parachain(1)]).into()),
			Box::new(
				VersionedLocation::from(parachain_beneficiary_absolute.clone())
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

	ParaA::execute_with(|| {
		// Alice should have received the DOTs
		assert_eq!(Assets::balance(source_relay_id, &PARAALICE.into()), 200);

		// Alice has received 125 USDC
		assert_eq!(
			Assets::balance(source_statemint_asset_id, &PARAALICE.into()),
			125
		);
	});

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

	let statemint_asset_to_send = Asset {
		id: AssetId(statemint_asset),
		fun: Fungibility::Fungible(100),
	};

	let relay_asset_to_send = Asset {
		id: AssetId(Location::parent()),
		fun: Fungibility::Fungible(100),
	};

	let (chain_part, beneficiary) = split_location_into_chain_part_and_beneficiary(dest).unwrap();
	let assets_to_send: XcmAssets =
		XcmAssets::from(vec![statemint_asset_to_send, relay_asset_to_send.clone()]);

	// For some reason the order of the assets is inverted when creating the array above.
	// We need to use relay asset for fees, so we pick index 0.
	assert_eq!(assets_to_send.get(0).unwrap(), &relay_asset_to_send);

	// Finally we test that we are able to send back the DOTs to AssetHub from the ParaA
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

	Statemint::execute_with(|| {
		// Check that Bob received relay tokens back in AssetHub
		assert_eq!(
			StatemintBalances::free_balance(RELAYBOB),
			INITIAL_BALANCE + 100
		);

		// Check that BOB received 100 USDC on AssetHub
		assert_eq!(StatemintAssets::account_balances(RELAYBOB), vec![(10, 100)]);
	});

	// Send back tokens from AH to ParaA from Bob's account
	Statemint::execute_with(|| {
		let bob_previous_balance = StatemintBalances::free_balance(RELAYBOB);

		// Now send those tokens to ParaA
		assert_ok!(StatemintChainPalletXcm::limited_reserve_transfer_assets(
			statemint_like::RuntimeOrigin::signed(RELAYBOB),
			Box::new(Location::new(1, [Parachain(1)]).into()),
			Box::new(
				VersionedLocation::from(parachain_beneficiary_absolute)
					.clone()
					.into()
			),
			Box::new((Location::parent(), 100).into()),
			0,
			WeightLimit::Unlimited
		));

		// 100 DOTs were deducted from Bob's account
		assert_eq!(
			StatemintBalances::free_balance(RELAYBOB),
			bob_previous_balance - 100
		);
	});

	ParaA::execute_with(|| {
		// Alice should have received 100 DOTs
		assert_eq!(Assets::balance(source_relay_id, &PARAALICE.into()), 200);
	});
}

#[test]
fn transact_through_signed_multilocation() {
	MockNet::reset();
	let mut ancestry = Location::parent();

	ParaA::execute_with(|| {
		// Root can set transact info
		assert_ok!(XcmTransactor::set_transact_info(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::from(Location::parent())),
			// Relay charges 1000 for every instruction, and we have 3, so 3000
			3000.into(),
			20000000000.into(),
			// 4 instructions in transact through signed
			Some(4000.into())
		));
		// Root can set transact info
		assert_ok!(XcmTransactor::set_fee_per_second(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::from(Location::parent())),
			WEIGHT_REF_TIME_PER_SECOND as u128,
		));
		ancestry = parachain::UniversalLocation::get().into();
	});

	// Let's construct the Junction that we will append with DescendOrigin
	let signed_origin: Junctions = [AccountKey20 {
		network: None,
		key: PARAALICE,
	}]
	.into();

	let mut descend_origin_multilocation = parachain::SelfLocation::get();
	descend_origin_multilocation
		.append_with(signed_origin)
		.unwrap();

	// To convert it to what the relay will see instead of us
	descend_origin_multilocation
		.reanchor(&Location::parent(), &ancestry.interior)
		.unwrap();

	let derived = xcm_builder::Account32Hash::<
		relay_chain::KusamaNetwork,
		relay_chain::AccountId,
	>::convert_location(&descend_origin_multilocation)
	.unwrap();

	Relay::execute_with(|| {
		// free execution, full amount received
		assert_ok!(RelayBalances::transfer_allow_death(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			derived.clone(),
			4000004100u128,
		));
		// derived account has all funds
		assert!(RelayBalances::free_balance(&derived) == 4000004100);
		// sovereign account has 0 funds
		assert!(RelayBalances::free_balance(&para_a_account()) == 0);
	});

	// Encode the call. Balances transact to para_a_account
	// First index
	let mut encoded: Vec<u8> = Vec::new();
	let index = <relay_chain::Runtime as frame_system::Config>::PalletInfo::index::<
		relay_chain::Balances,
	>()
	.unwrap() as u8;

	encoded.push(index);

	// Then call bytes
	let mut call_bytes = pallet_balances::Call::<relay_chain::Runtime>::transfer_allow_death {
		// 100 to sovereign
		dest: para_a_account(),
		value: 100u32.into(),
	}
	.encode();
	encoded.append(&mut call_bytes);

	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_signed(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(xcm::VersionedLocation::from(Location::parent())),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					Location::parent()
				))),
				fee_amount: None
			},
			encoded,
			// 4000000000 for transfer + 4000 for XCM
			TransactWeights {
				transact_required_weight_at_most: 4000000000.into(),
				overall_weight: None
			},
			false
		));
	});

	Relay::execute_with(|| {
		assert!(RelayBalances::free_balance(&para_a_account()) == 100);

		assert!(RelayBalances::free_balance(&derived) == 0);
	});
}

#[test]
fn transact_through_signed_multilocation_custom_fee_and_weight() {
	MockNet::reset();
	let mut ancestry = Location::parent();

	ParaA::execute_with(|| {
		ancestry = parachain::UniversalLocation::get().into();
	});

	// Let's construct the Junction that we will append with DescendOrigin
	let signed_origin: Junctions = [AccountKey20 {
		network: None,
		key: PARAALICE,
	}]
	.into();

	let mut descend_origin_multilocation = parachain::SelfLocation::get();
	descend_origin_multilocation
		.append_with(signed_origin)
		.unwrap();

	// To convert it to what the relay will see instead of us
	descend_origin_multilocation
		.reanchor(&Location::parent(), &ancestry.interior)
		.unwrap();

	let derived = xcm_builder::Account32Hash::<
		relay_chain::KusamaNetwork,
		relay_chain::AccountId,
	>::convert_location(&descend_origin_multilocation)
	.unwrap();

	Relay::execute_with(|| {
		// free execution, full amount received
		assert_ok!(RelayBalances::transfer_allow_death(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			derived.clone(),
			4000004100u128,
		));
		// derived account has all funds
		assert!(RelayBalances::free_balance(&derived) == 4000004100);
		// sovereign account has 0 funds
		assert!(RelayBalances::free_balance(&para_a_account()) == 0);
	});

	// Encode the call. Balances transact to para_a_account
	// First index
	let mut encoded: Vec<u8> = Vec::new();
	let index = <relay_chain::Runtime as frame_system::Config>::PalletInfo::index::<
		relay_chain::Balances,
	>()
	.unwrap() as u8;

	encoded.push(index);

	// Then call bytes
	let mut call_bytes = pallet_balances::Call::<relay_chain::Runtime>::transfer_allow_death {
		// 100 to sovereign
		dest: para_a_account(),
		value: 100u32.into(),
	}
	.encode();
	encoded.append(&mut call_bytes);

	let total_weight = 4000004000u64;
	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_signed(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(xcm::VersionedLocation::from(Location::parent())),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					Location::parent()
				))),
				fee_amount: Some(total_weight as u128)
			},
			encoded,
			// 4000000000 for transfer + 4000 for XCM
			TransactWeights {
				transact_required_weight_at_most: 4000000000.into(),
				overall_weight: Some(Limited(total_weight.into()))
			},
			false
		));
	});

	Relay::execute_with(|| {
		assert!(RelayBalances::free_balance(&para_a_account()) == 100);

		assert!(RelayBalances::free_balance(&derived) == 0);
	});
}

#[test]
fn transact_through_signed_multilocation_custom_fee_and_weight_refund() {
	MockNet::reset();
	let mut ancestry = Location::parent();

	ParaA::execute_with(|| {
		ancestry = parachain::UniversalLocation::get().into();
	});

	// Let's construct the Junction that we will append with DescendOrigin
	let signed_origin: Junctions = [AccountKey20 {
		network: None,
		key: PARAALICE,
	}]
	.into();

	let mut descend_origin_multilocation = parachain::SelfLocation::get();
	descend_origin_multilocation
		.append_with(signed_origin)
		.unwrap();

	// To convert it to what the relay will see instead of us
	descend_origin_multilocation
		.reanchor(&Location::parent(), &ancestry.interior)
		.unwrap();

	let derived = xcm_builder::Account32Hash::<
		relay_chain::KusamaNetwork,
		relay_chain::AccountId,
	>::convert_location(&descend_origin_multilocation)
	.unwrap();

	Relay::execute_with(|| {
		// free execution, full amount received
		assert_ok!(RelayBalances::transfer_allow_death(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			derived.clone(),
			4000009100u128,
		));
		// derived account has all funds
		assert!(RelayBalances::free_balance(&derived) == 4000009100);
		// sovereign account has 0 funds
		assert!(RelayBalances::free_balance(&para_a_account()) == 0);
	});

	// Encode the call. Balances transact to para_a_account
	// First index
	let mut encoded: Vec<u8> = Vec::new();
	let index = <relay_chain::Runtime as frame_system::Config>::PalletInfo::index::<
		relay_chain::Balances,
	>()
	.unwrap() as u8;

	encoded.push(index);

	// Then call bytes
	let mut call_bytes = pallet_balances::Call::<relay_chain::Runtime>::transfer_allow_death {
		// 100 to sovereign
		dest: para_a_account(),
		value: 100u32.into(),
	}
	.encode();
	encoded.append(&mut call_bytes);

	let total_weight = 4000009000u64;
	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_signed(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(xcm::VersionedLocation::from(Location::parent())),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					Location::parent()
				))),
				fee_amount: Some(total_weight as u128)
			},
			encoded,
			// 4000000000 for transfer + 9000 for XCM
			TransactWeights {
				transact_required_weight_at_most: 4000000000.into(),
				overall_weight: Some(Limited(total_weight.into()))
			},
			true
		));
	});

	Relay::execute_with(|| {
		// 100 transferred
		assert_eq!(RelayBalances::free_balance(&para_a_account()), 100);

		// 4000005186 refunded
		assert_eq!(RelayBalances::free_balance(&derived), 4000005186);
	});
}

#[test]
fn transact_through_signed_multilocation_para_to_para() {
	MockNet::reset();
	let mut ancestry = Location::parent();

	let para_b_location = Location::new(1, [Parachain(2)]);

	let para_b_balances = Location::new(1, [Parachain(2), PalletInstance(1u8)]);

	ParaA::execute_with(|| {
		// Root can set transact info
		assert_ok!(XcmTransactor::set_transact_info(
			parachain::RuntimeOrigin::root(),
			// ParaB
			Box::new(xcm::VersionedLocation::from(para_b_location.clone())),
			// Para charges 1000 for every instruction, and we have 3, so 3
			3.into(),
			20000000000.into(),
			// 4 instructions in transact through signed
			Some(4.into())
		));
		// Root can set transact info
		assert_ok!(XcmTransactor::set_fee_per_second(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::from(para_b_balances.clone())),
			parachain::ParaTokensPerSecond::get(),
		));
		ancestry = parachain::UniversalLocation::get().into();
	});

	// Let's construct the Junction that we will append with DescendOrigin
	let signed_origin: Junctions = [AccountKey20 {
		network: None,
		key: PARAALICE,
	}]
	.into();

	let mut descend_origin_multilocation = parachain::SelfLocation::get();
	descend_origin_multilocation
		.append_with(signed_origin)
		.unwrap();

	// To convert it to what the paraB will see instead of us
	descend_origin_multilocation
		.reanchor(&para_b_location, &ancestry.interior)
		.unwrap();

	let derived = xcm_builder::HashedDescription::<
		parachain::AccountId,
		xcm_builder::DescribeFamily<xcm_builder::DescribeAllTerminal>,
	>::convert_location(&descend_origin_multilocation)
	.unwrap();

	ParaB::execute_with(|| {
		// free execution, full amount received
		assert_ok!(ParaBalances::transfer_allow_death(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			derived.clone(),
			4000000104u128,
		));
		// derived account has all funds
		assert!(ParaBalances::free_balance(&derived) == 4000000104);
		// sovereign account has 0 funds
		assert!(ParaBalances::free_balance(&para_a_account_20()) == 0);
	});

	// Encode the call. Balances transact to para_a_account
	// First index
	let mut encoded: Vec<u8> = Vec::new();
	let index =
		<parachain::Runtime as frame_system::Config>::PalletInfo::index::<parachain::Balances>()
			.unwrap() as u8;

	encoded.push(index);

	// Then call bytes
	let mut call_bytes = pallet_balances::Call::<parachain::Runtime>::transfer_allow_death {
		// 100 to sovereign
		dest: para_a_account_20(),
		value: 100u32.into(),
	}
	.encode();
	encoded.append(&mut call_bytes);

	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_signed(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(xcm::VersionedLocation::from(para_b_location)),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					para_b_balances
				))),
				fee_amount: None
			},
			encoded,
			// 4000000000 for transfer + 4000 for XCM
			// 1-1 to fee
			TransactWeights {
				transact_required_weight_at_most: 4000000000.into(),
				overall_weight: None
			},
			false
		));
	});

	ParaB::execute_with(|| {
		assert!(ParaBalances::free_balance(&derived) == 0);

		assert!(ParaBalances::free_balance(&para_a_account_20()) == 100);
	});
}

#[test]
fn transact_through_signed_multilocation_para_to_para_refund() {
	MockNet::reset();
	let mut ancestry = Location::parent();

	let para_b_location = Location::new(1, [Parachain(2)]);

	let para_b_balances = Location::new(1, [Parachain(2), PalletInstance(1u8)]);

	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::set_fee_per_second(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::from(para_b_balances.clone())),
			parachain::ParaTokensPerSecond::get(),
		));
		ancestry = parachain::UniversalLocation::get().into();
	});

	// Let's construct the Junction that we will append with DescendOrigin
	let signed_origin: Junctions = [AccountKey20 {
		network: None,
		key: PARAALICE,
	}]
	.into();

	let mut descend_origin_multilocation = parachain::SelfLocation::get();
	descend_origin_multilocation
		.append_with(signed_origin)
		.unwrap();

	// To convert it to what the paraB will see instead of us
	descend_origin_multilocation
		.reanchor(&para_b_location, &ancestry.interior)
		.unwrap();

	let derived = xcm_builder::HashedDescription::<
		parachain::AccountId,
		xcm_builder::DescribeFamily<xcm_builder::DescribeAllTerminal>,
	>::convert_location(&descend_origin_multilocation)
	.unwrap();

	ParaB::execute_with(|| {
		// free execution, full amount received
		assert_ok!(ParaBalances::transfer_allow_death(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			derived.clone(),
			4000009100u128,
		));
		// derived account has all funds
		assert!(ParaBalances::free_balance(&derived) == 4000009100);
		// sovereign account has 0 funds
		assert!(ParaBalances::free_balance(&para_a_account_20()) == 0);
	});

	// Encode the call. Balances transact to para_a_account
	// First index
	let mut encoded: Vec<u8> = Vec::new();
	let index =
		<parachain::Runtime as frame_system::Config>::PalletInfo::index::<parachain::Balances>()
			.unwrap() as u8;

	encoded.push(index);

	// Then call bytes
	let mut call_bytes = pallet_balances::Call::<parachain::Runtime>::transfer_allow_death {
		// 100 to sovereign
		dest: para_a_account_20(),
		value: 100u32.into(),
	}
	.encode();
	encoded.append(&mut call_bytes);

	let overall_weight = 4000009000u64;
	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_signed(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(xcm::VersionedLocation::from(para_b_location)),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					para_b_balances
				))),
				fee_amount: Some(overall_weight as u128)
			},
			encoded,
			// 4000000000 for transfer + 9000 for XCM
			TransactWeights {
				transact_required_weight_at_most: 4000000000.into(),
				overall_weight: Some(Limited(overall_weight.into()))
			},
			true
		));
	});

	ParaB::execute_with(|| {
		// Check the derived account was refunded
		assert_eq!(ParaBalances::free_balance(&derived), 8993);

		// Check the transfer was executed
		assert_eq!(ParaBalances::free_balance(&para_a_account_20()), 100);
	});
}

#[test]
fn transact_through_signed_multilocation_para_to_para_ethereum() {
	MockNet::reset();
	let mut ancestry = Location::parent();

	let para_b_location = Location::new(1, [Parachain(2)]);

	let para_b_balances = Location::new(1, [Parachain(2), PalletInstance(1u8)]);

	ParaA::execute_with(|| {
		// Root can set transact info
		assert_ok!(XcmTransactor::set_transact_info(
			parachain::RuntimeOrigin::root(),
			// ParaB
			Box::new(xcm::VersionedLocation::from(para_b_location.clone())),
			// Para charges 1000 for every instruction, and we have 3, so 3
			3.into(),
			20000000000.into(),
			// 4 instructions in transact through signed
			Some(4.into())
		));
		// Root can set transact info
		assert_ok!(XcmTransactor::set_fee_per_second(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::from(para_b_balances.clone())),
			parachain::ParaTokensPerSecond::get(),
		));
		ancestry = parachain::UniversalLocation::get().into();
	});

	// Let's construct the Junction that we will append with DescendOrigin
	let signed_origin: Junctions = [AccountKey20 {
		network: None,
		key: PARAALICE,
	}]
	.into();

	let mut descend_origin_multilocation = parachain::SelfLocation::get();
	descend_origin_multilocation
		.append_with(signed_origin)
		.unwrap();

	// To convert it to what the paraB will see instead of us
	descend_origin_multilocation
		.reanchor(&para_b_location, &ancestry.interior)
		.unwrap();

	let derived = xcm_builder::HashedDescription::<
		parachain::AccountId,
		xcm_builder::DescribeFamily<xcm_builder::DescribeAllTerminal>,
	>::convert_location(&descend_origin_multilocation)
	.unwrap();

	let mut parachain_b_alice_balances_before = 0;
	ParaB::execute_with(|| {
		assert_ok!(ParaBalances::transfer_allow_death(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			derived.clone(),
			4000000104u128,
		));
		// derived account has all funds
		assert!(ParaBalances::free_balance(&derived) == 4000000104);
		// sovereign account has 0 funds
		assert!(ParaBalances::free_balance(&para_a_account_20()) == 0);

		parachain_b_alice_balances_before = ParaBalances::free_balance(&PARAALICE.into())
	});

	// Encode the call. Balances transact to para_a_account
	// First index
	let mut encoded: Vec<u8> = Vec::new();
	let index =
		<parachain::Runtime as frame_system::Config>::PalletInfo::index::<parachain::EthereumXcm>()
			.unwrap() as u8;

	encoded.push(index);

	use sp_core::U256;
	// Let's do a EVM transfer
	let eth_tx =
		xcm_primitives::EthereumXcmTransaction::V1(xcm_primitives::EthereumXcmTransactionV1 {
			gas_limit: U256::from(21000),
			fee_payment: xcm_primitives::EthereumXcmFee::Auto,
			action: pallet_ethereum::TransactionAction::Call(PARAALICE.into()),
			value: U256::from(100),
			input: BoundedVec::<
				u8,
				ConstU32<{ xcm_primitives::MAX_ETHEREUM_XCM_INPUT_SIZE }>
			>::try_from(vec![]).unwrap(),
			access_list: None,
		});

	// Then call bytes
	let mut call_bytes = pallet_ethereum_xcm::Call::<parachain::Runtime>::transact {
		xcm_transaction: eth_tx,
	}
	.encode();
	encoded.append(&mut call_bytes);

	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_signed(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(xcm::VersionedLocation::from(para_b_location)),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					para_b_balances
				))),
				fee_amount: None
			},
			encoded,
			// 4000000000 for transfer + 4000 for XCM
			// 1-1 to fee
			TransactWeights {
				transact_required_weight_at_most: 4000000000.into(),
				overall_weight: None
			},
			false
		));
	});

	ParaB::execute_with(|| {
		// Make sure the EVM transfer went through
		assert!(
			ParaBalances::free_balance(&PARAALICE.into())
				== parachain_b_alice_balances_before + 100
		);
	});
}

#[test]
fn transact_through_signed_multilocation_para_to_para_ethereum_no_proxy_fails() {
	MockNet::reset();
	let mut ancestry = Location::parent();

	let para_b_location = Location::new(1, [Parachain(2)]);

	let para_b_balances = Location::new(1, [Parachain(2), PalletInstance(1u8)]);

	ParaA::execute_with(|| {
		// Root can set transact info
		assert_ok!(XcmTransactor::set_transact_info(
			parachain::RuntimeOrigin::root(),
			// ParaB
			Box::new(xcm::VersionedLocation::from(para_b_location.clone())),
			// Para charges 1000 for every instruction, and we have 3, so 3
			3.into(),
			20000000000.into(),
			// 4 instructions in transact through signed
			Some(4.into())
		));
		// Root can set transact info
		assert_ok!(XcmTransactor::set_fee_per_second(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::from(para_b_balances.clone())),
			parachain::ParaTokensPerSecond::get(),
		));
		ancestry = parachain::UniversalLocation::get().into();
	});

	// Let's construct the Junction that we will append with DescendOrigin
	let signed_origin: Junctions = [AccountKey20 {
		network: None,
		key: PARAALICE,
	}]
	.into();

	let mut descend_origin_multilocation = parachain::SelfLocation::get();
	descend_origin_multilocation
		.append_with(signed_origin)
		.unwrap();

	// To convert it to what the paraB will see instead of us
	descend_origin_multilocation
		.reanchor(&para_b_location, &ancestry.interior)
		.unwrap();

	let derived = xcm_builder::HashedDescription::<
		parachain::AccountId,
		xcm_builder::DescribeFamily<xcm_builder::DescribeAllTerminal>,
	>::convert_location(&descend_origin_multilocation)
	.unwrap();

	let mut parachain_b_alice_balances_before = 0;
	ParaB::execute_with(|| {
		assert_ok!(ParaBalances::transfer_allow_death(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			derived.clone(),
			4000000104u128,
		));
		// derived account has all funds
		assert!(ParaBalances::free_balance(&derived) == 4000000104);
		// sovereign account has 0 funds
		assert!(ParaBalances::free_balance(&para_a_account_20()) == 0);

		parachain_b_alice_balances_before = ParaBalances::free_balance(&PARAALICE.into())
	});

	// Encode the call. Balances transact to para_a_account
	// First index
	let mut encoded: Vec<u8> = Vec::new();
	let index =
		<parachain::Runtime as frame_system::Config>::PalletInfo::index::<parachain::EthereumXcm>()
			.unwrap() as u8;

	encoded.push(index);

	use sp_core::U256;
	// Let's do a EVM transfer
	let eth_tx =
		xcm_primitives::EthereumXcmTransaction::V1(xcm_primitives::EthereumXcmTransactionV1 {
			gas_limit: U256::from(21000),
			fee_payment: xcm_primitives::EthereumXcmFee::Auto,
			action: pallet_ethereum::TransactionAction::Call(PARAALICE.into()),
			value: U256::from(100),
			input: BoundedVec::<
				u8,
				ConstU32<{ xcm_primitives::MAX_ETHEREUM_XCM_INPUT_SIZE }>
			>::try_from(vec![]).unwrap(),
			access_list: None,
		});

	// Then call bytes
	let mut call_bytes = pallet_ethereum_xcm::Call::<parachain::Runtime>::transact_through_proxy {
		transact_as: PARAALICE.into(),
		xcm_transaction: eth_tx,
	}
	.encode();
	encoded.append(&mut call_bytes);

	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_signed(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(xcm::VersionedLocation::from(para_b_location)),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					para_b_balances
				))),
				fee_amount: None
			},
			encoded,
			TransactWeights {
				transact_required_weight_at_most: 4000000000.into(),
				overall_weight: None
			},
			false
		));
	});

	ParaB::execute_with(|| {
		// Make sure the EVM transfer wasn't executed
		assert!(ParaBalances::free_balance(&PARAALICE.into()) == parachain_b_alice_balances_before);
	});
}

#[test]
fn transact_through_signed_multilocation_para_to_para_ethereum_proxy_succeeds() {
	MockNet::reset();
	let mut ancestry = Location::parent();

	let para_b_location = Location::new(1, [Parachain(2)]);

	let para_b_balances = Location::new(1, [Parachain(2), PalletInstance(1u8)]);

	ParaA::execute_with(|| {
		// Root can set transact info
		assert_ok!(XcmTransactor::set_transact_info(
			parachain::RuntimeOrigin::root(),
			// ParaB
			Box::new(xcm::VersionedLocation::from(para_b_location.clone())),
			// Para charges 1000 for every instruction, and we have 3, so 3
			3.into(),
			20000000000.into(),
			// 4 instructions in transact through signed
			Some(4.into())
		));
		// Root can set transact info
		assert_ok!(XcmTransactor::set_fee_per_second(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::from(para_b_balances.clone())),
			parachain::ParaTokensPerSecond::get(),
		));
		ancestry = parachain::UniversalLocation::get().into();
	});

	// Let's construct the Junction that we will append with DescendOrigin
	let signed_origin: Junctions = [AccountKey20 {
		network: None,
		key: PARAALICE,
	}]
	.into();

	let mut descend_origin_multilocation = parachain::SelfLocation::get();
	descend_origin_multilocation
		.append_with(signed_origin)
		.unwrap();

	// To convert it to what the paraB will see instead of us
	descend_origin_multilocation
		.reanchor(&para_b_location, &ancestry.interior)
		.unwrap();

	let derived = xcm_builder::HashedDescription::<
		parachain::AccountId,
		xcm_builder::DescribeFamily<xcm_builder::DescribeAllTerminal>,
	>::convert_location(&descend_origin_multilocation)
	.unwrap();

	let transfer_recipient = evm_account();
	let mut transfer_recipient_balance_before = 0;
	ParaB::execute_with(|| {
		assert_ok!(ParaBalances::transfer_allow_death(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			derived.clone(),
			4000000104u128,
		));
		// derived account has all funds
		assert!(ParaBalances::free_balance(&derived) == 4000000104);
		// sovereign account has 0 funds
		assert!(ParaBalances::free_balance(&para_a_account_20()) == 0);

		transfer_recipient_balance_before = ParaBalances::free_balance(&transfer_recipient.into());

		// Add proxy ALICE  -> derived
		let _ = parachain::Proxy::add_proxy_delegate(
			&PARAALICE.into(),
			derived,
			parachain::ProxyType::Any,
			0,
		);
	});

	// Encode the call. Balances transact to para_a_account
	// First index
	let mut encoded: Vec<u8> = Vec::new();
	let index =
		<parachain::Runtime as frame_system::Config>::PalletInfo::index::<parachain::EthereumXcm>()
			.unwrap() as u8;

	encoded.push(index);

	use sp_core::U256;
	// Let's do a EVM transfer
	let eth_tx =
		xcm_primitives::EthereumXcmTransaction::V2(xcm_primitives::EthereumXcmTransactionV2 {
			gas_limit: U256::from(21000),
			action: pallet_ethereum::TransactionAction::Call(transfer_recipient.into()),
			value: U256::from(100),
			input: BoundedVec::<
				u8,
				ConstU32<{ xcm_primitives::MAX_ETHEREUM_XCM_INPUT_SIZE }>
			>::try_from(vec![]).unwrap(),
			access_list: None,
		});

	// Then call bytes
	let mut call_bytes = pallet_ethereum_xcm::Call::<parachain::Runtime>::transact_through_proxy {
		transact_as: PARAALICE.into(),
		xcm_transaction: eth_tx,
	}
	.encode();
	encoded.append(&mut call_bytes);

	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_signed(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(xcm::VersionedLocation::from(para_b_location)),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					para_b_balances
				))),
				fee_amount: None
			},
			encoded,
			TransactWeights {
				transact_required_weight_at_most: 4000000000.into(),
				overall_weight: None
			},
			false
		));
	});

	ParaB::execute_with(|| {
		// Make sure the EVM transfer was executed
		assert!(
			ParaBalances::free_balance(&transfer_recipient.into())
				== transfer_recipient_balance_before + 100
		);
	});
}

#[test]
fn hrmp_init_accept_through_root() {
	MockNet::reset();

	Relay::execute_with(|| {
		assert_ok!(RelayBalances::transfer_allow_death(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			para_a_account(),
			1000u128
		));
		assert_ok!(RelayBalances::transfer_allow_death(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			para_b_account(),
			1000u128
		));
	});

	ParaA::execute_with(|| {
		let total_fee = 1_000u128;
		let total_weight: u64 = 1_000_000_000;
		let tx_weight: u64 = 500_000_000;
		// Root can send hrmp init channel
		assert_ok!(XcmTransactor::hrmp_manage(
			parachain::RuntimeOrigin::root(),
			HrmpOperation::InitOpen(HrmpInitParams {
				para_id: 2u32.into(),
				proposed_max_capacity: 1,
				proposed_max_message_size: 1
			}),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					Location::parent()
				))),
				fee_amount: Some(total_fee)
			},
			TransactWeights {
				transact_required_weight_at_most: tx_weight.into(),
				overall_weight: Some(Limited(total_weight.into()))
			}
		));
	});
	Relay::execute_with(|| {
		let expected_event: relay_chain::RuntimeEvent =
			polkadot_runtime_parachains::hrmp::Event::OpenChannelRequested {
				sender: 1u32.into(),
				recipient: 2u32.into(),
				proposed_max_capacity: 1u32,
				proposed_max_message_size: 1u32,
			}
			.into();
		assert!(relay_chain::relay_events().contains(&expected_event));
	});
	ParaB::execute_with(|| {
		let total_fee = 1_000u128;
		let total_weight: u64 = 1_000_000_000;
		let tx_weight: u64 = 500_000_000;
		// Root can send hrmp accept channel
		assert_ok!(XcmTransactor::hrmp_manage(
			parachain::RuntimeOrigin::root(),
			HrmpOperation::Accept {
				para_id: 1u32.into()
			},
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					Location::parent()
				))),
				fee_amount: Some(total_fee)
			},
			TransactWeights {
				transact_required_weight_at_most: tx_weight.into(),
				overall_weight: Some(Limited(total_weight.into()))
			}
		));
	});

	Relay::execute_with(|| {
		let expected_event: relay_chain::RuntimeEvent =
			polkadot_runtime_parachains::hrmp::Event::OpenChannelAccepted {
				sender: 1u32.into(),
				recipient: 2u32.into(),
			}
			.into();
		assert!(relay_chain::relay_events().contains(&expected_event));
	});
}

#[test]
fn hrmp_close_works() {
	MockNet::reset();

	Relay::execute_with(|| {
		assert_ok!(RelayBalances::transfer_allow_death(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			para_a_account(),
			1000u128
		));
		assert_ok!(Hrmp::force_open_hrmp_channel(
			relay_chain::RuntimeOrigin::root(),
			1u32.into(),
			2u32.into(),
			1u32,
			1u32
		));
		assert_ok!(Hrmp::force_process_hrmp_open(
			relay_chain::RuntimeOrigin::root(),
			1u32
		));
	});

	ParaA::execute_with(|| {
		let total_fee = 1_000u128;
		let total_weight: u64 = 1_000_000_000;
		let tx_weight: u64 = 500_000_000;
		assert_ok!(XcmTransactor::hrmp_manage(
			parachain::RuntimeOrigin::root(),
			HrmpOperation::Close(HrmpChannelId {
				sender: 1u32.into(),
				recipient: 2u32.into()
			}),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					Location::parent()
				))),
				fee_amount: Some(total_fee)
			},
			TransactWeights {
				transact_required_weight_at_most: tx_weight.into(),
				overall_weight: Some(Limited(total_weight.into()))
			}
		));
	});
	Relay::execute_with(|| {
		let expected_event: relay_chain::RuntimeEvent =
			polkadot_runtime_parachains::hrmp::Event::ChannelClosed {
				by_parachain: 1u32.into(),
				channel_id: HrmpChannelId {
					sender: 1u32.into(),
					recipient: 2u32.into(),
				},
			}
			.into();
		assert!(relay_chain::relay_events().contains(&expected_event));
	});
}

use parity_scale_codec::{Decode, Encode};
use sp_io::hashing::blake2_256;

// Helper to derive accountIds
pub fn derivative_account_id(who: sp_runtime::AccountId32, index: u16) -> sp_runtime::AccountId32 {
	let entropy = (b"modlpy/utilisuba", who, index).using_encoded(blake2_256);
	sp_runtime::AccountId32::decode(&mut &entropy[..]).expect("valid account id")
}

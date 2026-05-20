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

//! Unit testing
use frame_support::traits::{Contains, ContainsPair};
use frame_support::{assert_noop, assert_ok};
use sp_core::H160;
use sp_io::TestExternalities;
use sp_runtime::{BoundedVec, BuildStorage};
use xcm::latest::{Asset, Junction, Junctions, Location};

use crate::mock::{
	Erc20XcmBridge, Erc20XcmBridgeTransferGasLimit, RuntimeEvent, RuntimeOrigin, System, Test,
};
use crate::{Event, IsTeleportableErc20, TeleportableErc20s};

fn new_test_ext() -> TestExternalities {
	let storage = frame_system::GenesisConfig::<Test>::default()
		.build_storage()
		.expect("genesis storage builds");
	let mut ext = TestExternalities::new(storage);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

fn erc20_asset(contract: [u8; 20], amount: u128) -> Asset {
	let location = Location {
		parents: 0,
		interior: Junctions::from([
			Junction::PalletInstance(42u8),
			Junction::AccountKey20 {
				key: contract,
				network: None,
			},
		]),
	};
	Asset::from((location, amount))
}

#[test]
fn add_teleportable_erc20_root_works_and_emits_event() {
	new_test_ext().execute_with(|| {
		let contract = H160([1; 20]);
		assert!(!TeleportableErc20s::<Test>::contains_key(&contract));
		assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));
		assert!(TeleportableErc20s::<Test>::contains_key(&contract));
		System::assert_has_event(RuntimeEvent::Erc20XcmBridge(
			Event::TeleportableErc20Added { contract },
		));
	});
}

#[test]
fn add_teleportable_erc20_requires_root() {
	new_test_ext().execute_with(|| {
		let contract = H160([2; 20]);
		assert_noop!(
			Erc20XcmBridge::add_teleportable_erc20(RuntimeOrigin::none(), contract),
			sp_runtime::DispatchError::BadOrigin
		);
		assert!(!TeleportableErc20s::<Test>::contains_key(&contract));
	});
}

#[test]
fn add_teleportable_erc20_rejects_duplicate() {
	new_test_ext().execute_with(|| {
		let contract = H160([3; 20]);
		assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));
		assert_noop!(
			Erc20XcmBridge::add_teleportable_erc20(RuntimeOrigin::root(), contract),
			crate::Error::<Test>::Erc20AlreadyTeleportable
		);
	});
}

#[test]
fn remove_teleportable_erc20_works_and_emits_event() {
	new_test_ext().execute_with(|| {
		let contract = H160([4; 20]);
		assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));
		assert_ok!(Erc20XcmBridge::remove_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));
		assert!(!TeleportableErc20s::<Test>::contains_key(&contract));
		System::assert_has_event(RuntimeEvent::Erc20XcmBridge(
			Event::TeleportableErc20Removed { contract },
		));
	});
}

#[test]
fn remove_teleportable_erc20_rejects_unknown() {
	new_test_ext().execute_with(|| {
		let contract = H160([5; 20]);
		assert_noop!(
			Erc20XcmBridge::remove_teleportable_erc20(RuntimeOrigin::root(), contract),
			crate::Error::<Test>::Erc20NotTeleportable
		);
	});
}

#[test]
fn is_teleportable_erc20_filter_pair_only_admits_whitelisted() {
	new_test_ext().execute_with(|| {
		let contract = H160([6; 20]);
		let asset = erc20_asset(contract.0, 100);
		let dest = Location::new(1, [Junction::Parachain(1001)]);

		// Not yet whitelisted: ContainsPair must reject.
		assert!(!<IsTeleportableErc20<Test> as ContainsPair<
			Asset,
			Location,
		>>::contains(&asset, &dest,));

		assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));

		// Whitelisted: ContainsPair admits the asset for any destination.
		assert!(<IsTeleportableErc20<Test> as ContainsPair<
			Asset,
			Location,
		>>::contains(&asset, &dest,));

		// Non-ERC-20 asset (e.g. native) is never admitted.
		let native = Asset::from((Location::parent(), 1u128));
		assert!(!<IsTeleportableErc20<Test> as ContainsPair<
			Asset,
			Location,
		>>::contains(&native, &dest,));
	});
}

#[test]
fn is_teleportable_erc20_filter_extrinsic_requires_all_whitelisted() {
	new_test_ext().execute_with(|| {
		let whitelisted = H160([7; 20]);
		let non_whitelisted = H160([8; 20]);
		let dest = Location::new(1, [Junction::Parachain(1001)]);

		assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
			RuntimeOrigin::root(),
			whitelisted,
		));

		// Empty asset list: rejected.
		let empty = (dest.clone(), Vec::<Asset>::new());
		assert!(!<IsTeleportableErc20<Test> as Contains<(
			Location,
			Vec<Asset>,
		)>>::contains(&empty));

		// All whitelisted: admitted.
		let all_ok = (
			dest.clone(),
			vec![erc20_asset(whitelisted.0, 1), erc20_asset(whitelisted.0, 2)],
		);
		assert!(<IsTeleportableErc20<Test> as Contains<(
			Location,
			Vec<Asset>,
		)>>::contains(&all_ok));

		// Mixed: rejected.
		let mixed = (
			dest,
			vec![
				erc20_asset(whitelisted.0, 1),
				erc20_asset(non_whitelisted.0, 1),
			],
		);
		assert!(!<IsTeleportableErc20<Test> as Contains<(
			Location,
			Vec<Asset>,
		)>>::contains(&mixed));
	});
}

#[test]
fn general_key_data_size_32() {
	let junction: Junction = (BoundedVec::new()).into();

	// Assert that GeneralKey data length is 32 bytes
	match junction {
		Junction::GeneralKey { length: _, data } => {
			let _: [u8; 32] = data;
		}
		_ => assert!(false),
	}

	assert_eq!(
		Erc20XcmBridge::gas_limit_of_erc20_transfer(&junction.into()),
		Erc20XcmBridgeTransferGasLimit::get()
	)
}

#[test]
fn gas_limit_override() {
	let text = "gas_limit:".as_bytes();
	let limit = 300_000u64;
	let data = [text, &limit.to_le_bytes()].concat();
	let vec = BoundedVec::try_from(data).expect("vec should convert");
	let junction: Junction = (vec).into();
	assert_eq!(
		Erc20XcmBridge::gas_limit_of_erc20_transfer(&junction.into()),
		limit
	)
}

#[test]
fn gas_limit_override_typo() {
	let text = "gaslimit:".as_bytes();
	let limit = 300_000u64;
	let data = [text, &limit.to_le_bytes()].concat();
	let vec = BoundedVec::try_from(data).expect("vec should convert");
	let junction: Junction = (vec).into();
	assert_eq!(
		Erc20XcmBridge::gas_limit_of_erc20_transfer(&junction.into()),
		Erc20XcmBridgeTransferGasLimit::get()
	)
}

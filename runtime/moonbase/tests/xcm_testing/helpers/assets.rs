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

//! Asset management helpers for XCM tests

use crate::xcm_mock::{parachain, AssetManager, ParaA};
use crate::xcm_testing::add_supported_asset;
use frame_support::assert_ok;
use xcm_simulator::TestExt;

pub struct AssetBuilder {
	location: xcm::v3::Location,
	is_sufficient: bool,
}

impl AssetBuilder {
	pub fn relay_asset() -> Self {
		Self {
			location: xcm::v3::Location::parent(),
			is_sufficient: true,
		}
	}

	pub fn with_sufficient(mut self, is_sufficient: bool) -> Self {
		self.is_sufficient = is_sufficient;
		self
	}

	pub fn register(self) -> parachain::AssetId {
		let source_location = parachain::AssetType::Xcm(self.location.clone());
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
				self.is_sufficient
			));
			assert_ok!(add_supported_asset(source_location.clone(), 0));
		});

		source_location.into()
	}

	pub fn register_with_units_per_second(self, units_per_second: u128) -> parachain::AssetId {
		let source_location = parachain::AssetType::Xcm(self.location.clone());
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
				self.is_sufficient
			));
			assert_ok!(add_supported_asset(
				source_location.clone(),
				units_per_second
			));
		});

		source_location.into()
	}
}

// Convenience functions for common asset types
pub fn register_relay_asset() -> parachain::AssetId {
	AssetBuilder::relay_asset().register()
}

pub fn register_relay_asset_with_units_per_second(units_per_second: u128) -> parachain::AssetId {
	AssetBuilder::relay_asset().register_with_units_per_second(units_per_second)
}

pub fn register_relay_asset_non_sufficient() -> parachain::AssetId {
	AssetBuilder::relay_asset()
		.with_sufficient(false)
		.register()
}

pub fn register_relay_asset_in_para_b() -> parachain::AssetId {
	let source_location = parachain::AssetType::Xcm(xcm::v3::Location::parent());
	let asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};

	use crate::xcm_mock::ParaB;
	ParaB::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata,
			1u128,
			true
		));
		assert_ok!(add_supported_asset(source_location.clone(), 0));
	});

	source_location.into()
}

// Helper for the common relay asset setup pattern in statemint tests
pub fn setup_relay_asset_for_statemint() -> (parachain::AssetType, parachain::AssetId) {
	let relay_location = parachain::AssetType::Xcm(xcm::v3::Location::parent());
	let source_relay_id: parachain::AssetId = relay_location.clone().into();

	let relay_asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};

	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			relay_location.clone(),
			relay_asset_metadata,
			1u128,
			true
		));
		assert_ok!(add_supported_asset(relay_location.clone(), 0));
	});

	(relay_location, source_relay_id)
}

// Copyright 2025 Moonbeam Foundation.Inc.
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

//! # Moonbeam specific Migrations

use crate::bridge_config::XcmOverKusamaInstance;
use crate::{BridgeKusamaMessages, PolkadotXcm, Runtime, RuntimeOrigin};
use bp_messages::MessagesOperatingMode;
use bp_runtime::AccountIdOf;
use frame_support::instances::Instance1;
use frame_support::traits::{ConstBool, OnRuntimeUpgrade};
use moonbeam_core_primitives::AccountId;
use pallet_migrations::{GetMigrations, Migration};
use pallet_xcm_bridge::ThisChainOf;
use sp_std::{prelude::*, vec};
use sp_weights::Weight;

use frame_support::parameter_types;

use xcm::prelude::{InteriorLocation, Location, Parachain};

parameter_types! {
	pub Lane: bp_moonbeam::LaneId = Default::default();
	pub RelativeLocation: Location = Location::new(
		1,
		[
			Parachain(bp_moonbeam::PARACHAIN_ID)
		],
	);
	pub BridgedUniversalLocation: InteriorLocation = bp_moonriver::GlobalConsensusLocation::get().interior;
}

pub struct SetupBridge;
impl Migration for SetupBridge
where
	Runtime: frame_system::Config<AccountId = AccountIdOf<ThisChainOf<Runtime, Instance1>>>
		+ pallet_xcm_bridge::Config<Instance1>
		+ pallet_xcm_bridge::Config<Instance1>,
	AccountIdOf<ThisChainOf<Runtime, Instance1>>: From<AccountId>,
{
	fn friendly_name(&self) -> &str {
		"MM_SetupBridge"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		let mut weight = <Runtime as frame_system::Config>::DbWeight::get().writes(1);
		let _ = PolkadotXcm::force_xcm_version(
			RuntimeOrigin::root(),
			Box::new(bp_moonriver::GlobalConsensusLocation::get()),
			xcm::v5::VERSION,
		)
		.map_err(|err| {
			log::error!("Failed to set xcm version: {:?}", err);
		});

		weight =
			weight.saturating_add(<Runtime as frame_system::Config>::DbWeight::get().writes(1));
		let _ = BridgeKusamaMessages::set_operating_mode(
			RuntimeOrigin::root(),
			MessagesOperatingMode::RejectingOutboundMessages,
		)
		.map_err(|err| {
			log::error!("Failed to set operating mode: {:?}", err);
		});

		weight = weight.saturating_add(pallet_xcm_bridge::migration::OpenBridgeForLane::<
			Runtime,
			XcmOverKusamaInstance,
			Lane,
			ConstBool<true>,
			RelativeLocation,
			BridgedUniversalLocation,
		>::on_runtime_upgrade());

		weight
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<Vec<u8>, sp_runtime::DispatchError> {
		Ok(Default::default())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self, state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		Ok(())
	}
}

pub struct MoonbeamMigrations;

impl GetMigrations for MoonbeamMigrations {
	fn get_migrations() -> Vec<Box<dyn Migration>> {
		vec![Box::new(SetupBridge)]
	}
}

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

//! Generic transfer helpers for XCM tests

use crate::xcm_mock::{parachain, relay_chain, ParaA, Relay, RelayChainPalletXcm, RELAYALICE};
use frame_support::assert_ok;
use sp_std::boxed::Box;
use xcm::latest::prelude::{Location, Parachain, WeightLimit};
use xcm_simulator::TestExt;

use super::core::{account_key20_location, para_to_para_location, parachain_location};
use super::weights::standard_transfer_weight;

// Generic transfer execution helpers

pub fn execute_transfer_to_para(
	from_account: [u8; 20],
	asset: xcm::VersionedAssets,
	dest_para: u32,
	dest_account: [u8; 20],
	weight: Option<WeightLimit>,
) {
	let dest = para_to_para_location(dest_para, dest_account);
	let (chain_part, beneficiary) =
		xcm_primitives::split_location_into_chain_part_and_beneficiary(dest).unwrap();
	let weight_limit = weight.unwrap_or(standard_transfer_weight());

	ParaA::execute_with(|| {
		assert_ok!(crate::xcm_mock::parachain::PolkadotXcm::transfer_assets(
			parachain::RuntimeOrigin::signed(parachain::AccountId::from(from_account)),
			Box::new(xcm::VersionedLocation::from(chain_part)),
			Box::new(xcm::VersionedLocation::from(beneficiary)),
			Box::new(asset),
			0,
			weight_limit
		));
	});
}

// Relay to Statemint transfer helper
pub fn execute_relay_to_statemint_transfer(beneficiary: Location, amount: u128) {
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::limited_teleport_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1000).into()),
			Box::new(xcm::VersionedLocation::from(beneficiary).into()),
			Box::new(([], amount).into()),
			0,
			WeightLimit::Unlimited
		));
	});
}

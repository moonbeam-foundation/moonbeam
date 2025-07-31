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

//! Transactor and call encoding helpers for XCM tests

use crate::xcm_mock::{parachain, ParaA, XcmTransactor};
use frame_support::{assert_ok, weights::constants::WEIGHT_REF_TIME_PER_SECOND};
use sp_std::boxed::Box;
use xcm::latest::prelude::Location;
use xcm_simulator::TestExt;

// Transactor setup helpers

pub fn setup_relay_transactor_config() {
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
		// Root can set fee per second
		assert_ok!(XcmTransactor::set_fee_per_second(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::from(Location::parent())),
			WEIGHT_REF_TIME_PER_SECOND as u128,
		));
	});
}

// Call encoding helpers for relay chain transactions

pub fn encode_relay_balance_transfer_call(
	dest: crate::xcm_mock::relay_chain::AccountId,
	amount: u128,
) -> Vec<u8> {
	use frame_support::traits::PalletInfo;
	use parity_scale_codec::Encode;

	let mut encoded: Vec<u8> = Vec::new();
	let index =
		<crate::xcm_mock::relay_chain::Runtime as frame_system::Config>::PalletInfo::index::<
			crate::xcm_mock::relay_chain::Balances,
		>()
		.unwrap() as u8;

	encoded.push(index);

	let mut call_bytes =
		pallet_balances::Call::<crate::xcm_mock::relay_chain::Runtime>::transfer_allow_death {
			dest,
			value: amount.into(),
		}
		.encode();

	encoded.append(&mut call_bytes);
	encoded
}

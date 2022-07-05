// Copyright 2019-2022 PureStake Inc.
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

//! Randomness precompile unit tests

use std::collections::HashSet;

use precompile_utils::solidity;

use crate::Action;
// use crate::mock::*;

#[test]
fn test_solidity_interface_has_all_implemented_selectors() {
	let selectors = solidity::get_selectors("Randomness.sol")
		.keys()
		.cloned()
		.collect::<HashSet<_>>();

	assert_eq!(Action::RelayBlockNumber as u32, 0xedfec347);
	assert!(selectors.contains(&(Action::RelayBlockNumber as u32)));

	assert_eq!(Action::RelayEpochIndex as u32, 0x81797566);
	assert!(selectors.contains(&(Action::RelayEpochIndex as u32)));

	assert_eq!(Action::RequestBabeRandomnessCurrentBlock as u32, 0xc4921133);
	assert!(selectors.contains(&(Action::RequestBabeRandomnessCurrentBlock as u32)));

	assert_eq!(Action::RequestBabeRandomnessOneEpochAgo as u32, 0xbbc9e95f);
	assert!(selectors.contains(&(Action::RequestBabeRandomnessOneEpochAgo as u32)));

	assert_eq!(Action::RequestBabeRandomnessTwoEpochsAgo as u32, 0x25b14a0b);
	assert!(selectors.contains(&(Action::RequestBabeRandomnessTwoEpochsAgo as u32)));

	assert_eq!(Action::RequestLocalRandomness as u32, 0xb4a11763);
	assert!(selectors.contains(&(Action::RequestLocalRandomness as u32)));

	assert_eq!(Action::FulfillRequest as u32, 0xb9904a86);
	assert!(selectors.contains(&(Action::FulfillRequest as u32)));

	assert_eq!(Action::IncreaseRequestFee as u32, 0x6a5b3380);
	assert!(selectors.contains(&(Action::IncreaseRequestFee as u32)));

	assert_eq!(Action::ExecuteRequestExpiration as u32, 0x8fcdcc49);
	assert!(selectors.contains(&(Action::ExecuteRequestExpiration as u32)));
}

#[test]
fn test_solidity_interface_has_all_selectors_implemented() {
	for (selector, fn_name) in solidity::get_selectors("Randomness.sol") {
		if Action::try_from(selector).is_err() {
			panic!(
				"failed decoding selector 0x{:x} => '{}' as Action",
				selector, fn_name
			)
		}
	}
}

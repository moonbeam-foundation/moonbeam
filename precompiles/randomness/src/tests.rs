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

// use std::collections::HashSet;

// use precompile_utils::solidity;

// use crate::Action;
// use crate::mock::*;

// #[test]
// fn test_all_actions_are_implemented_in_solidity_interface() {
// 	let selectors = solidity::get_selectors("Randomness.sol")
// 		.into_iter()
// 		.map(|sf| sf.compute_selector())
// 		.collect::<HashSet<_>>();

// 	assert_eq!(Action::RelayEpochIndex as u32, 0x81797566);
// 	assert!(selectors.contains(&(Action::RelayEpochIndex as u32)));

// 	assert_eq!(Action::RequestBabeRandomness as u32, 0xbbc9e95f);
// 	assert!(selectors.contains(&(Action::RequestBabeRandomness as u32)));

// 	assert_eq!(Action::RequestLocalRandomness as u32, 0xb4a11763);
// 	assert!(selectors.contains(&(Action::RequestLocalRandomness as u32)));

// 	assert_eq!(Action::FulfillRequest as u32, 0xb9904a86);
// 	assert!(selectors.contains(&(Action::FulfillRequest as u32)));

// 	assert_eq!(Action::IncreaseRequestFee as u32, 0x6a5b3380);
// 	assert!(selectors.contains(&(Action::IncreaseRequestFee as u32)));

// 	assert_eq!(Action::ExecuteRequestExpiration as u32, 0x8fcdcc49);
// 	assert!(selectors.contains(&(Action::ExecuteRequestExpiration as u32)));
// }

// #[test]
// fn test_solidity_interface_has_all_function_selectors_documented_and_implemented() {
// 	for solidity_fn in solidity::get_selectors("Randomness.sol") {
// 		assert_eq!(
// 			solidity_fn.compute_selector_hex(),
// 			solidity_fn.docs_selector,
// 			"documented selector for '{}' did not match",
// 			solidity_fn.signature()
// 		);

// 		let selector = solidity_fn.compute_selector();
// 		if Action::try_from(selector).is_err() {
// 			panic!(
// 				"failed decoding selector 0x{:x} => '{}' as Action",
// 				selector,
// 				solidity_fn.signature()
// 			)
// 		}
// 	}
// }

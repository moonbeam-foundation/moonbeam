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

use frame_support::weights::{constants::WEIGHT_PER_SECOND, Weight};
use pallet_randomness::weights::WeightInfo;

pub const REQUEST_RANDOMNESS_ESTIMATED_COST: u64 = 26342;
pub const FULFILLMENT_OVERHEAD_ESTIMATED_COST: u64 = 23461;
pub const INCREASE_REQUEST_FEE_ESTIMATED_COST: u64 = 16737;
pub const EXECUTE_EXPIRATION_ESTIMATED_COST: u64 = 21993;

// TODO: move tests to moonbase runtime to verify constants

/// Current approximation of the gas/s consumption considering
/// EVM execution over compiled WASM (on 4.4Ghz CPU).
/// Given the 500ms Weight, from which 75% only are used for transactions,
/// the total EVM execution gas limit is: GAS_PER_SECOND * 0.500 * 0.75 ~= 15_000_000.
pub const GAS_PER_SECOND: u64 = 40_000_000;

/// Approximate ratio of the amount of Weight per Gas.
/// u64 works for approximations because Weight is a very small unit compared to gas.
pub const WEIGHT_PER_GAS: u64 = WEIGHT_PER_SECOND / GAS_PER_SECOND;

fn moonbase_weight_to_gas(weight: Weight) -> u64 {
	u64::try_from(weight.wrapping_div(WEIGHT_PER_GAS)).unwrap_or(u32::MAX as u64)
}

fn request_randomness() {
	let weight = <()>::request_randomness();
	let constant_weight = moonbase_weight_to_gas(weight);
	println!("{:?}", constant_weight);
}

fn fulfillment() {
	let weight = <()>::prepare_fulfillment() + <()>::finish_fulfillment();
	let constant_weight = moonbase_weight_to_gas(weight);
	println!("{:?}", constant_weight);
}

fn increase_fee() {
	let weight = <()>::increase_fee();
	let constant_weight = moonbase_weight_to_gas(weight);
	println!("{:?}", constant_weight);
}

fn execute_expiration() {
	let weight = <()>::execute_request_expiration();
	let constant_weight = moonbase_weight_to_gas(weight);
	println!("{:?}", constant_weight);
}

// Uncomment to see generated weights
// #[test]
// fn test() {
// 	request_randomness();
// 	fulfillment();
// 	increase_fee();
// 	execute_expiration();
// 	assert!(false);
// }

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

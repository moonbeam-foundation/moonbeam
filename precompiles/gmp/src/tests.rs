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

use crate::mock::PCall;
use precompile_utils::testing::*;

#[test]
fn test_sample_wormhole_vm_output() {
	use crate::*;

	let output = vec![
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 100, 11, 102, 93, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 89, 156, 234, 34,
		4, 180, 250, 236, 213, 132, 171, 31, 43, 106, 202, 19, 122, 10, 251, 232, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 158, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 96,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 2, 224, 71, 230, 3, 116, 190, 146, 26, 173, 35, 232, 145, 148, 172, 124, 85, 42, 199,
		161, 167, 88, 112, 54, 68, 112, 162, 117, 51, 133, 149, 124, 112, 227, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 69, 3, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 49, 45, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 241, 39, 125, 30, 216, 173, 70, 107, 237, 223, 146, 239, 68,
		138, 19, 38, 97, 149, 102, 33, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 21, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 183,
		232, 195, 86, 9, 202, 115, 39, 123, 34, 7, 208, 123, 81, 201, 172, 87, 152, 179, 128, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 32,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 128, 123, 32, 34, 112, 97, 114, 101, 110, 116, 115, 34, 58, 32, 49, 44, 32, 34, 105,
		110, 116, 101, 114, 105, 111, 114, 34, 58, 32, 123, 32, 34, 88, 50, 34, 58, 32, 91, 32,
		123, 32, 34, 80, 97, 114, 97, 99, 104, 97, 105, 110, 34, 58, 32, 56, 56, 56, 32, 125, 44,
		32, 123, 32, 34, 65, 99, 99, 111, 117, 110, 116, 75, 101, 121, 50, 48, 34, 58, 32, 34, 48,
		120, 51, 53, 52, 66, 49, 48, 68, 52, 55, 101, 56, 52, 65, 48, 48, 54, 98, 57, 69, 55, 101,
		54, 54, 65, 50, 50, 57, 68, 49, 55, 52, 69, 56, 70, 70, 50, 65, 48, 54, 51, 34, 32, 125,
		32, 93, 125, 125, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 1, 217, 139, 6, 132, 1, 124, 255, 161, 165, 111, 119, 243, 64, 141, 87, 8, 9,
		196, 64, 181, 51, 184, 195, 239, 43, 37, 155, 68, 203, 249, 49, 5, 53, 45, 17, 171, 134,
		46, 124, 51, 254, 196, 229, 77, 62, 241, 13, 92, 137, 40, 197, 195, 179, 222, 21, 188, 177,
		213, 217, 45, 182, 106, 146, 249, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 28, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
	];

	let mut reader = EvmDataReader::new(&output[..]);
	let vm: WormholeVM = reader.read().expect("Failed to read WormholeVM");

	// TODO: assertions about struct contents (or remove test)

	/*
	let as_bytes: Vec<u8> = vm.payload.into();
	let as_hex = hex::encode(&as_bytes[..]);
	panic!("payload: {}", as_hex);
	*/
}

#[test]
fn test_solidity_interface_has_all_function_selectors_documented_and_implemented() {
	for file in ["Gmp.sol"] {
		// assert_eq!(solidity::get_selectors(file).len(), 2);
		for solidity_fn in solidity::get_selectors(file) {
			assert_eq!(
				solidity_fn.compute_selector_hex(),
				solidity_fn.docs_selector,
				"documented selector for '{}' did not match for file '{}'",
				solidity_fn.signature(),
				file,
			);

			let selector = solidity_fn.compute_selector();
			if !PCall::supports_selector(selector) {
				panic!(
					"failed decoding selector 0x{:x} => '{}' as Action for file '{}'",
					selector,
					solidity_fn.signature(),
					file,
				)
			}
		}
	}
}

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

use crate::{mock::PCall, types::XcmRoutingUserAction};
use precompile_utils::testing::*;
use xcm::latest::MultiLocation;

#[test]
fn test_sample_wormhole_vm_output() {
	use crate::*;

	let output =
		"00000000000000000000000000000000000000000000000000000000000000200000000000000000 \
	  0000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000 \
	  0000000000000640b665d0000000000000000000000000000000000000000000000000000000000000001000000 \
	  000000000000000000000000000000000000000000000000000000000a000000000000000000000000599cea220 \
	  4b4faecd584ab1f2b6aca137a0afbe8000000000000000000000000000000000000000000000000000000000000 \
	  009e000000000000000000000000000000000000000000000000000000000000000100000000000000000000000 \
	  0000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000 \
	  0000000000000000000000000000000000000000000000000000000000000000000000000002e047e60374be921 \
	  aad23e89194ac7c552ac7a1a75870364470a2753385957c70e30000000000000000000000000000000000000000 \
	  000000000000000000000145030000000000000000000000000000000000000000000000000000000001312d000 \
	  00000000000000000000000f1277d1ed8ad466beddf92ef448a132661956621000a000000000000000000000000 \
	  00000000000000000000000000000000000008150010000000000000000000000000b7e8c35609ca73277b2207d \
	  07b51c9ac5798b38000000000000000000000000000000000000000000000000000000000000000200000000000 \
	  0000000000000000000000000000000000000000000000000000807b2022706172656e7473223a20312c2022696 \
	  e746572696f72223a207b20225832223a205b207b202250617261636861696e223a20383838207d2c207b202241 \
	  63636f756e744b65793230223a20223078333534423130443437653834413030366239453765363641323239443 \
	  1373445384646324130363322207d205d7d7d000000000000000000000000000000000000000000000000000000 \
	  0000000000000000000000000000000000000000000000000000000000000001d98b0684017cffa1a56f77f3408 \
	  d570809c440b533b8c3ef2b259b44cbf93105352d11ab862e7c33fec4e54d3ef10d5c8928c5c3b3de15bcb1d5d9 \
	  2db66a92f9000000000000000000000000000000000000000000000000000000000000001c00000000000000000 \
	  00000000000000000000000000000000000000000000000";
	let output = hex::decode(&output).expect("invalid VAA hex");

	let mut reader = EvmDataReader::new(&output[..]);
	let vm: WormholeVM = reader.read().expect("Failed to read WormholeVM");

	// TODO: assertions about struct contents (or remove test)

	// let as_bytes: Vec<u8> = vm.payload.into();
	// let as_hex = hex::encode(&as_bytes[..]);
	// panic!("payload: {}", as_hex);
	panic!("wormhole vm: {:?}", vm);
}

#[test]
fn test_user_action_decode() {
	use crate::VersionedUserAction;
	use parity_scale_codec::Encode;

	// TODO: remove test, just using to print value
	let action = VersionedUserAction::V1(XcmRoutingUserAction {
		destination_chain: MultiLocation::parent(),
		destination_account: MultiLocation::parent(),
	});

	let as_bytes: Vec<u8> = action.encode();
	let as_hex = hex::encode(&as_bytes[..]);
	panic!("payload: {}", as_hex);
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

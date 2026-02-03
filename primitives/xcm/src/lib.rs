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

//! The XCM primitive trait implementations

#![cfg_attr(not(feature = "std"), no_std)]

mod asset_id_conversions;
pub use asset_id_conversions::*;

mod constants;
pub use constants::*;

mod ethereum_xcm;
pub use ethereum_xcm::*;

mod filter_asset_max_fee;
pub use filter_asset_max_fee::*;

mod origin_conversion;
pub use origin_conversion::*;

mod transactor_traits;
pub use transactor_traits::*;

mod fee_trader;
pub use fee_trader::*;

use sp_std::sync::Arc;
use xcm::latest::{Junction, Junctions, Location};

/// Build Junctions from a slice of junctions
fn junctions_from_slice(junctions: &[Junction]) -> Option<Junctions> {
	match junctions.len() {
		0 => Some(Junctions::Here),
		1 => Some(Junctions::X1(Arc::new([junctions[0].clone()]))),
		2 => Some(Junctions::X2(Arc::new([
			junctions[0].clone(),
			junctions[1].clone(),
		]))),
		3 => Some(Junctions::X3(Arc::new([
			junctions[0].clone(),
			junctions[1].clone(),
			junctions[2].clone(),
		]))),
		4 => Some(Junctions::X4(Arc::new([
			junctions[0].clone(),
			junctions[1].clone(),
			junctions[2].clone(),
			junctions[3].clone(),
		]))),
		5 => Some(Junctions::X5(Arc::new([
			junctions[0].clone(),
			junctions[1].clone(),
			junctions[2].clone(),
			junctions[3].clone(),
			junctions[4].clone(),
		]))),
		6 => Some(Junctions::X6(Arc::new([
			junctions[0].clone(),
			junctions[1].clone(),
			junctions[2].clone(),
			junctions[3].clone(),
			junctions[4].clone(),
			junctions[5].clone(),
		]))),
		7 => Some(Junctions::X7(Arc::new([
			junctions[0].clone(),
			junctions[1].clone(),
			junctions[2].clone(),
			junctions[3].clone(),
			junctions[4].clone(),
			junctions[5].clone(),
			junctions[6].clone(),
		]))),
		8 => Some(Junctions::X8(Arc::new([
			junctions[0].clone(),
			junctions[1].clone(),
			junctions[2].clone(),
			junctions[3].clone(),
			junctions[4].clone(),
			junctions[5].clone(),
			junctions[6].clone(),
			junctions[7].clone(),
		]))),
		_ => None,
	}
}

pub fn split_location_into_chain_part_and_beneficiary(
	mut location: Location,
) -> Option<(Location, Location)> {
	let mut beneficiary_junctions_vec = Vec::new();

	// start popping junctions until we reach chain identifier
	while let Some(j) = location.last() {
		if matches!(j, Junction::Parachain(_) | Junction::GlobalConsensus(_)) {
			// return chain subsection
			// Reverse the vec to restore original order, then build Junctions
			beneficiary_junctions_vec.reverse();
			let beneficiary_junctions = junctions_from_slice(&beneficiary_junctions_vec)?;
			return Some((location, beneficiary_junctions.into_location()));
		} else {
			let (location_prefix, maybe_last_junction) = location.split_last_interior();
			location = location_prefix;
			if let Some(junction) = maybe_last_junction {
				beneficiary_junctions_vec.push(junction);
			}
		}
	}

	// Reverse the vec to restore original order, then build Junctions
	beneficiary_junctions_vec.reverse();
	let beneficiary_junctions = junctions_from_slice(&beneficiary_junctions_vec)?;

	if location.parent_count() == 1 {
		Some((Location::parent(), beneficiary_junctions.into_location()))
	} else {
		None
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use xcm::latest::prelude::*;

	#[test]
	fn test_split_location_single_beneficiary_junction() {
		// Test: Parachain(2) + AccountKey20
		let location = Location {
			parents: 1,
			interior: [Parachain(2), AccountKey20 { network: None, key: [1u8; 20] }].into(),
		};

		let (chain_part, beneficiary) =
			split_location_into_chain_part_and_beneficiary(location).unwrap();

		// Chain part should be Parachain(2)
		assert_eq!(
			chain_part,
			Location { parents: 1, interior: [Parachain(2)].into() }
		);

		// Beneficiary should be AccountKey20
		assert_eq!(
			beneficiary,
			Location { parents: 0, interior: [AccountKey20 { network: None, key: [1u8; 20] }].into() }
		);
	}

	#[test]
	fn test_split_location_multiple_beneficiary_junctions_order_preserved() {
		// Test: Parachain(100) + AccountId32 + GeneralIndex(42)
		// This test verifies that the order is preserved (AccountId32 comes before GeneralIndex)
		let account_id = AccountId32 { network: None, id: [2u8; 32] };
		let general_index = GeneralIndex(42);

		let location = Location {
			parents: 1,
			interior: [Parachain(100), account_id, general_index].into(),
		};

		let (chain_part, beneficiary) =
			split_location_into_chain_part_and_beneficiary(location).unwrap();

		// Chain part should be Parachain(100)
		assert_eq!(
			chain_part,
			Location { parents: 1, interior: [Parachain(100)].into() }
		);

		// Beneficiary should preserve order: AccountId32, then GeneralIndex
		assert_eq!(
			beneficiary,
			Location {
				parents: 0,
				interior: [account_id, general_index].into()
			}
		);
	}

	#[test]
	fn test_split_location_three_beneficiary_junctions_order_preserved() {
		// Test: Parachain(200) + PalletInstance(5) + AccountId32 + GeneralIndex(10)
		let pallet = PalletInstance(5);
		let account_id = AccountId32 { network: None, id: [3u8; 32] };
		let general_index = GeneralIndex(10);

		let location = Location {
			parents: 1,
			interior: [Parachain(200), pallet, account_id, general_index].into(),
		};

		let (chain_part, beneficiary) =
			split_location_into_chain_part_and_beneficiary(location).unwrap();

		// Chain part should be Parachain(200)
		assert_eq!(
			chain_part,
			Location { parents: 1, interior: [Parachain(200)].into() }
		);

		// Beneficiary should preserve order: PalletInstance, AccountId32, GeneralIndex
		assert_eq!(
			beneficiary,
			Location {
				parents: 0,
				interior: [pallet, account_id, general_index].into()
			}
		);
	}

	#[test]
	fn test_split_location_with_global_consensus() {
		// Test: GlobalConsensus(Polkadot) + Parachain(1) + AccountId32
		let account_id = AccountId32 { network: None, id: [4u8; 32] };

		let location = Location {
			parents: 1,
			interior: [
				GlobalConsensus(NetworkId::Polkadot),
				Parachain(1),
				account_id,
			]
			.into(),
		};

		let (chain_part, beneficiary) =
			split_location_into_chain_part_and_beneficiary(location).unwrap();

		// Chain part should stop at Parachain(1) (last chain identifier when processing from end)
		// Since Parachain(1) is encountered first when processing from the end, chain_part includes
		// both GlobalConsensus and Parachain(1)
		assert_eq!(
			chain_part,
			Location {
				parents: 1,
				interior: [GlobalConsensus(NetworkId::Polkadot), Parachain(1)].into()
			}
		);

		// Beneficiary should be AccountId32
		assert_eq!(
			beneficiary,
			Location { parents: 0, interior: [account_id].into() }
		);
	}

	#[test]
	fn test_split_location_parent_only() {
		// Test: Parent + AccountId32
		let account_id = AccountId32 { network: None, id: [5u8; 32] };

		let location = Location {
			parents: 1,
			interior: [account_id].into(),
		};

		let (chain_part, beneficiary) =
			split_location_into_chain_part_and_beneficiary(location).unwrap();

		// Chain part should be parent
		assert_eq!(chain_part, Location::parent());

		// Beneficiary should be AccountId32
		assert_eq!(
			beneficiary,
			Location { parents: 0, interior: [account_id].into() }
		);
	}

	#[test]
	fn test_split_location_multiple_junctions_order_verification() {
		// Test with multiple junctions to verify order is NOT reversed
		// Original: [Parachain(300), JunctionA, JunctionB, JunctionC]
		// Expected beneficiary: [JunctionA, JunctionB, JunctionC] (same order)
		let junction_a = AccountKey20 { network: None, key: [10u8; 20] };
		let junction_b = AccountId32 { network: None, id: [20u8; 32] };
		let junction_c = GeneralIndex(30);

		let location = Location {
			parents: 1,
			interior: [Parachain(300), junction_a, junction_b, junction_c].into(),
		};

		let (chain_part, beneficiary) =
			split_location_into_chain_part_and_beneficiary(location).unwrap();

		// Verify chain part
		assert_eq!(
			chain_part,
			Location { parents: 1, interior: [Parachain(300)].into() }
		);

		// Verify beneficiary order is preserved (A, B, C - not reversed)
		let beneficiary_interior = beneficiary.interior;
		match beneficiary_interior {
			Junctions::X3(junctions) => {
				assert_eq!(junctions[0], Junction::AccountKey20 { network: None, key: [10u8; 20] });
				assert_eq!(junctions[1], Junction::AccountId32 { network: None, id: [20u8; 32] });
				assert_eq!(junctions[2], Junction::GeneralIndex(30));
			},
			_ => panic!("Expected X3 junctions"),
		}
	}

	#[test]
	fn test_split_location_no_beneficiary() {
		// Test: Only Parachain (no beneficiary junctions)
		let location = Location {
			parents: 1,
			interior: [Parachain(400)].into(),
		};

		let (chain_part, beneficiary) =
			split_location_into_chain_part_and_beneficiary(location).unwrap();

		// Chain part should be Parachain(400)
		assert_eq!(
			chain_part,
			Location { parents: 1, interior: [Parachain(400)].into() }
		);

		// Beneficiary should be Here (empty)
		assert_eq!(beneficiary, Location { parents: 0, interior: Junctions::Here });
	}

	#[test]
	fn test_split_location_invalid_no_chain_identifier() {
		// Test: Only beneficiary junctions, no chain identifier (should return None)
		let location = Location {
			parents: 0,
			interior: [AccountId32 { network: None, id: [6u8; 32] }].into(),
		};

		let result = split_location_into_chain_part_and_beneficiary(location);
		assert!(result.is_none());
	}

	#[test]
	fn test_split_location_invalid_wrong_parent_count() {
		// Test: Wrong parent count (not 1)
		let location = Location {
			parents: 2,
			interior: [AccountId32 { network: None, id: [7u8; 32] }].into(),
		};

		let result = split_location_into_chain_part_and_beneficiary(location);
		assert!(result.is_none());
	}
}

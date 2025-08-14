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

pub mod get_by_key;
pub use get_by_key::*;

mod ethereum_xcm;
pub use ethereum_xcm::*;

mod filter_asset_max_fee;
pub use filter_asset_max_fee::*;

mod origin_conversion;
pub use origin_conversion::*;

mod transactor_traits;
pub use transactor_traits::*;

use xcm::latest::{Junction, Junctions, Location};

pub fn split_location_into_chain_part_and_beneficiary(
	mut location: Location,
) -> Option<(Location, Location)> {
	let mut beneficiary_junctions = Junctions::Here;

	// start popping junctions until we reach chain identifier
	while let Some(j) = location.last() {
		if matches!(j, Junction::Parachain(_) | Junction::GlobalConsensus(_)) {
			// return chain subsection
			return Some((location, beneficiary_junctions.into_location()));
		} else {
			let (location_prefix, maybe_last_junction) = location.split_last_interior();
			location = location_prefix;
			if let Some(junction) = maybe_last_junction {
				beneficiary_junctions.push(junction).ok()?;
			}
		}
	}

	if location.parent_count() == 1 {
		Some((Location::parent(), beneficiary_junctions.into_location()))
	} else {
		None
	}
}

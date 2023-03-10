// Copyright 2019-2023 PureStake Inc.
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

//! Precompile to receive GMP callbacks and forward to XCM

#![cfg_attr(not(feature = "std"), no_std)]

use xcm::latest::MultiLocation;

// TODO: design this with the following criteria in mind:
//       * friendly for other chains (performance, ease, security)
//       * future proof (bare minimum: version this)
//       * easy to parse
//       * flexible -- need to support "MVP" level of XCM functionality
pub struct XcmUserAction {
	pub destination: MultiLocation,
	pub destination_account: MultiLocation,
}

pub enum VersionedUserAction {
	V1(XcmUserAction),
}

/// Parse a user action from some bytes
pub fn parse_user_action(input: &Vec<u8>) -> Result<VersionedUserAction, &'static str> {
	// TODO: actually parse :)
	// more importantly, define a structure (see criteria above)

	Ok(VersionedUserAction::V1(XcmUserAction {
		destination: MultiLocation::parent(),
		destination_account: MultiLocation::parent(),
	}))
}

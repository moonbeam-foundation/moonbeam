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

//! The XCM primitive trait implementations

#![cfg_attr(not(feature = "std"), no_std)]

mod asset_id_conversions;
pub use asset_id_conversions::*;

mod barriers;
pub use barriers::*;

mod fee_handlers;
pub use fee_handlers::*;

mod location_conversion;
pub use location_conversion::*;

mod origin_conversion;
pub use origin_conversion::*;

mod transactor_traits;
pub use transactor_traits::*;

mod ethereum_xcm;
pub use ethereum_xcm::*;

mod filter_asset_max_fee;
pub use filter_asset_max_fee::*;

mod xcm_execution_traits;
pub use xcm_execution_traits::*;

pub type XcmV2Weight = xcm::v2::Weight;

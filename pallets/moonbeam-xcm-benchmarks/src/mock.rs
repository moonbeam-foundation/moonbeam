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

use frame_support::{parameter_types, traits::ContainsPair, weights::Weight};
use xcm::latest::prelude::*;
use xcm_executor::traits::ConvertLocation;

// An xcm sender/receiver akin to > /dev/null
pub struct DevNull;
impl SendXcm for DevNull {
	type Ticket = ();

	fn validate(
		_destination: &mut Option<Location>,
		_message: &mut Option<opaque::Xcm>,
	) -> SendResult<Self::Ticket> {
		Ok(((), Assets::new()))
	}

	fn deliver(_: Self::Ticket) -> Result<XcmHash, SendError> {
		Ok(XcmHash::default())
	}
}

impl xcm_executor::traits::OnResponse for DevNull {
	fn expecting_response(_: &Location, _: u64, _: Option<&Location>) -> bool {
		false
	}
	fn on_response(
		_: &Location,
		_: u64,
		_: Option<&Location>,
		_: Response,
		_: Weight,
		_: &XcmContext,
	) -> Weight {
		Weight::zero()
	}
}

pub struct AccountIdConverter;
impl ConvertLocation<u64> for AccountIdConverter {
	fn convert_location(ml: &Location) -> Option<u64> {
		match ml.unpack() {
			(0, [Junction::AccountId32 { id, .. }]) => {
				Some(<u64 as parity_scale_codec::Decode>::decode(&mut &*id.to_vec()).unwrap())
			}
			_ => None,
		}
	}
}

parameter_types! {
	pub Ancestry: Location = Junction::Parachain(101).into();
	pub UnitWeightCost: u64 = 10;
	pub WeightPrice: (AssetId, u128, u128) = (AssetId(Location::parent()), 1_000_000, 1024);
}

pub struct AllAssetLocationsPass;
impl ContainsPair<Asset, Location> for AllAssetLocationsPass {
	fn contains(_: &Asset, _: &Location) -> bool {
		true
	}
}

#[cfg(feature = "runtime-benchmarks")]
pub fn mock_worst_case_holding() -> Assets {
	let assets: Vec<Asset> = vec![Asset {
		id: AssetId(Location::parent()),
		fun: Fungible(u128::MAX),
	}];
	assets.into()
}

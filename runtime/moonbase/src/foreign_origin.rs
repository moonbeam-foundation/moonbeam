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

use crate::xcm_config::LocationToAccountId;
use crate::{Runtime, RuntimeOrigin};
use frame_support::ensure;
use frame_support::traits::{EnsureOrigin, Everything};
use frame_system::ensure_signed;
use moonbeam_core_primitives::AccountId;
use pallet_moonbeam_foreign_assets::EnsureXcmLocation;
use sp_runtime::DispatchError;
use xcm::latest::Location;
use xcm_executor::traits::ConvertLocation;

// `EnsureOriginWithArg` impl for `ForeignAssetOwnerOrigin` which allows only XCM origins
// which are locations containing the class location.
pub struct ForeignAssetsEnsureXcmLocation;

impl EnsureXcmLocation<Runtime> for ForeignAssetsEnsureXcmLocation {
	fn ensure_xcm_origin(
		o: RuntimeOrigin,
		location: &Location,
	) -> Result<AccountId, DispatchError> {
		let origin_account = ensure_signed(o.clone())?;
		let origin_location = pallet_xcm::EnsureXcm::<Everything>::try_origin(o.clone())
			.map_err(|_| DispatchError::BadOrigin)?;
		ensure!(
			location.starts_with(&origin_location),
			DispatchError::BadOrigin
		);
		Ok(Self::account_for_location(location).unwrap_or(origin_account))
	}

	fn account_for_location(location: &Location) -> Option<AccountId> {
		LocationToAccountId::convert_location(location)
	}
}

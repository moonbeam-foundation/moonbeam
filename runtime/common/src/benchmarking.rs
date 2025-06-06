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

use account::AccountId20;
use frame_support::traits::fungible::NativeOrWithId;
use moonbeam_core_primitives::AssetId;
use pallet_treasury::ArgumentsFactory;

pub struct BenchmarkHelper;

impl ArgumentsFactory<NativeOrWithId<AssetId>, AccountId20> for BenchmarkHelper {
	fn create_asset_kind(seed: u32) -> NativeOrWithId<AssetId> {
		NativeOrWithId::WithId(seed.into())
	}

	fn create_beneficiary(seed: [u8; 32]) -> AccountId20 {
		// Avoid generating a zero address
		if seed == [0; 32] {
			return AccountId20::from([1; 32]);
		}
		AccountId20::from(seed)
	}
}

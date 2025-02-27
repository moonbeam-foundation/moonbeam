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
use pallet_treasury::ArgumentsFactory;
use frame_support::traits::fungible::NativeOrWithId;
use pallet_asset_rate::AssetKindFactory;

pub struct BenchmarkHelper;

impl ArgumentsFactory<NativeOrWithId<u128>, AccountId20> for BenchmarkHelper {
	fn create_asset_kind(_seed: u32) -> NativeOrWithId<u128> {
		NativeOrWithId::Native
	}

	fn create_beneficiary(seed: [u8; 32]) -> AccountId20 {
		AccountId20::from(seed)
	}
}

impl AssetKindFactory<NativeOrWithId<u128>> for BenchmarkHelper {
	fn create_asset_kind(_seed: u32) -> NativeOrWithId<u128> {
		NativeOrWithId::Native
	}
}
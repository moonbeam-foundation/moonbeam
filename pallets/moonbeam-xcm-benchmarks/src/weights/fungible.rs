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

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;
use xcm::latest::MultiAsset;

// Values copied from statemint benchmarks
const ASSET_BURN_MAX_PROOF_SIZE: u64 = 7242;
const ASSET_MINT_MAX_PROOF_SIZE: u64 = 7242;
const ASSET_TRANSFER_MAX_PROOF_SIZE: u64 = 13412;

/// Weights for `pallet_xcm_benchmarks::fungible`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config + pallet_erc20_xcm_bridge::Config> WeightInfo<T> {
	pub(crate) fn withdraw_asset(asset: &MultiAsset) -> Weight {
		if pallet_erc20_xcm_bridge::Pallet::<T>::is_erc20_asset(asset) {
			pallet_erc20_xcm_bridge::Pallet::<T>::weight_of_erc20_transfer(&asset.id)
			
		} else {
			Weight::from_parts(200_000_000 as u64, ASSET_BURN_MAX_PROOF_SIZE)
		}
	}
	pub(crate) fn transfer_asset(asset: &MultiAsset) -> Weight {
		if pallet_erc20_xcm_bridge::Pallet::<T>::is_erc20_asset(asset) {
			pallet_erc20_xcm_bridge::Pallet::<T>::weight_of_erc20_transfer(&asset.id)
		} else {
			Weight::from_parts(200_000_000 as u64, ASSET_TRANSFER_MAX_PROOF_SIZE)
		}
	}
	pub(crate) fn transfer_reserve_asset(asset: &MultiAsset) -> Weight {
		if pallet_erc20_xcm_bridge::Pallet::<T>::is_erc20_asset(asset) {
			pallet_erc20_xcm_bridge::Pallet::<T>::weight_of_erc20_transfer(&asset.id)
		} else {
			Weight::from_parts(200_000_000 as u64, ASSET_TRANSFER_MAX_PROOF_SIZE)
		}
	}
	pub(crate) fn receive_teleported_asset() -> Weight {
		// Instruction disabled
		Weight::MAX
	}
	pub(crate) fn deposit_asset() -> Weight {
		Weight::from_parts(200_000_000 as u64, ASSET_MINT_MAX_PROOF_SIZE)
	}
	pub(crate) fn deposit_reserve_asset() -> Weight {
		Weight::from_parts(200_000_000 as u64, ASSET_MINT_MAX_PROOF_SIZE)
	}
	pub(crate) fn initiate_teleport() -> Weight {
		// Instruction disabled
		Weight::MAX
	}
	pub(crate) fn reserve_asset_deposited() -> Weight {
		// This instruction is a no-op for PoV (no storage access)
		Weight::from_parts(200_000_000 as u64, 0)
	}
}

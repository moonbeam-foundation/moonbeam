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

use frame_support::traits::Get;
use xcm::latest::MultiAsset;
use xcm_builder::Case;

/// Filters max fee for a given multiasset.
/// It takes self (a multiasset) and runs contains in the argument multiasset
/// Can be amalgamated into tuples.
/// If any item returns `true`, it short-circuits, else `false` is returned.
pub trait FilterMaxAssetFee {
	/// A filter to be able to compare against a max asset.
	fn filter_max_asset_fee(asset: &MultiAsset) -> bool;
}

#[impl_trait_for_tuples::impl_for_tuples(30)]
impl FilterMaxAssetFee for Tuple {
	fn filter_max_asset_fee(what: &MultiAsset) -> bool {
		for_tuples!( #(
			if Tuple::filter_max_asset_fee(what) { return true }
		)* );
		log::trace!(
			target: "xcm::filter_max_asset_fee",
			"got filtered: what: {:?}",
			what,
		);
		false
	}
}

impl<T: Get<MultiAsset>> FilterMaxAssetFee for Case<T> {
	fn filter_max_asset_fee(asset: &MultiAsset) -> bool {
		log::trace!(target: "xcm::filter_max_asset_fee", "Case asset: {:?}", asset);
		let max = T::get();
		max.contains(asset)
	}
}

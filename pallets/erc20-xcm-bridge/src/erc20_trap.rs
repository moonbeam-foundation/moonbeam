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

use sp_std::vec::Vec;
use xcm::latest::prelude::*;
use xcm_executor::traits::DropAssets;

/// Morph a given `DropAssets` implementation into one which filter out erc20 assets.
pub struct AssetTrapWrapper<AssetTrap, T>(core::marker::PhantomData<(AssetTrap, T)>);

// Morph a given `DropAssets` implementation into one which filter out erc20 assets.
impl<AssetTrap: DropAssets, T: crate::Config> DropAssets for AssetTrapWrapper<AssetTrap, T> {
	fn drop_assets(
		origin: &xcm::latest::Location,
		mut assets: xcm_executor::AssetsInHolding,
		context: &XcmContext,
	) -> xcm::latest::Weight {
		// Remove all erc20 assets
		let assets_to_remove: Vec<_> = assets
			.fungible_assets_iter()
			.filter_map(|multiasset| {
				crate::Pallet::<T>::is_erc20_asset(&multiasset).then_some(multiasset.id)
			})
			.collect();
		for id in assets_to_remove {
			assets.saturating_take(xcm::latest::AssetFilter::Wild(
				xcm::latest::WildAsset::AllOf {
					fun: WildFungible,
					id,
				},
			));
		}
		AssetTrap::drop_assets(origin, assets, context)
	}
}

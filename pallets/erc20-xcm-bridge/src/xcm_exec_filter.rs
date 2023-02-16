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

use frame_support::traits::Contains;
use xcm::latest::prelude::*;

type RuntimeCallOf<T> = <T as frame_system::Config>::RuntimeCall;

/// Morph a given "xcm execution" filter implementation into one which filter out
/// some XCM instructions if they could manipulate erc20 assets.
/// If your runtime allows arbitrary xcm messages to be executed locally you should use this
// wrapper.
pub struct XcmExecuteFilterWrapper<Runtime, FallbackFilter>(
	core::marker::PhantomData<(Runtime, FallbackFilter)>,
);

impl<T, FallbackFilter> Contains<(MultiLocation, Xcm<RuntimeCallOf<T>>)>
	for XcmExecuteFilterWrapper<T, FallbackFilter>
where
	T: crate::Config,
	FallbackFilter: Contains<(MultiLocation, Xcm<RuntimeCallOf<T>>)>,
{
	// To be sure that the execution of this message will not lead to instructions incompatible
	// with the erc20 assets to manipulate them, we must "quickly simulate" the execution of the
	// message to know what will be in the xcm holding when we wait for the problematic
	// instructions.
	fn contains((_location, message): &(MultiLocation, Xcm<RuntimeCallOf<T>>)) -> bool {
		// Track erc20 assets in the "simulated xcm holding"
		let mut erc20_assets = sp_std::collections::btree_set::BTreeSet::new();
		for instruction in &message.0 {
			match instruction {
				// Fill in the "simulated xcm holding"
				Instruction::WithdrawAsset(assets) => {
					for asset in assets.inner() {
						if crate::Pallet::<T>::is_erc20_asset(asset) {
							erc20_assets.insert(&asset.id);
						}
					}
				}
				// Take erc20 assets from the "simulated xcm holding"
				Instruction::DepositAsset { assets, .. }
				| Instruction::DepositReserveAsset { assets, .. } => match assets {
					MultiAssetFilter::Wild(All) => {
						erc20_assets = Default::default();
					}
					MultiAssetFilter::Wild(AllOf {
						fun: WildFungible,
						id,
					}) => {
						erc20_assets.remove(id);
					}
					MultiAssetFilter::Wild(AllOf {
						fun: WildNonFungible,
						..
					}) => {}
					MultiAssetFilter::Definite(_assets) => {
						// If we wanted a perfect simulation, we would have to remove the assets
						//  defined according to their amounts, but this would involve calculations
						// on the balances.
						// To keep the simulation fast, we voluntarily choose to  do nothing here,
						// This implies to refuse some messages that are not problematic but that
						// would take too long to check properly.
					}
				},
				// Theses instructions should never handle erc20 assets
				// So, if the "simulated xcm holding" contains erc20 assets compatibles with
				// the `MultiAssetFilter` the message should be forbid.
				Instruction::InitiateReserveWithdraw { assets, .. }
				| Instruction::InitiateTeleport { assets, .. } => match assets {
					MultiAssetFilter::Wild(All) => {
						if !erc20_assets.is_empty() {
							return false;
						}
					}
					MultiAssetFilter::Wild(AllOf {
						fun: WildFungible,
						id,
					}) => {
						if erc20_assets.contains(id) {
							return false;
						}
					}
					MultiAssetFilter::Wild(AllOf {
						fun: WildNonFungible,
						..
					}) => {}
					MultiAssetFilter::Definite(assets) => {
						for asset in assets.inner() {
							if erc20_assets.contains(&asset.id) {
								return false;
							}
						}
					}
				},
				// We don't care of any other XCM instructions
				_ => {}
			}
		}
		true
	}
}

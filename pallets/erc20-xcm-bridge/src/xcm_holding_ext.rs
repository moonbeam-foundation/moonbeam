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

//! Module that provides types to extend xcm holding.

use crate::erc20_matcher::Erc20Matcher;
use core::marker::PhantomData;
use sp_core::{H160, U256};
use sp_std::collections::btree_map::BTreeMap;
use sp_std::vec::Vec;
use xcm_executor::traits::{DropAssets, MatchesFungibles};

environmental::environmental!(XCM_HOLDING_ERC20_ORIGINS: XcmHoldingErc20sOrigins);

#[cfg_attr(test, derive(PartialEq, Debug))]
pub(crate) enum DrainError {
	NotEnoughFounds,
	AssetNotFound,
}

/// Xcm holding erc20 origins extension.
/// This extension track down the origin of alls erc20 tokens in the xcm holding.
#[derive(Default)]
pub(crate) struct XcmHoldingErc20sOrigins {
	map: BTreeMap<H160, Vec<(H160, U256)>>,
}
impl XcmHoldingErc20sOrigins {
	pub(crate) fn drain(
		&mut self,
		contract_address: H160,
		amount: U256,
	) -> Result<Vec<(H160, U256)>, DrainError> {
		let tokens_to_transfer = self.drain_inner(&contract_address, amount)?;

		self.map
			.entry(contract_address)
			.and_modify(|erc20_origins| {
				if tokens_to_transfer.len() > 1 {
					*erc20_origins = erc20_origins.split_off(tokens_to_transfer.len() - 1);
				}
				if erc20_origins.len() > 0 && tokens_to_transfer.len() > 0 {
					let last_index = erc20_origins.len() - 1;
					erc20_origins[last_index] = tokens_to_transfer[tokens_to_transfer.len() - 1];
				}
			});

		Ok(tokens_to_transfer)
	}
	fn drain_inner(
		&self,
		contract_address: &H160,
		mut amount: U256,
	) -> Result<Vec<(H160, U256)>, DrainError> {
		let mut tokens_to_transfer = Vec::new();
		if let Some(erc20_origins) = self.map.get(contract_address) {
			for (from, subamount) in erc20_origins {
				let amount_to_transfer = core::cmp::min(amount, *subamount);
				amount -= amount_to_transfer;
				tokens_to_transfer.push((*from, amount_to_transfer));
				//*subamount -= amount_to_transfer;
				if amount == U256::zero() {
					return Ok(tokens_to_transfer);
				}
			}
			if amount == U256::zero() {
				Ok(tokens_to_transfer)
			} else {
				Err(DrainError::NotEnoughFounds)
			}
		} else {
			Err(DrainError::AssetNotFound)
		}
	}
	pub(crate) fn insert(&mut self, contract_address: H160, who: H160, amount: U256) {
		self.map
			.entry(contract_address)
			.or_default()
			.push((who, amount));
	}
	pub(crate) fn with<R, F>(f: F) -> Option<R>
	where
		F: FnOnce(&mut Self) -> R,
	{
		XCM_HOLDING_ERC20_ORIGINS::with(|erc20s_origins| f(erc20s_origins))
	}
}

// Morph a given `DropAssets` implementation into one which filter out erc20 assets.
pub struct AssetTrapWrapper<AssetTrap, Runtime>(core::marker::PhantomData<(AssetTrap, Runtime)>);

impl<AssetTrap: DropAssets, Runtime: crate::Config> DropAssets
	for AssetTrapWrapper<AssetTrap, Runtime>
{
	fn drop_assets(
		origin: &xcm::latest::MultiLocation,
		mut assets: xcm_executor::Assets,
	) -> xcm::latest::Weight {
		// Remove all erc20 assets
		let assets_to_remove: Vec<_> = assets
			.fungible_assets_iter()
			.filter_map(|multiasset| {
				if Erc20Matcher::<Runtime>::matches_fungibles(&multiasset).is_ok() {
					Some(multiasset.id)
				} else {
					None
				}
			})
			.collect();
		for id in assets_to_remove {
			assets.saturating_take(xcm::latest::MultiAssetFilter::Wild(
				xcm::latest::WildMultiAsset::AllOf {
					fun: xcm::latest::prelude::WildFungible,
					id,
				},
			));
		}
		AssetTrap::drop_assets(origin, assets)
	}
}

/// Xcm executor wrapper that inject xcm holding extension "XcmHoldingErc20sOrigins"
pub struct XcmExecutorWrapper<RuntimeCall, InnerXcmExecutor>(
	PhantomData<(RuntimeCall, InnerXcmExecutor)>,
);
impl<RuntimeCall, InnerXcmExecutor> xcm::latest::ExecuteXcm<RuntimeCall>
	for XcmExecutorWrapper<RuntimeCall, InnerXcmExecutor>
where
	InnerXcmExecutor: xcm::latest::ExecuteXcm<RuntimeCall>,
{
	fn execute_xcm_in_credit(
		origin: impl Into<xcm::latest::MultiLocation>,
		message: xcm::latest::Xcm<RuntimeCall>,
		weight_limit: xcm::latest::Weight,
		weight_credit: xcm::latest::Weight,
	) -> xcm::latest::Outcome {
		let mut erc20s_origins = Default::default();
		XCM_HOLDING_ERC20_ORIGINS::using(&mut erc20s_origins, || {
			InnerXcmExecutor::execute_xcm_in_credit(origin, message, weight_limit, weight_credit)
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_xcm_holding_ext_erc20s_origins() {
		const TOKEN1: H160 = H160([1; 20]);
		const TOKEN2: H160 = H160([2; 20]);
		const USER1: H160 = H160([3; 20]);
		const USER2: H160 = H160([4; 20]);

		// Simple case
		let mut erc20s_origins_ = Default::default();
		XCM_HOLDING_ERC20_ORIGINS::using(&mut erc20s_origins_, || {
			XcmHoldingErc20sOrigins::with(|erc20s_origins| {
				erc20s_origins.insert(TOKEN1, USER1, U256::from(100));
				assert_eq!(
					erc20s_origins.drain(TOKEN2, U256::from(1)),
					Err(DrainError::AssetNotFound)
				);
				assert_eq!(
					erc20s_origins.drain(TOKEN1, U256::from(100)),
					Ok(vec![(USER1, U256::from(100))])
				);
			})
		});

		// Complex case
		let mut erc20s_origins_ = Default::default();
		XCM_HOLDING_ERC20_ORIGINS::using(&mut erc20s_origins_, || {
			XcmHoldingErc20sOrigins::with(|erc20s_origins| {
				erc20s_origins.insert(TOKEN1, USER1, U256::from(100));
				erc20s_origins.insert(TOKEN1, USER2, U256::from(200));
				assert_eq!(
					erc20s_origins.drain(TOKEN1, U256::from(200)),
					Ok(vec![(USER1, U256::from(100)), (USER2, U256::from(100))])
				);
				assert_eq!(
					erc20s_origins.drain(TOKEN1, U256::from(50)),
					Ok(vec![(USER2, U256::from(50))])
				);
				assert_eq!(
					erc20s_origins.drain(TOKEN1, U256::from(51)),
					Err(DrainError::NotEnoughFounds)
				);
			})
		});
	}
}

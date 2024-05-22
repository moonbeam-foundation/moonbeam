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

use core::marker::PhantomData;
use sp_core::{H160, U256};
use sp_std::collections::btree_map::BTreeMap;
use sp_std::vec::Vec;
use xcm_executor::traits::XcmAssetTransfers;

environmental::environmental!(XCM_HOLDING_ERC20_ORIGINS: XcmHoldingErc20sOrigins);

#[cfg_attr(test, derive(PartialEq, Debug))]
pub(crate) enum DrainError {
	AssetNotFound,
	NotEnoughFounds,
	SplitError,
}

/// Xcm holding erc20 origins extension.
/// This extension track down the origin of alls erc20 tokens in the xcm holding.
#[derive(Default)]
pub(crate) struct XcmHoldingErc20sOrigins {
	map: BTreeMap<H160, Vec<(H160, U256)>>,
}
impl XcmHoldingErc20sOrigins {
	/// Take and remove a given amounts of erc20 tokens from the XCM holding.
	/// These tokens can come from one or more holders that we had tracked earlier in the XCM
	/// execution, so we return an array of (holder, balance).
	pub(crate) fn drain(
		&mut self,
		contract_address: H160,
		amount: U256,
	) -> Result<Vec<(H160, U256)>, DrainError> {
		let tokens_to_transfer = self.drain_inner(&contract_address, amount)?;

		self.map
			.entry(contract_address)
			.and_modify(|erc20_origins| {
				*erc20_origins = erc20_origins.split_off(tokens_to_transfer.len());
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
				if &amount > subamount {
					tokens_to_transfer.push((*from, *subamount));
					#[allow(clippy::arithmetic_side_effects)]
					{
						// Safe substraction because we check "amount > subamount" 2 lines above
						amount -= *subamount;
					}
				} else if &amount == subamount {
					tokens_to_transfer.push((*from, *subamount));
					return Ok(tokens_to_transfer);
				} else {
					// Each insertion of tokens must be drain at once
					return Err(DrainError::SplitError);
				}
			}
			// If there were enough tokens, we had to return in the for loop
			Err(DrainError::NotEnoughFounds)
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

/// Xcm executor wrapper that inject xcm holding extension "XcmHoldingErc20sOrigins"
pub struct XcmExecutorWrapper<Config, InnerXcmExecutor>(PhantomData<(Config, InnerXcmExecutor)>);
impl<Config, InnerXcmExecutor> xcm::latest::ExecuteXcm<Config::RuntimeCall>
	for XcmExecutorWrapper<Config, InnerXcmExecutor>
where
	Config: xcm_executor::Config,
	InnerXcmExecutor: xcm::latest::ExecuteXcm<Config::RuntimeCall>,
{
	type Prepared = InnerXcmExecutor::Prepared;

	fn prepare(
		message: xcm::latest::Xcm<Config::RuntimeCall>,
	) -> Result<Self::Prepared, xcm::latest::Xcm<Config::RuntimeCall>> {
		InnerXcmExecutor::prepare(message)
	}

	fn execute(
		origin: impl Into<xcm::latest::Location>,
		pre: Self::Prepared,
		hash: &mut xcm::latest::XcmHash,
		weight_credit: xcm::latest::Weight,
	) -> xcm::latest::Outcome {
		let mut erc20s_origins = Default::default();
		XCM_HOLDING_ERC20_ORIGINS::using(&mut erc20s_origins, || {
			InnerXcmExecutor::execute(origin, pre, hash, weight_credit)
		})
	}

	fn charge_fees(
		location: impl Into<xcm::latest::Location>,
		fees: xcm::latest::Assets,
	) -> Result<(), xcm::latest::Error> {
		InnerXcmExecutor::charge_fees(location, fees)
	}
}

impl<Config, InnerXcmExecutor> XcmAssetTransfers for XcmExecutorWrapper<Config, InnerXcmExecutor>
where
	Config: xcm_executor::Config,
{
	type IsReserve = Config::IsReserve;
	type IsTeleporter = Config::IsTeleporter;
	type AssetTransactor = Config::AssetTransactor;
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
					erc20s_origins.drain(TOKEN1, U256::from(100)),
					Ok(vec![(USER1, U256::from(100))])
				);
				assert_eq!(
					erc20s_origins.drain(TOKEN1, U256::from(201)),
					Err(DrainError::NotEnoughFounds)
				);
				assert_eq!(
					erc20s_origins.drain(TOKEN1, U256::from(199)),
					Err(DrainError::SplitError)
				);
				assert_eq!(
					erc20s_origins.drain(TOKEN1, U256::from(200)),
					Ok(vec![(USER2, U256::from(200))])
				);
			})
		});
	}
}

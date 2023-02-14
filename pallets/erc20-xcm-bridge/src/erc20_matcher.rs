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

//! Module that provides types to match erc20 assets.

use sp_core::{Get, H160, U256};
use xcm::latest::prelude::*;
use xcm::latest::{Junction, MultiLocation};
use xcm_executor::traits::{Error as MatchError, MatchesFungibles};

pub(crate) struct Erc20Matcher<Runtime>(core::marker::PhantomData<Runtime>);

impl<Runtime: crate::Config> MatchesFungibles<H160, U256> for Erc20Matcher<Runtime> {
	fn matches_fungibles(multiasset: &MultiAsset) -> Result<(H160, U256), MatchError> {
		let (amount, id) = match (&multiasset.fun, &multiasset.id) {
			(Fungible(ref amount), Concrete(ref id)) => (amount, id),
			_ => return Err(MatchError::AssetNotFound),
		};
		let contract_address = Self::matches_erc20_multilocation(id)
			.map_err(|_| MatchError::AssetIdConversionFailed)?;
		let amount =
			U256::try_from(*amount).map_err(|_| MatchError::AmountToBalanceConversionFailed)?;

		Ok((contract_address, amount))
	}
}

impl<Runtime: crate::Config> Erc20Matcher<Runtime> {
	fn matches_erc20_multilocation(multilocation: &MultiLocation) -> Result<H160, ()> {
		let prefix = Runtime::Erc20MultilocationPrefix::get();
		if prefix.parent_count() != multilocation.parent_count()
			|| prefix
				.interior()
				.iter()
				.enumerate()
				.any(|(index, junction)| multilocation.interior().at(index) != Some(junction))
		{
			return Err(());
		}
		match multilocation.interior().at(prefix.interior().len()) {
			Some(Junction::AccountKey20 {
				key: contract_address,
				..
			}) => Ok(H160(*contract_address)),
			_ => Err(()),
		}
	}
}

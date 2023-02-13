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
use xcm::latest::{Junction, MultiLocation, NetworkId};
use xcm_executor::traits::Error as MatchError;

pub(crate) struct Erc20Asset {
	pub contract_address: H160,
	pub amount: U256,
	pub maybe_holder: Option<H160>,
}

pub(crate) struct Erc20Matcher<Runtime>(core::marker::PhantomData<Runtime>);

impl<Runtime: crate::Config> Erc20Matcher<Runtime> {
	pub(crate) fn insert_holder(
		multiasset: &MultiAsset,
		who: H160,
	) -> Result<MultiAsset, MatchError> {
		match (&multiasset.fun, &multiasset.id) {
			(Fungible(ref amount), Concrete(ref multilocation)) => Ok(MultiAsset {
				fun: Fungible(*amount),
				id: Concrete(
					multilocation
						.clone()
						.pushed_with_interior(Junction::AccountKey20 {
							key: who.0,
							network: NetworkId::Any,
						})
						.map_err(|_| MatchError::AssetIdConversionFailed)?,
				),
			}),
			_ => Err(MatchError::AssetNotFound),
		}
	}
	pub(crate) fn matches_erc20(multiasset: &MultiAsset) -> Result<Erc20Asset, MatchError> {
		let (amount, id) = match (&multiasset.fun, &multiasset.id) {
			(Fungible(ref amount), Concrete(ref id)) => (amount, id),
			_ => return Err(MatchError::AssetNotFound),
		};
		let (contract_address, maybe_holder) = Self::matches_erc20_multilocation(id)
			.map_err(|_| MatchError::AssetIdConversionFailed)?;
		let amount =
			U256::try_from(*amount).map_err(|_| MatchError::AmountToBalanceConversionFailed)?;
		Ok(Erc20Asset {
			contract_address,
			amount,
			maybe_holder,
		})
	}
	fn matches_erc20_multilocation(
		multilocation: &MultiLocation,
	) -> Result<(H160, Option<H160>), ()> {
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
		let prefix_len = prefix.interior().len();
		match multilocation.interior().at(prefix_len) {
			Some(Junction::AccountKey20 {
				key: contract_address,
				..
			}) => match prefix_len.checked_add(1) {
				Some(prefix_len_plus_one) => match multilocation.interior().at(prefix_len_plus_one)
				{
					Some(Junction::AccountKey20 { key: holder, .. }) => {
						Ok((H160(*contract_address), Some(H160(*holder))))
					}
					None => Ok((H160(*contract_address), None)),
					_ => Err(()),
				},
				_ => Err(()),
			},
			_ => Err(()),
		}
	}
}

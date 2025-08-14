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

//! Module that provides types to match erc20 assets.

use sp_core::{Get, H160, U256};
use xcm::latest::prelude::*;
use xcm::latest::{Junction, Location};
use xcm_executor::traits::{Error as MatchError, MatchesFungibles};

pub(crate) struct Erc20Matcher<Erc20MultilocationPrefix>(
	core::marker::PhantomData<Erc20MultilocationPrefix>,
);

impl<Erc20MultilocationPrefix: Get<Location>> MatchesFungibles<H160, U256>
	for Erc20Matcher<Erc20MultilocationPrefix>
{
	fn matches_fungibles(multiasset: &Asset) -> Result<(H160, U256), MatchError> {
		let (amount, id) = match (&multiasset.fun, &multiasset.id) {
			(Fungible(ref amount), AssetId(ref id)) => (amount, id),
			_ => return Err(MatchError::AssetNotHandled),
		};
		let contract_address = Self::matches_erc20_multilocation(id)
			.map_err(|_| MatchError::AssetIdConversionFailed)?;
		let amount = U256::from(*amount);

		Ok((contract_address, amount))
	}
}

impl<Erc20MultilocationPrefix: Get<Location>> Erc20Matcher<Erc20MultilocationPrefix> {
	pub(crate) fn is_erc20_asset(multiasset: &Asset) -> bool {
		match (&multiasset.fun, &multiasset.id) {
			(Fungible(_), AssetId(ref id)) => Self::matches_erc20_multilocation(id).is_ok(),
			_ => false,
		}
	}
	fn matches_erc20_multilocation(multilocation: &Location) -> Result<H160, ()> {
		let prefix = Erc20MultilocationPrefix::get();
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

#[cfg(test)]
mod tests {
	use super::*;

	macro_rules! assert_ok {
		( $x:expr, $y:expr $(,)? ) => {
			let is = $x;
			match is {
				Ok(ok) => assert_eq!(ok, $y),
				_ => assert!(false, "Expected Ok(_). Got Err(_)"),
			}
		};
	}

	frame_support::parameter_types! {
		pub Erc20MultilocationPrefix: Location = Location {
			parents:0,
			interior: [PalletInstance(42u8)].into()
		};
	}

	#[test]
	fn should_match_valid_erc20_location() {
		let location = Location {
			parents: 0,
			interior: [
				PalletInstance(42u8),
				AccountKey20 {
					key: [0; 20],
					network: None,
				},
			]
			.into(),
		};

		assert_ok!(
			Erc20Matcher::<Erc20MultilocationPrefix>::matches_fungibles(&Asset::from((
				location, 100u128
			))),
			(H160([0; 20]), U256([100, 0, 0, 0]))
		);
	}

	#[test]
	fn should_match_valid_erc20_location_with_amount_greater_than_u64() {
		let location = Location {
			parents: 0,
			interior: [
				PalletInstance(42u8),
				AccountKey20 {
					key: [0; 20],
					network: None,
				},
			]
			.into(),
		};

		assert_ok!(
			Erc20Matcher::<Erc20MultilocationPrefix>::matches_fungibles(&Asset::from((
				location,
				100000000000000000u128
			))),
			(H160([0; 20]), U256::from(100000000000000000u128))
		);
	}

	#[test]
	fn should_not_match_invalid_erc20_location() {
		let invalid_location = Location {
			parents: 0,
			interior: [PalletInstance(42u8), GeneralIndex(0)].into(),
		};

		assert!(
			Erc20Matcher::<Erc20MultilocationPrefix>::matches_fungibles(&Asset::from((
				invalid_location,
				100u128
			)))
			.is_err()
		);
	}
}

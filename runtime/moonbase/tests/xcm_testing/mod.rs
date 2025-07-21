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

//! Moonbase Runtime Xcm Tests

use crate::xcm_mock::*;
use parity_scale_codec::{Decode, Encode};
use sp_io::hashing::blake2_256;
use sp_runtime::traits::Convert;
use sp_weights::Weight;
use xcm::latest::prelude::{Asset, AssetId, Fungibility};
use xcm::{IntoVersion, VersionedLocation};

pub mod helpers;

mod automatic_versioning;
mod evm_accounts;
mod hrmp;
mod para_asset;
mod relay_asset;
mod statemint;
mod transact_derivative;
mod transact_signed;
mod transact_sovereign;

// Re-export helpers for easy access

// Helper to derive accountIds
pub fn derivative_account_id(who: sp_runtime::AccountId32, index: u16) -> sp_runtime::AccountId32 {
	let entropy = (b"modlpy/utilisuba", who, index).using_encoded(blake2_256);
	sp_runtime::AccountId32::decode(&mut &entropy[..]).expect("valid account id")
}

pub fn add_supported_asset(
	asset_type: parachain::AssetType,
	units_per_second: u128,
) -> Result<(), ()> {
	let parachain::AssetType::Xcm(location_v3) = asset_type;
	let VersionedLocation::V5(location_v5) = VersionedLocation::V3(location_v3)
		.into_version(xcm::latest::VERSION)
		.map_err(|_| ())?
	else {
		return Err(());
	};
	use frame_support::weights::WeightToFee as _;
	let native_amount_per_second: u128 =
		<parachain::Runtime as pallet_xcm_weight_trader::Config>::WeightToFee::weight_to_fee(
			&Weight::from_parts(
				frame_support::weights::constants::WEIGHT_REF_TIME_PER_SECOND,
				0,
			),
		)
		.try_into()
		.map_err(|_| ())?;
	let precision_factor = 10u128.pow(pallet_xcm_weight_trader::RELATIVE_PRICE_DECIMALS);
	let relative_price: u128 = if units_per_second > 0u128 {
		native_amount_per_second
			.saturating_mul(precision_factor)
			.saturating_div(units_per_second)
	} else {
		0u128
	};
	pallet_xcm_weight_trader::SupportedAssets::<parachain::Runtime>::insert(
		location_v5,
		(true, relative_price),
	);
	Ok(())
}

pub fn currency_to_asset(currency_id: parachain::CurrencyId, amount: u128) -> Asset {
	Asset {
		id: AssetId(
			<parachain::Runtime as pallet_xcm_transactor::Config>::CurrencyIdToLocation::convert(
				currency_id,
			)
			.unwrap(),
		),
		fun: Fungibility::Fungible(amount),
	}
}

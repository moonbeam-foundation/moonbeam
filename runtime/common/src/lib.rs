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

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::weights::{constants::WEIGHT_REF_TIME_PER_MILLIS, Weight};
use sp_core::H160;
use sp_runtime::Perbill;

mod apis;
mod impl_moonbeam_xcm_call;
mod impl_moonbeam_xcm_call_tracing;
mod impl_on_charge_evm_transaction;
mod impl_self_contained_call;
pub mod migrations;

// Here we assume Ethereum's base fee of 21000 gas and convert to weight, but we
// subtract roughly the cost of a balance transfer from it (about 1/3 the cost)
// and some cost to account for per-byte-fee.
// TODO: we should use benchmarking's overhead feature to measure this
pub fn extrinsic_base_weight(weight_per_gas: Weight) -> Weight {
	weight_per_gas * 10000
}

/// `WeightPerGas` is an approximate ratio of the amount of Weight per Gas.
/// u64 works for approximations because Weight is a very small unit compared to gas.
///
/// `GAS_PER_MILLIS * WEIGHT_MILLIS_PER_BLOCK * TXN_RATIO ~= BLOCK_GAS_LIMIT`
/// `WEIGHT_PER_GAS = WEIGHT_REF_TIME_PER_MILLIS / GAS_PER_MILLIS
///       = WEIGHT_REF_TIME_PER_MILLIS / (BLOCK_GAS_LIMIT / TXN_RATIO / WEIGHT_MILLIS_PER_BLOCK)
///       = TXN_RATIO * (WEIGHT_REF_TIME_PER_MILLIS * WEIGHT_MILLIS_PER_BLOCK) / BLOCK_GAS_LIMIT`
///
/// For example, given the 500ms Weight, from which 75% only are used for transactions,
/// the total EVM execution gas limit is `GAS_PER_MILLIS * 500 * 75% = BLOCK_GAS_LIMIT`.
pub fn weight_per_gas(
	block_gas_limit: u64,
	txn_ratio: Perbill,
	weight_millis_per_block: u64,
) -> u64 {
	let weight_per_block = WEIGHT_REF_TIME_PER_MILLIS.saturating_mul(weight_millis_per_block);
	let weight_per_gas = (txn_ratio * weight_per_block).saturating_div(block_gas_limit);
	assert!(
		weight_per_gas >= 1,
		"WeightPerGas must greater than or equal with 1"
	);
	weight_per_gas
}

//TODO maybe this should be upstreamed into Frontier.

/// And implementation of Frontier's AddressMapping trait for Moonbeam Accounts.
/// This is basically identical to Frontier's own IdentityAddressMapping, but it works for any type
/// that is Into<H160> like AccountId20 for example.
pub struct IntoAddressMapping;

impl<T: From<H160>> pallet_evm::AddressMapping<T> for IntoAddressMapping {
	fn into_account_id(address: H160) -> T {
		address.into()
	}
}

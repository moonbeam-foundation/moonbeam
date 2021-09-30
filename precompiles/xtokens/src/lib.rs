// Copyright 2019-2021 PureStake Inc.
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

//! Precompile to xtokens runtime methods via the EVM

#![cfg_attr(not(feature = "std"), no_std)]

use evm::{executor::PrecompileOutput, Context, ExitError, ExitSucceed};
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use pallet_evm::{AddressMapping, Precompile};
use precompile_utils::{
	error, Address, Bytes, EvmData, EvmDataReader, EvmDataWriter, EvmResult, Gasometer,
	RuntimeHelper,
};

use sp_core::{H160, U256};
use sp_std::{
	convert::{TryFrom, TryInto},
	fmt::Debug,
	marker::PhantomData,
	vec::Vec,
};
mod encoding;
use encoding::Encoder;
use sp_std::boxed::Box;
use xcm::v1::{AssetId, Fungibility, Junctions, MultiAsset, MultiLocation};
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub type BalanceOf<Runtime> = <Runtime as orml_xtokens::Config>::Balance;

pub type CurrencyIdOf<Runtime> = <Runtime as orml_xtokens::Config>::CurrencyId;

#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq, num_enum::TryFromPrimitive, num_enum::IntoPrimitive)]
pub enum Action {
	Transfer = "transfer(address, u256, bytes[], u64)",
	TransferMultiAsset = "transfer_multiasset(bytes[], u256, bytes[], u64)",
}

/// This trait ensure we can convert AccountIds to AssetIds
/// We will require Runtime to have this trait implemented
pub trait AccountIdToCurrencyId<Account, CurrencyId> {
	// Get assetId from account
	fn account_to_currency_id(account: Account) -> Option<CurrencyId>;
}

/// A precompile to wrap the functionality from xtokens
pub struct XtokensWrapper<Runtime>(PhantomData<Runtime>);

impl<Runtime> Precompile for XtokensWrapper<Runtime>
where
	Runtime: orml_xtokens::Config + pallet_evm::Config + frame_system::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::Call: From<orml_xtokens::Call<Runtime>>,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	BalanceOf<Runtime>: TryFrom<U256> + Into<U256> + EvmData,
	Runtime: AccountIdToCurrencyId<Runtime::AccountId, CurrencyIdOf<Runtime>>,
{
	fn execute(
		input: &[u8], //Reminder this is big-endian
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<PrecompileOutput, ExitError> {
		let mut input = EvmDataReader::new(input);

		match &input.read_selector()? {
			// Check for accessor methods first. These return results immediately
			Action::Transfer => Self::transfer(input, target_gas, context),
			Action::TransferMultiAsset => Self::transfer_multiasset(input, target_gas, context),
		}
	}
}

impl<Runtime> XtokensWrapper<Runtime>
where
	Runtime: orml_xtokens::Config + pallet_evm::Config + frame_system::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::Call: From<orml_xtokens::Call<Runtime>>,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	BalanceOf<Runtime>: TryFrom<U256> + Into<U256> + EvmData,
	Runtime: AccountIdToCurrencyId<Runtime::AccountId, CurrencyIdOf<Runtime>>,
{
	fn transfer(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);

		// Bound check
		input.expect_arguments(2)?;
		let to_address: H160 = input.read::<Address>()?.into();
		let amount: U256 = input.read()?;

		let destination: MultiLocation = input.read::<MultiLocationWrapper>()?.into();

		// Bound check
		input.expect_arguments(1)?;
		let weight: u64 = input.read::<u64>()?;

		let to_account = Runtime::AddressMapping::into_account_id(to_address);
		let to_currency_id: <Runtime as orml_xtokens::Config>::CurrencyId =
			Runtime::account_to_currency_id(to_account)
				.ok_or(error("cannot convert into currency id"))?;

		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let to_balance = amount
			.try_into()
			.map_err(|_| error("Amount is too large for provided balance type"))?;

		let call = orml_xtokens::Call::<Runtime>::transfer(
			to_currency_id,
			to_balance,
			Box::new(destination),
			weight,
		);

		let used_gas = RuntimeHelper::<Runtime>::try_dispatch(
			Some(origin).into(),
			call,
			gasometer.remaining_gas()?,
		)?;

		gasometer.record_cost(used_gas)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn transfer_multiasset(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);
		// read destination
		let asset_multilocation: MultiLocation = input.read::<MultiLocationWrapper>()?.into();
		// Bound check
		input.expect_arguments(1)?;
		let amount: U256 = input.read()?;

		// read destination
		let destination: MultiLocation = input.read::<MultiLocationWrapper>()?.into();

		// Bound check
		input.expect_arguments(1)?;
		let weight: u64 = input.read::<u64>()?;

		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let to_balance = amount
			.try_into()
			.map_err(|_| error("Amount is too large for provided balance type"))?;

		let call = orml_xtokens::Call::<Runtime>::transfer_multiasset(
			Box::new(MultiAsset {
				id: AssetId::Concrete(asset_multilocation),
				fun: Fungibility::Fungible(to_balance),
			}),
			Box::new(destination),
			weight,
		);

		let used_gas = RuntimeHelper::<Runtime>::try_dispatch(
			Some(origin).into(),
			call,
			gasometer.remaining_gas()?,
		)?;

		gasometer.record_cost(used_gas)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}
}

// A wrapper to be able to implement here the evmData reader
pub struct MultiLocationWrapper(MultiLocation);

impl From<MultiLocation> for MultiLocationWrapper {
	fn from(location: MultiLocation) -> Self {
		MultiLocationWrapper(location)
	}
}

impl Into<MultiLocation> for MultiLocationWrapper {
	fn into(self) -> MultiLocation {
		self.0
	}
}

impl EvmData for MultiLocationWrapper {
	fn read(reader: &mut EvmDataReader) -> EvmResult<Self> {
		let num_parents = reader
			.read::<u8>()
			.map_err(|_| error("tried to parse array offset out of bounds"))?;

		let junctions: Vec<Bytes> = reader.read()?;

		Ok(MultiLocationWrapper(MultiLocation {
			parents: num_parents,
			interior: Junctions::from_encoded(junctions)?,
		}))
	}

	fn write(writer: &mut EvmDataWriter, value: Self) {
		EvmData::write(writer, U256::from(value.0.parents));
		EvmData::write(writer, Junctions::to_encoded(&value.0.interior));
	}
}

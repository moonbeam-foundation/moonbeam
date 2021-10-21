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

//! Precompile to xcm transactor runtime methods via the EVM

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
};
use xcm::latest::{AssetId, Fungibility, MultiAsset, MultiLocation};
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub type XBalanceOf<Runtime> = <Runtime as xcm_transactor::Config>::Balance;

pub type TransactorOf<Runtime> = <Runtime as xcm_transactor::Config>::Transactor;

#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq, num_enum::TryFromPrimitive, num_enum::IntoPrimitive)]
pub enum Action {
	IndexToAccount = "index_to_account(uint16)",
	TransactThroughDerivative =
		"transact_through_derivative(uint8,uint16,(uint8,bytes[]),uint256,uint64,bytes)",
}

/// A precompile to wrap the functionality from xcm transactor
pub struct XcmTransactorWrapper<Runtime>(PhantomData<Runtime>);

impl<Runtime> Precompile for XcmTransactorWrapper<Runtime>
where
	Runtime: xcm_transactor::Config + pallet_evm::Config + frame_system::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::Call: From<xcm_transactor::Call<Runtime>>,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	XBalanceOf<Runtime>: TryFrom<U256> + Into<U256> + EvmData,
	TransactorOf<Runtime>: TryFrom<u8>,
	Runtime::AccountId: Into<H160>,
{
	fn execute(
		input: &[u8], //Reminder this is big-endian
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<PrecompileOutput, ExitError> {
		let (input, selector) = EvmDataReader::new_with_selector(input)?;

		match selector {
			// Check for accessor methods first. These return results immediately
			Action::IndexToAccount => Self::account_index(input, target_gas),
			Action::TransactThroughDerivative => {
				Self::transact_through_derivative(input, target_gas, context)
			}
		}
	}
}

impl<Runtime> XcmTransactorWrapper<Runtime>
where
	Runtime: xcm_transactor::Config + pallet_evm::Config + frame_system::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::Call: From<xcm_transactor::Call<Runtime>>,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	XBalanceOf<Runtime>: TryFrom<U256> + Into<U256> + EvmData,
	TransactorOf<Runtime>: TryFrom<u8>,
	Runtime::AccountId: Into<H160>,
{
	fn account_index(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Bound check
		input.expect_arguments(1)?;
		let index: u16 = input.read::<u16>()?;

		// fetch data from pallet
		let account: H160 = xcm_transactor::Pallet::<Runtime>::index_to_account(index)
			.ok_or(error("No index assigned"))?
			.into();

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(Address(account)).build(),
			logs: Default::default(),
		})
	}

	fn transact_through_derivative(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);

		// Bound check
		input.expect_arguments(2)?;
		let transactor: TransactorOf<Runtime> = input
			.read::<u8>()?
			.try_into()
			.map_err(|_| error("Non-existent transactor"))?;
		let index: u16 = input.read::<u16>()?;

		// read fee location
		// defined as a multiLocation. For now we are assuming these are concrete
		// fungible assets
		let fee_multilocation: MultiLocation = input.read::<MultiLocation>()?;
		input.expect_arguments(2)?;
		// read fee amount
		let fee_amount: U256 = input.read()?;
		let weight: u64 = input.read::<u64>()?;

		// inner call
		let inner_call = input.read::<Bytes>()?;

		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let to_balance = fee_amount
			.try_into()
			.map_err(|_| error("Amount is too large for provided balance type"))?;

		let call = xcm_transactor::Call::<Runtime>::transact_through_derivative {
			dest: transactor,
			index,
			fee: MultiAsset {
				id: AssetId::Concrete(fee_multilocation),
				fun: Fungibility::Fungible(to_balance),
			},
			dest_weight: weight.clone(),
			inner_call: inner_call.as_bytes().to_vec(),
			dispatch_weight: weight,
		};

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

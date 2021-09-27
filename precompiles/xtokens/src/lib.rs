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
use frame_support::{
	dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
	ensure,
};
use pallet_evm::{AddressMapping, Precompile};
use precompile_utils::{
	error, Address, Bytes, EvmData, EvmDataReader, EvmResult, Gasometer, RuntimeHelper,
};

use sp_core::{H160, U256};
use sp_runtime::traits::Convert;
use sp_std::{
	convert::{TryFrom, TryInto},
	fmt::Debug,
	marker::PhantomData,
	vec,
	vec::Vec,
};

use xcm::v0::{Junction, MultiLocation, NetworkId};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub type BalanceOf<Runtime> = <Runtime as orml_xtokens::Config>::Balance;

#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq, num_enum::TryFromPrimitive)]
enum Action {
	Transfer = "transfer(address, u256, bytes, u64)",
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
	<Runtime as orml_xtokens::Config>::CurrencyId: From<Runtime::AccountId>,
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
	<Runtime as orml_xtokens::Config>::CurrencyId: From<Runtime::AccountId>,
{
	// The accessors are first. They directly return their result.
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

		let multilocation: Vec<Bytes> = input.read()?;

		let destination: MultiLocation =
			convert_encoded_multilocation_into_multilocation(multilocation)?;

		// Bound check
		input.expect_arguments(1)?;
		let weight: u64 = input.read::<u64>()?;

		let to_account = Runtime::AddressMapping::into_account_id(to_address);
		let to_currency_id: <Runtime as orml_xtokens::Config>::CurrencyId = to_account.into();

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
			exit_status: ExitSucceed::Stopped,
			cost: Default::default(),
			output: Default::default(),
			logs: Default::default(),
		})
	}
}

fn convert_encoded_multilocation_into_multilocation(
	encoded_multilocation: Vec<Bytes>,
) -> Result<MultiLocation, ExitError> {
	match encoded_multilocation.len() {
		0 => Ok(MultiLocation::Null),
		1 => {
			let first_junction =
				convert_encoded_junction_to_junction(encoded_multilocation[0].clone().into())?;
			Ok(MultiLocation::X1(first_junction))
		}

		2 => {
			let first_junction =
				convert_encoded_junction_to_junction(encoded_multilocation[0].clone().into())?;
			let second_junction =
				convert_encoded_junction_to_junction(encoded_multilocation[1].clone().into())?;

			Ok(MultiLocation::X2(first_junction, second_junction))
		}
		3 => {
			let first_junction =
				convert_encoded_junction_to_junction(encoded_multilocation[0].clone().into())?;
			let second_junction =
				convert_encoded_junction_to_junction(encoded_multilocation[1].clone().into())?;
			let third_junction =
				convert_encoded_junction_to_junction(encoded_multilocation[2].clone().into())?;

			Ok(MultiLocation::X3(
				first_junction,
				second_junction,
				third_junction,
			))
		}
		4 => {
			let first_junction =
				convert_encoded_junction_to_junction(encoded_multilocation[0].clone().into())?;
			let second_junction =
				convert_encoded_junction_to_junction(encoded_multilocation[1].clone().into())?;
			let third_junction =
				convert_encoded_junction_to_junction(encoded_multilocation[2].clone().into())?;
			let fourth_junction =
				convert_encoded_junction_to_junction(encoded_multilocation[3].clone().into())?;

			Ok(MultiLocation::X4(
				first_junction,
				second_junction,
				third_junction,
				fourth_junction,
			))
		}
		_ => Err(error("Provided more than 9 arguments for multilocation")),
	}
}

fn convert_encoded_junction_to_junction(
	mut encoded_junction: Vec<u8>,
) -> Result<Junction, ExitError> {
	ensure!(
		encoded_junction.len() > 0,
		error("Junctions cannot be emptyt")
	);
	let extra_data = encoded_junction.split_off(1);

	match encoded_junction[0] {
		0 => Ok(Junction::Parent),
		1 => {
			ensure!(
				extra_data.len() >= 4,
				error("Parachain Junction needs to specify u32 paraId")
			);
			let mut data: [u8; 4] = Default::default();
			data.copy_from_slice(&extra_data[0..4]);
			let para_id = u32::from_be_bytes(data);
			Ok(Junction::Parachain(para_id))
		}
		2 => {
			ensure!(
				extra_data.len() >= 32,
				error("AccountKey32 Junction needs to specify 32 byte id")
			);
			let mut data: [u8; 32] = Default::default();
			data.copy_from_slice(&extra_data[0..32]);
			Ok(Junction::AccountId32 {
				network: NetworkId::Any,
				id: data,
			})
		}
		3 => {
			ensure!(
				extra_data.len() >= 8,
				error("AccountIndex64 Junction needs to specify u64 index")
			);
			let mut data: [u8; 8] = Default::default();
			data.copy_from_slice(&extra_data[0..8]);

			Ok(Junction::AccountIndex64 {
				network: NetworkId::Any,
				index: u64::from_be_bytes(data),
			})
		}
		4 => {
			ensure!(
				extra_data.len() >= 20,
				error("AccountKey20 Junction needs to specify 20 bytes key")
			);
			let mut data: [u8; 20] = Default::default();
			data.copy_from_slice(&extra_data[0..20]);

			Ok(Junction::AccountKey20 {
				network: NetworkId::Any,
				key: data,
			})
		}
		5 => {
			ensure!(
				extra_data.len() >= 1,
				error("PalletInstance Junction needs to specify one byte instance id")
			);
			Ok(Junction::PalletInstance(extra_data[0]))
		}
		6 => {
			ensure!(
				extra_data.len() >= 16,
				error("GeneralIndex Junction needs to specify 16 bytes of index id")
			);
			let mut data: [u8; 16] = Default::default();
			data.copy_from_slice(&extra_data[0..16]);
			Ok(Junction::GeneralIndex {
				id: u128::from_be_bytes(data),
			})
		}
		7 => Ok(Junction::GeneralKey(extra_data)),
		8 => Ok(Junction::OnlyChild),
		_ => Err(error("Parachain Junction needs to specify u32 paraId")),
	}
}

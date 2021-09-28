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
use sp_std::{
	convert::{TryFrom, TryInto},
	fmt::Debug,
	marker::PhantomData,
	vec::Vec,
};

use xcm::v0::{Junction, MultiLocation, NetworkId, MultiAsset};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub type BalanceOf<Runtime> = <Runtime as orml_xtokens::Config>::Balance;

#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq, num_enum::TryFromPrimitive, num_enum::IntoPrimitive)]
enum Action {
	Transfer = "transfer(address, u256, bytes[], u64)",
	TransferMultiAsset = "transfer_multiasset(bytes[], u256, bytes[], u64)",
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
	Runtime::AccountId: Into<Option<<Runtime as orml_xtokens::Config>::CurrencyId>>,
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
	Runtime::AccountId: Into<Option<<Runtime as orml_xtokens::Config>::CurrencyId>>,
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

		let multilocation: Vec<Bytes> = input.read()?;

		let destination: MultiLocation =
			convert_encoded_multilocation_into_multilocation(multilocation)?;

		// Bound check
		input.expect_arguments(1)?;
		let weight: u64 = input.read::<u64>()?;

		let to_account = Runtime::AddressMapping::into_account_id(to_address);
		let to_currency_id: <Runtime as orml_xtokens::Config>::CurrencyId = to_account.into().ok_or(error("cannot convert into currency id"))?;

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

		// Read the asset multilocation
		let asset: Vec<Bytes> = input.read()?;
		let asset_multilocation: MultiLocation =
			convert_encoded_multilocation_into_multilocation(asset)?;

		// Bound check
		input.expect_arguments(1)?;
		let amount: U256 = input.read()?;

		// read destination
		let multilocation: Vec<Bytes> = input.read()?;

		let destination: MultiLocation =
			convert_encoded_multilocation_into_multilocation(multilocation)?;

		// Bound check
		input.expect_arguments(1)?;
		let weight: u64 = input.read::<u64>()?;

		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let to_balance = amount
			.try_into()
			.map_err(|_| error("Amount is too large for provided balance type"))?;

		let call = orml_xtokens::Call::<Runtime>::transfer_multiasset(
			Box::new(MultiAsset::ConcreteFungible {
				id: asset_multilocation,
				amount: to_balance
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

pub(crate) fn convert_encoded_multilocation_into_multilocation(
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

pub(crate) fn convert_encoded_junction_to_junction(
	mut encoded_junction: Vec<u8>,
) -> Result<Junction, ExitError> {
	ensure!(
		encoded_junction.len() > 0,
		error("Junctions cannot be emptyt")
	);
	let mut encoded_junction = EvmDataReader::new(&encoded_junction);

	let enum_selector = encoded_junction.read_raw_bytes(1)?;

	match enum_selector[0] {
		0 => Ok(Junction::Parent),
		1 => {
			let mut data: [u8; 4] = Default::default();
			data.copy_from_slice(&encoded_junction.read_raw_bytes(4)?);
			let para_id = u32::from_be_bytes(data);
			Ok(Junction::Parachain(para_id))
		}
		2 => {			
			let mut account: [u8; 32] = Default::default();
			account.copy_from_slice(&encoded_junction.read_raw_bytes(32)?);

			Ok(Junction::AccountId32 {
				network: convert_encoded_network_id(encoded_junction.read_till_end()?.to_vec())?,
				id: account,
			})
		}
		3 => {
			let mut index: [u8; 8] = Default::default();
			index.copy_from_slice(&encoded_junction.read_raw_bytes(8)?);
			// Now we read the network
			Ok(Junction::AccountIndex64 {
				network: convert_encoded_network_id(encoded_junction.read_till_end()?.to_vec())?,
				index: u64::from_be_bytes(index),
			})
		}
		4 => {
			let mut account: [u8; 20] = Default::default();
			account.copy_from_slice(&encoded_junction.read_raw_bytes(20)?);

			Ok(Junction::AccountKey20 {
				network: convert_encoded_network_id(encoded_junction.read_till_end()?.to_vec())?,
				key: account,
			})
		}
		5 => {
			Ok(Junction::PalletInstance(encoded_junction.read_raw_bytes(1)?[0]))
		}
		6 => {
			let mut general_index: [u8; 16] = Default::default();
			general_index.copy_from_slice(&encoded_junction.read_raw_bytes(16)?);
			Ok(Junction::GeneralIndex {
				id: u128::from_be_bytes(general_index),
			})
		}
		7 => Ok(Junction::GeneralKey(encoded_junction.read_till_end()?.to_vec())),
		8 => Ok(Junction::OnlyChild),
		_ => Err(error("No selector for this")),
	}
}

pub(crate) fn convert_encoded_network_id(
	mut encoded_network_id: Vec<u8>,
) -> Result<NetworkId, ExitError> {
	ensure!(
		encoded_network_id.len() > 0,
		error("Junctions cannot be empty")
	);
	let extra_data = encoded_network_id.split_off(1);

	match encoded_network_id[0] {
		0 => Ok(NetworkId::Any),
		1 => {
			Ok(NetworkId::Named(extra_data))
		}
		2 => Ok(NetworkId::Polkadot),
		3 => Ok(NetworkId::Kusama),
		_ => Err(error("Non-valid Network Id"))
	}
}

pub(crate) fn convert_to_encoded_network_id(
	network_id: NetworkId,
) -> Vec<u8> {

	let mut encoded: Vec<u8> = Vec::new();
	match network_id {
		NetworkId::Any => {
			encoded.push(0u8);
			encoded
		},
		NetworkId::Named(mut name) => {
			encoded.push(1u8);
			encoded.append(&mut name);
			encoded
		},
		NetworkId::Polkadot => {
			encoded.push(2u8);
			encoded
		},
		NetworkId::Kusama => {
			encoded.push(3u8);
			encoded
		},
	}
}

pub(crate) fn convert_to_encoded_junction(
	junction: Junction,
) -> Vec<u8> {

	let mut encoded: Vec<u8> = Vec::new();
	match junction {
		Junction::Parent => {
			encoded.push(0u8);
			encoded
		},
		Junction::Parachain(para_id) => {
			encoded.push(1u8);
			encoded.append(&mut para_id.to_be_bytes().to_vec());
			encoded
		},
		Junction::AccountId32{network, id} => {
			encoded.push(2u8);
			encoded.append(&mut id.to_vec());
			encoded.append(&mut convert_to_encoded_network_id(network));
			encoded
		},
		Junction::AccountIndex64 {network, index} => {
			encoded.push(3u8);
			encoded.append(&mut index.to_be_bytes().to_vec());
			encoded.append(&mut convert_to_encoded_network_id(network));
			encoded
		},
		Junction::AccountKey20{network, key} => {
			encoded.push(4u8);
			encoded.append(&mut key.to_vec());
			encoded.append(&mut convert_to_encoded_network_id(network));
			encoded
		},
		Junction::PalletInstance(intance) => {
			encoded.push(5u8);
			encoded.append(&mut intance.to_be_bytes().to_vec());
			encoded
		},
		Junction::GeneralIndex{id} => {
			encoded.push(6u8);
			encoded.append(&mut id.to_be_bytes().to_vec());
			encoded
		},
		Junction::GeneralKey(mut key) => {
			encoded.push(7u8);
			encoded.append(&mut key);
			encoded
		},
		Junction::OnlyChild => {
			encoded.push(8u8);
			encoded
		},
		_ => todo!()
	}
}

pub(crate) fn convert_to_encoded_multilocation(
	multilocation: MultiLocation,
) -> Vec<Vec<u8>> {

	let encoded: Vec<Vec<u8>> = multilocation.iter().map(|junction| {
		convert_to_encoded_junction(junction.clone())
	}).collect();
	encoded
}

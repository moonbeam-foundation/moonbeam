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

#![cfg_attr(not(feature = "std"), no_std)]

use evm::{executor::PrecompileOutput, Context, ExitError, ExitSucceed};
use frame_support::{
	dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
	storage::types::StorageDoubleMap,
	traits::{Get, StorageInstance, StoredMap},
	Blake2_128Concat,
};
use pallet_balances::pallet::{
	Instance1, Instance10, Instance11, Instance12, Instance13, Instance14, Instance15, Instance16,
	Instance2, Instance3, Instance4, Instance5, Instance6, Instance7, Instance8, Instance9,
};
use pallet_evm::{AddressMapping, GasWeightMapping, Precompile};
use sp_core::{H160, U256};
use sp_std::marker::PhantomData;

pub trait InstanceToPrefix {
	type ApprovesPrefix;
}

macro_rules! impl_prefix {
	($prefix:ident, $instance:ty, $name:literal) => {
		pub struct $prefix;

		impl StorageInstance for $prefix {
			const STORAGE_PREFIX: &'static str = "Approves";

			fn pallet_prefix() -> &'static str {
				$name
			}
		}

		impl InstanceToPrefix for $instance {
			type ApprovesPrefix = $prefix;
		}
	};
}

impl_prefix!(ApprovesPrefix0, (), "Erc20Instance0Balances");
impl_prefix!(ApprovesPrefix1, Instance1, "Erc20Instance1Balances");
impl_prefix!(ApprovesPrefix2, Instance2, "Erc20Instance2Balances");
impl_prefix!(ApprovesPrefix3, Instance3, "Erc20Instance3Balances");
impl_prefix!(ApprovesPrefix4, Instance4, "Erc20Instance4Balances");
impl_prefix!(ApprovesPrefix5, Instance5, "Erc20Instance5Balances");
impl_prefix!(ApprovesPrefix6, Instance6, "Erc20Instance6Balances");
impl_prefix!(ApprovesPrefix7, Instance7, "Erc20Instance7Balances");
impl_prefix!(ApprovesPrefix8, Instance8, "Erc20Instance8Balances");
impl_prefix!(ApprovesPrefix9, Instance9, "Erc20Instance9Balances");
impl_prefix!(ApprovesPrefix10, Instance10, "Erc20Instance10Balances");
impl_prefix!(ApprovesPrefix11, Instance11, "Erc20Instance11Balances");
impl_prefix!(ApprovesPrefix12, Instance12, "Erc20Instance12Balances");
impl_prefix!(ApprovesPrefix13, Instance13, "Erc20Instance13Balances");
impl_prefix!(ApprovesPrefix14, Instance14, "Erc20Instance14Balances");
impl_prefix!(ApprovesPrefix15, Instance15, "Erc20Instance15Balances");
impl_prefix!(ApprovesPrefix16, Instance16, "Erc20Instance16Balances");

pub type BalanceOf<Runtime, Instance = ()> =
	<Runtime as pallet_balances::Config<Instance>>::Balance;

/// (Owner => Allowed => Amount)
pub type ApprovesStorage<Runtime, Instance> = StorageDoubleMap<
	<Instance as InstanceToPrefix>::ApprovesPrefix,
	Blake2_128Concat,
	<Runtime as frame_system::Config>::AccountId,
	Blake2_128Concat,
	<Runtime as frame_system::Config>::AccountId,
	<Runtime as pallet_balances::Config<Instance>>::Balance,
>;

pub struct Erc20BalancesWrapper<Runtime, Instance: 'static = ()>(PhantomData<(Runtime, Instance)>);

impl<Runtime, Instance> Precompile for Erc20BalancesWrapper<Runtime, Instance>
where
	Runtime: pallet_balances::Config<Instance> + pallet_evm::Config,
	Runtime::AccountId: From<H160>,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::Call: From<pallet_balances::Call<Runtime, Instance>>,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Instance: InstanceToPrefix + 'static,
	U256: From<BalanceOf<Runtime, Instance>>,
{
	fn execute(
		input: &[u8], //Reminder this is big-endian
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<PrecompileOutput, ExitError> {
		const SELECTOR_SIZE_BYTES: usize = 4;

		if input.len() < 4 {
			return Err(ExitError::Other("input length less than 4 bytes".into()));
		}

		let inner_call = match input[0..SELECTOR_SIZE_BYTES] {
			// Views
			[0x7c, 0x80, 0xaa, 0x9f] => return Self::total_supply(&input[SELECTOR_SIZE_BYTES..]),
			[0x70, 0xa0, 0x82, 0x31] => return Self::balance_of(&input[SELECTOR_SIZE_BYTES..]),
			[0xdd, 0x62, 0xed, 0x3e] => return Self::allowance(&input[SELECTOR_SIZE_BYTES..]),
			// Only affect this Precompile storage.
			[0x09, 0x5e, 0xa7, 0xb3] => return Self::approve(&input[SELECTOR_SIZE_BYTES..]),
			// Results in a Substrate call
			[0xa9, 0x05, 0x9c, 0xbb] => Self::transfer(&input[SELECTOR_SIZE_BYTES..])?,
			[0x0c, 0x41, 0xb0, 0x33] => Self::transfer_from(&input[SELECTOR_SIZE_BYTES..])?,
			// Fallback
			_ => {
				return Err(ExitError::Other(
					"No staking wrapper method at selector given selector".into(),
				))
			}
		};

		let outer_call: Runtime::Call = inner_call.into();
		let info = outer_call.get_dispatch_info();

		// Make sure enough gas
		if let Some(gas_limit) = target_gas {
			let required_gas = Runtime::GasWeightMapping::weight_to_gas(info.weight);
			if required_gas > gas_limit {
				return Err(ExitError::OutOfGas);
			}
		}
		// log::trace!(target: "staking-precompile", "Made it past gas check");

		// Dispatch that call
		let origin = Runtime::AddressMapping::into_account_id(context.caller);

		// log::trace!(target: "staking-precompile", "Gonna call with origin {:?}", origin);

		match outer_call.dispatch(Some(origin).into()) {
			Ok(post_info) => {
				let gas_used = Runtime::GasWeightMapping::weight_to_gas(
					post_info.actual_weight.unwrap_or(info.weight),
				);
				Ok(PrecompileOutput {
					exit_status: ExitSucceed::Stopped,
					cost: gas_used,
					output: Default::default(),
					logs: Default::default(),
				})
			}
			Err(_) => {
				// log::trace!(
				// 	target: "staking-precompile",
				// 	"Parachain staking call via evm failed {:?}",
				// 	e
				// );
				Err(ExitError::Other("ERC20 wrapper call via EVM failed".into()))
			}
		}
	}
}

impl<Runtime, Instance> Erc20BalancesWrapper<Runtime, Instance>
where
	Runtime: pallet_balances::Config<Instance> + pallet_evm::Config,
	Runtime::AccountId: From<H160>,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Instance: InstanceToPrefix + 'static,
	U256: From<BalanceOf<Runtime, Instance>>,
{
	fn total_supply(input: &[u8]) -> Result<PrecompileOutput, ExitError> {
		if !input.is_empty() {
			return Err(ExitError::Other("Incorrect input lenght".into()));
		}

		let amount = pallet_balances::Pallet::<Runtime, Instance>::total_issuance();

		let gas_consumed = <Runtime as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
			<Runtime as frame_system::Config>::DbWeight::get().read,
		);

		let mut output = [0u8; 32];
		U256::from(amount).to_big_endian(&mut output);

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gas_consumed,
			output: output.to_vec(),
			logs: Default::default(),
		})
	}

	fn balance_of(input: &[u8]) -> Result<PrecompileOutput, ExitError> {
		if input.len() != 32 {
			return Err(ExitError::Other("Incorrect input lenght".into()));
		}

		let address = H160::from_slice(&input[12..32]);

		let amount = pallet_balances::Pallet::<Runtime, Instance>::usable_balance(&address.into());

		let gas_consumed = <Runtime as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
			<Runtime as frame_system::Config>::DbWeight::get().read,
		);

		let mut output = [0u8; 32];
		U256::from(amount).to_big_endian(&mut output);

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gas_consumed,
			output: output.to_vec(),
			logs: Default::default(),
		})
	}

	fn allowance(_input: &[u8]) -> Result<PrecompileOutput, ExitError> {
		todo!()
	}

	fn approve(_input: &[u8]) -> Result<PrecompileOutput, ExitError> {
		todo!()
	}

	fn transfer(_input: &[u8]) -> Result<pallet_balances::Call<Runtime, Instance>, ExitError> {
		todo!()
	}

	fn transfer_from(_input: &[u8]) -> Result<pallet_balances::Call<Runtime, Instance>, ExitError> {
		todo!()
	}
}

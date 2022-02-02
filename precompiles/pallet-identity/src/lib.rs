use enumflags2::BitFlags;
use fp_evm::{Context, ExitSucceed, Precompile, PrecompileOutput};
use frame_support::{
	dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
	traits::tokens::currency::Currency,
};
use pallet_evm::AddressMapping;
use pallet_identity::{IdentityField, IdentityFields};
use precompile_utils::{
	Address, EvmData, EvmDataReader, EvmResult, FunctionModifier, Gasometer, RuntimeHelper,
};
use sp_core::{H160, U256};
use sp_std::{
	convert::{TryFrom, TryInto},
	marker::PhantomData,
};

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

pub mod data;
use data::*;

pub type CurrencyOf<Runtime> = <Runtime as pallet_identity::Config>::Currency;
pub type AccountIdOf<Runtime> = <Runtime as frame_system::Config>::AccountId;
pub type BalanceOf<Runtime> = <CurrencyOf<Runtime> as Currency<AccountIdOf<Runtime>>>::Balance;

#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq)]
pub enum Action {
	SetIdentity = "setIdentity(bytes)",
	SetSubs = "setSubs((address,bytes)[])",
	ClearIdentity = "clearIdentity()",
	RequestJudgement = "requestJudgement(uint256,uint256)",
	CancelRequest = "cancelRequest(uint256)",
	SetFee = "setFee(uint256,uint256)",
	SetAccountId = "setAccountId(uint256,address)",
	SetFields = "setFields(uint256,uint256)",
	ProvideJudgement = "provideJudgement(uint256,address,uint256)",
	AddSub = "addSub(address,bytes)",
	RenameSub = "renameSub(address,bytes)",
	RemoveSub = "removeSub(address)",
	QuitSub = "quitSub()",
}

/// A precompile to wrap pallet_identity.
pub struct IdentityWrapper<Runtime>(PhantomData<Runtime>);

impl<Runtime> Precompile for IdentityWrapper<Runtime>
where
	Runtime: pallet_identity::Config + pallet_evm::Config + frame_system::Config,
	Runtime::AccountId: From<H160>,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::Call: From<pallet_identity::Call<Runtime>>,
	BalanceOf<Runtime>: TryFrom<U256> + Into<U256> + EvmData,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
{
	fn execute(
		input: &[u8],
		target_gas: Option<u64>,
		context: &Context,
		is_static: bool,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);
		let gasometer = &mut gasometer;

		let (mut input, selector) = EvmDataReader::new_with_selector(gasometer, input)?;
		let input = &mut input;

		gasometer.check_function_modifier(context, is_static, FunctionModifier::NonPayable)?;

		match selector {
			Action::SetIdentity => Self::set_identity(input, gasometer, context),
			Action::SetSubs => Self::set_subs(input, gasometer, context),
			Action::ClearIdentity => Self::clear_identity(input, gasometer, context),
			Action::RequestJudgement => Self::request_judgement(input, gasometer, context),
			Action::CancelRequest => Self::cancel_request(input, gasometer, context),
			Action::SetFee => Self::set_fee(input, gasometer, context),
			Action::SetAccountId => Self::set_account_id(input, gasometer, context),
			_ => todo!(),
		}
	}
}

impl<Runtime> IdentityWrapper<Runtime>
where
	Runtime: pallet_identity::Config + pallet_evm::Config + frame_system::Config,
	Runtime::AccountId: From<H160>,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::Call: From<pallet_identity::Call<Runtime>>,
	BalanceOf<Runtime>: TryFrom<U256> + Into<U256> + EvmData,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
{
	fn set_identity(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		let info: Wrapped<IdentityInfo<Runtime::MaxAdditionalFields>> = input.read(gasometer)?;
		let info = info.0 .0;

		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = pallet_identity::Call::<Runtime>::set_identity {
			info: Box::new(info),
		};

		RuntimeHelper::<Runtime>::try_dispatch(Some(origin).into(), call, gasometer)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn set_subs(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		let subs: Vec<Sub> = input.read(gasometer)?;
		let subs: Vec<_> = subs
			.into_iter()
			.map(|x| {
				(
					Runtime::AddressMapping::into_account_id(x.sub_account),
					x.identity_data,
				)
			})
			.collect();

		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = pallet_identity::Call::<Runtime>::set_subs { subs };

		RuntimeHelper::<Runtime>::try_dispatch(Some(origin).into(), call, gasometer)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn clear_identity(
		_: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = pallet_identity::Call::<Runtime>::clear_identity {};

		RuntimeHelper::<Runtime>::try_dispatch(Some(origin).into(), call, gasometer)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn request_judgement(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		let reg_index: u32 = input.read(gasometer)?;
		let max_fee: U256 = input.read(gasometer)?;
		let max_fee = Self::u256_to_amount(gasometer, max_fee)?;

		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = pallet_identity::Call::<Runtime>::request_judgement { reg_index, max_fee };

		RuntimeHelper::<Runtime>::try_dispatch(Some(origin).into(), call, gasometer)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn cancel_request(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		let reg_index: u32 = input.read(gasometer)?;

		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = pallet_identity::Call::<Runtime>::cancel_request { reg_index };

		RuntimeHelper::<Runtime>::try_dispatch(Some(origin).into(), call, gasometer)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn set_fee(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		let index: u32 = input.read(gasometer)?;
		let fee: U256 = input.read(gasometer)?;
		let fee = Self::u256_to_amount(gasometer, fee)?;

		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = pallet_identity::Call::<Runtime>::set_fee { index, fee };

		RuntimeHelper::<Runtime>::try_dispatch(Some(origin).into(), call, gasometer)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn set_account_id(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		let index: u32 = input.read(gasometer)?;
		let new: Address = input.read(gasometer)?;
		let new = Runtime::AddressMapping::into_account_id(new.0);

		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = pallet_identity::Call::<Runtime>::set_account_id { index, new };

		RuntimeHelper::<Runtime>::try_dispatch(Some(origin).into(), call, gasometer)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn set_fields(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		let index: u32 = input.read(gasometer)?;
		let fields: u64 = input.read(gasometer)?;
		let fields: BitFlags<IdentityField> = BitFlags::from_bits(fields).unwrap();
		
		// Not possible yet in Substrate. Openned a PR.
		let fields = IdentityFields(fields);
		todo!()

		// let origin = Runtime::AddressMapping::into_account_id(context.caller);
		// let call = pallet_identity::Call::<Runtime>::set_account_id { index, new };

		// RuntimeHelper::<Runtime>::try_dispatch(Some(origin).into(), call, gasometer)?;

		// Ok(PrecompileOutput {
		// 	exit_status: ExitSucceed::Stopped,
		// 	cost: gasometer.used_gas(),
		// 	output: Default::default(),
		// 	logs: Default::default(),
		// })
	}

	fn u256_to_amount(gasometer: &mut Gasometer, value: U256) -> EvmResult<BalanceOf<Runtime>> {
		value
			.try_into()
			.map_err(|_| gasometer.revert("amount is too large for balance type"))
	}
}

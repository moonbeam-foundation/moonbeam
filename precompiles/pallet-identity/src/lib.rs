use fp_evm::{Context, Precompile, PrecompileOutput};
use frame_support::{
	dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
	traits::tokens::currency::Currency,
};
use pallet_evm::AddressMapping;
use precompile_utils::{EvmData, EvmDataReader, EvmResult, FunctionModifier, Gasometer};
use sp_core::{H160, U256};
use sp_std::{convert::TryFrom, marker::PhantomData};

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
}

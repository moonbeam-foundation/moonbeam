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

#[macro_export]
macro_rules! impl_evm_runner_precompile_or_eth_xcm {
	{} => {
		use fp_evm::{CallInfo, CallOrCreateInfo, Context, Transfer};
		use frame_support::dispatch::CallableCallFor;
		use pallet_evm::{Runner, RunnerError};
		use precompile_utils::{prelude::*, evm::handle::with_precompile_handle};
		use sp_core::U256;
		use sp_runtime::DispatchError;
		use sp_std::vec::Vec;
		use xcm_primitives::{EthereumXcmTransaction, EthereumXcmTransactionV2};

		pub struct EvmRunnerPrecompileOrEthXcm<CallDispatcher, Runtime>(
			core::marker::PhantomData<(CallDispatcher, Runtime)>,
		);

		impl<CallDispatcher, Runtime> Runner<Runtime>
			for EvmRunnerPrecompileOrEthXcm<CallDispatcher, Runtime>
		where
			CallDispatcher: xcm_executor::traits::CallDispatcher<RuntimeCall>,
			Runtime: pallet_evm::Config + pallet_ethereum_xcm::Config,
			Runtime::RuntimeOrigin: From<pallet_ethereum_xcm::RawOrigin>,
		{
			type Error = DispatchError;

			fn call(
				source: H160,
				target: H160,
				input: Vec<u8>,
				value: U256,
				gas_limit: u64,
				_max_fee_per_gas: Option<U256>,
				_max_priority_fee_per_gas: Option<U256>,
				_nonce: Option<U256>,
				access_list: Vec<(H160, Vec<H256>)>,
				_is_transactional: bool,
				_validate: bool,
				_weight_limit: Option<Weight>,
				_transaction_len: Option<u64>,
				_config: &fp_evm::Config,
			) -> Result<CallInfo, RunnerError<Self::Error>> {
				// The `with_precompile_handle` function will execute the closure (and return the
				// result in a Some) if and only if there is an available EVM context. Otherwise,
				// it will return None.
				if let Some((exit_reason, value)) = with_precompile_handle(|precompile_handle| {
					let transfer = if value.is_zero() {
						None
					} else {
						Some(Transfer {
							source,
							target,
							value,
						})
					};

					precompile_handle.call(
						target,
						transfer,
						input.clone(),
						Some(gas_limit),
						false,
						&Context {
							address: target,
							caller: source,
							apparent_value: value,
						},
					)
				}) {
					Ok(CallInfo {
						exit_reason,
						value,
						used_gas: fp_evm::UsedGas {
							standard: U256::default(),
							effective: U256::default(),
						},
						logs: Default::default(),
						weight_info: None,
					})
				} else {
					let xcm_transaction = EthereumXcmTransaction::V2(EthereumXcmTransactionV2 {
						gas_limit: gas_limit.into(),
						action: pallet_ethereum_xcm::TransactionAction::Call(target),
						value,
						input: input.try_into().map_err(|_| RunnerError {
							error: DispatchError::Exhausted,
							weight: Default::default(),
						})?,
						access_list: Some(access_list),
					});

					let mut execution_info: Option<CallOrCreateInfo> = None;
					pallet_ethereum::catch_exec_info(&mut execution_info, || {
						CallDispatcher::dispatch(
							RuntimeCall::EthereumXcm(pallet_ethereum_xcm::Call::transact { xcm_transaction }),
							RawOrigin::Signed(source.into()).into(),
						)
						.map_err(|DispatchErrorWithPostInfo { error, .. }| RunnerError {
							error,
							weight: Default::default(),
						})
					})?;

					if let Some(CallOrCreateInfo::Call(call_info))= execution_info {
						Ok(call_info)
					} else {
						// `execution_info` must have been filled in
						Err(RunnerError {
							error: DispatchError::Unavailable,
							weight: Default::default(),
						})
					}
				}
			}

			fn create(
				_source: H160,
				_init: Vec<u8>,
				_value: U256,
				_gas_limit: u64,
				_max_fee_per_gas: Option<U256>,
				_max_priority_fee_per_gas: Option<U256>,
				_nonce: Option<U256>,
				_access_list: Vec<(H160, Vec<H256>)>,
				_is_transactional: bool,
				_validate: bool,
				_weight_limit: Option<Weight>,
				_transaction_len: Option<u64>,
				_config: &fp_evm::Config,
			) -> Result<fp_evm::CreateInfo, RunnerError<Self::Error>> {
				unimplemented!()
			}

			fn create2(
				_source: H160,
				_init: Vec<u8>,
				_salt: H256,
				_value: U256,
				_gas_limit: u64,
				_max_fee_per_gas: Option<U256>,
				_max_priority_fee_per_gas: Option<U256>,
				_nonce: Option<U256>,
				_access_list: Vec<(H160, Vec<H256>)>,
				_is_transactional: bool,
				_validate: bool,
				_weight_limit: Option<Weight>,
				_transaction_len: Option<u64>,
				_config: &fp_evm::Config,
			) -> Result<fp_evm::CreateInfo, RunnerError<Self::Error>> {
				unimplemented!()
			}

			fn create_force_address(
				_source: H160,
				_init: Vec<u8>,
				_value: U256,
				_gas_limit: u64,
				_max_fee_per_gas: Option<U256>,
				_max_priority_fee_per_gas: Option<U256>,
				_nonce: Option<U256>,
				_access_list: Vec<(H160, Vec<H256>)>,
				_is_transactional: bool,
				_validate: bool,
				_weight_limit: Option<Weight>,
				_transaction_len: Option<u64>,
				_config: &fp_evm::Config,
				_force_address: H160,
			) -> Result<fp_evm::CreateInfo, RunnerError<Self::Error>> {
				unimplemented!()
			}

			fn validate(
				_source: H160,
				_target: Option<H160>,
				_input: Vec<u8>,
				_value: U256,
				_gas_limit: u64,
				_max_fee_per_gas: Option<U256>,
				_max_priority_fee_per_gas: Option<U256>,
				_nonce: Option<U256>,
				_access_list: Vec<(H160, Vec<H256>)>,
				_is_transactional: bool,
				_weight_limit: Option<Weight>,
				_transaction_len: Option<u64>,
				_evm_config: &fp_evm::Config,
			) -> Result<(), RunnerError<Self::Error>> {
				unimplemented!()
			}
		}

	}
}

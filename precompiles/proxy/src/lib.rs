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
#![feature(assert_matches)]

use evm::{ExitError, ExitReason};
use fp_evm::{Context, Log, PrecompileFailure, PrecompileHandle, Transfer};
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use pallet_evm::AddressMapping;
use pallet_proxy::Call as ProxyCall;
use pallet_proxy::Pallet as ProxyPallet;
use precompile_utils::data::Address;
use precompile_utils::prelude::*;
use sp_core::{H160, U256};
use sp_runtime::{
	codec::Decode,
	traits::{ConstU32, StaticLookup, Zero},
};
use sp_std::marker::PhantomData;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub const CALL_DATA_LIMIT: u32 = 2u32.pow(16);
pub const LOG_SUBCALL_SUCCEEDED: [u8; 32] = keccak256!("ProxiedCallSucceeded()");
pub const LOG_SUBCALL_FAILED: [u8; 32] = keccak256!("ProxiedCallFailed()");

type GetCallDataLimit = ConstU32<CALL_DATA_LIMIT>;

pub struct EvmSubCall {
	pub to: Address,
	pub value: U256,
	pub call_data: BoundedBytes<ConstU32<CALL_DATA_LIMIT>>,
}

/// Simple trait for providing a filter over a reference to some type, given an instance of itself.
pub trait EvmProxyFilter: Sized + Send + Sync {
	fn evm_proxy_filter(&self, _call: &EvmSubCall, _recipient_has_code: bool) -> bool {
		false
	}
}

pub fn log_subcall_succeeded(address: impl Into<H160>) -> Log {
	log1(address, LOG_SUBCALL_SUCCEEDED, EvmDataWriter::new().build())
}

pub fn log_subcall_failed(address: impl Into<H160>) -> Log {
	log1(address, LOG_SUBCALL_FAILED, EvmDataWriter::new().build())
}

/// A precompile to wrap the functionality from pallet-proxy.
pub struct ProxyPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> ProxyPrecompile<Runtime>
where
	Runtime: pallet_proxy::Config + pallet_evm::Config + frame_system::Config,
	<<Runtime as pallet_proxy::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin:
		From<Option<Runtime::AccountId>>,
	<Runtime as pallet_proxy::Config>::ProxyType: Decode + EvmProxyFilter,
	<Runtime as frame_system::Config>::RuntimeCall:
		Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<<Runtime as frame_system::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin:
		From<Option<Runtime::AccountId>>,
	<Runtime as frame_system::Config>::RuntimeCall: From<ProxyCall<Runtime>>,
{
	#[precompile::pre_check]
	fn pre_check(handle: &mut impl PrecompileHandle) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let caller_code = pallet_evm::Pallet::<Runtime>::account_codes(handle.context().caller);
		// Check that caller is not a smart contract s.t. no code is inserted into
		// pallet_evm::AccountCodes except if the caller is another precompile i.e. CallPermit
		if !(caller_code.is_empty() || &caller_code == &[0x60, 0x00, 0x60, 0x00, 0xfd]) {
			Err(revert("Proxy not callable by smart contracts"))
		} else {
			Ok(())
		}
	}

	/// Register a proxy account for the sender that is able to make calls on its behalf.
	/// The dispatch origin for this call must be Signed.
	///
	/// Parameters:
	/// * delegate: The account that the caller would like to make a proxy.
	/// * proxy_type: The permissions allowed for this proxy account.
	/// * delay: The announcement period required of the initial proxy. Will generally be zero.
	#[precompile::public("addProxy(address,uint8,uint32)")]
	fn add_proxy(
		handle: &mut impl PrecompileHandle,
		delegate: Address,
		proxy_type: u8,
		delay: u32,
	) -> EvmResult {
		let delegate = Runtime::AddressMapping::into_account_id(delegate.into());
		let proxy_type = Runtime::ProxyType::decode(&mut proxy_type.to_le_bytes().as_slice())
			.map_err(|_| {
				RevertReason::custom("Failed decoding value to ProxyType").in_field("proxyType")
			})?;
		let delay = delay.into();

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		// Disallow re-adding proxy via precompile to prevent re-entrancy.
		// See: https://github.com/PureStake/sr-moonbeam/issues/30
		// Note: It is also assumed that EVM calls are only allowed through `Origin::Root` and
		// filtered via CallFilter
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		if ProxyPallet::<Runtime>::proxies(&origin)
			.0
			.iter()
			.any(|pd| pd.delegate == delegate)
		{
			return Err(revert("Cannot add more than one proxy"));
		}

		let delegate: <Runtime::Lookup as StaticLookup>::Source =
			Runtime::Lookup::unlookup(delegate.clone());
		let call = ProxyCall::<Runtime>::add_proxy {
			delegate,
			proxy_type,
			delay,
		}
		.into();

		<RuntimeHelper<Runtime>>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	/// Unregister a proxy account for the sender.
	/// The dispatch origin for this call must be Signed.
	///
	/// Parameters:
	/// * delegate: The account that the caller would like to remove as a proxy.
	/// * proxy_type: The permissions currently enabled for the removed proxy account.
	/// * delay: The announcement period required of the initial proxy. Will generally be zero.
	#[precompile::public("removeProxy(address,uint8,uint32)")]
	fn remove_proxy(
		handle: &mut impl PrecompileHandle,
		delegate: Address,
		proxy_type: u8,
		delay: u32,
	) -> EvmResult {
		let delegate = Runtime::AddressMapping::into_account_id(delegate.into());
		let proxy_type = Runtime::ProxyType::decode(&mut proxy_type.to_le_bytes().as_slice())
			.map_err(|_| {
				RevertReason::custom("Failed decoding value to ProxyType").in_field("proxyType")
			})?;
		let delay = delay.into();

		let delegate: <Runtime::Lookup as StaticLookup>::Source =
			Runtime::Lookup::unlookup(delegate.clone());
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = ProxyCall::<Runtime>::remove_proxy {
			delegate,
			proxy_type,
			delay,
		}
		.into();

		<RuntimeHelper<Runtime>>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	/// Unregister all proxy accounts for the sender.
	/// The dispatch origin for this call must be Signed.
	/// WARNING: This may be called on accounts created by anonymous, however if done, then the
	/// unreserved fees will be inaccessible. All access to this account will be lost.
	#[precompile::public("removeProxies()")]
	fn remove_proxies(handle: &mut impl PrecompileHandle) -> EvmResult {
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = ProxyCall::<Runtime>::remove_proxies {}.into();

		<RuntimeHelper<Runtime>>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	/// Dispatch the given subcall (`call_to`, `call_data`) from an account that the sender is
	/// authorised for through `add_proxy`.
	///
	/// Parameters:
	/// - `real`: The account that the proxy will make a call on behalf of.
	/// - `force_proxy_type`: Specify the exact proxy type to be used and checked for this call
	/// (optional parameter, use `255` for None).
	/// - `call_to`: Recipient of the call to be made by the `real` account.
	/// - `call_data`: Data of the call to be made by the `real` account.
	#[precompile::public("proxy(address,uint8,address,bytes)")]
	fn proxy(
		handle: &mut impl PrecompileHandle,
		real: Address,
		force_proxy_type: u8,
		call_to: Address,
		call_data: BoundedBytes<GetCallDataLimit>,
	) -> EvmResult {
		let real_account_id = Runtime::AddressMapping::into_account_id(real.clone().into());
		let who = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let force_proxy_type = match force_proxy_type {
			255 => None,
			proxy_type => Some(
				Runtime::ProxyType::decode(&mut proxy_type.to_le_bytes().as_slice()).map_err(
					|_| {
						RevertReason::custom("Failed decoding value to ProxyType")
							.in_field("forceProxyType")
					},
				)?,
			),
		};

		let evm_subcall = EvmSubCall {
			to: call_to,
			value: handle.context().apparent_value,
			call_data,
		};

		// read proxy
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let def = pallet_proxy::Pallet::<Runtime>::find_proxy(
			&real_account_id,
			&who,
			force_proxy_type.clone(),
		)
		.map_err(|_| RevertReason::custom("Not proxy").in_field("forceProxyType"))?;
		frame_support::ensure!(def.delay.is_zero(), revert("Unannounced"));

		// read subcall recipient code
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let recipient_has_code =
			pallet_evm::AccountCodes::<Runtime>::decode_len(evm_subcall.to.0).unwrap_or(0) > 0;

		frame_support::ensure!(
			def.proxy_type
				.evm_proxy_filter(&evm_subcall, recipient_has_code),
			revert("CallFiltered")
		);

		Self::inner_proxy(handle, real.0, evm_subcall)
	}

	/// Checks if the caller has an account proxied with a given proxy type
	///
	/// Parameters:
	/// * delegate: The account that the caller has maybe proxied
	/// * proxyType: The permissions allowed for the proxy
	/// * delay: The announcement period required of the initial proxy. Will generally be zero.
	#[precompile::public("isProxy(address,address,uint8,uint32)")]
	#[precompile::view]
	fn is_proxy(
		handle: &mut impl PrecompileHandle,
		real: Address,
		delegate: Address,
		proxy_type: u8,
		delay: u32,
	) -> EvmResult<bool> {
		let delegate = Runtime::AddressMapping::into_account_id(delegate.into());
		let proxy_type = Runtime::ProxyType::decode(&mut proxy_type.to_le_bytes().as_slice())
			.map_err(|_| {
				RevertReason::custom("Failed decoding value to ProxyType").in_field("proxyType")
			})?;
		let delay = delay.into();

		let real = Runtime::AddressMapping::into_account_id(real.into());

		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let is_proxy = ProxyPallet::<Runtime>::proxies(real)
			.0
			.iter()
			.any(|pd| pd.delegate == delegate && pd.proxy_type == proxy_type && pd.delay == delay);

		Ok(is_proxy)
	}

	fn inner_proxy(
		handle: &mut impl PrecompileHandle,
		real: H160,
		evm_subcall: EvmSubCall,
	) -> EvmResult {
		let EvmSubCall {
			to,
			value,
			call_data,
		} = evm_subcall;
		let address = to.0;

		let sub_context = Context {
			caller: real,
			address: address.clone(),
			apparent_value: value,
		};

		let transfer = if value.is_zero() {
			None
		} else {
			Some(Transfer {
				source: handle.context().caller,
				target: address.clone(),
				value,
			})
		};

		// Cost of log.
		let log_cost = log_subcall_failed(handle.code_address())
			.compute_cost()
			.map_err(|_| revert("Failed to compute log cost"))?;

		// We reserve enough gas to emit a final log and perform the subcall itself.
		// If not enough gas we stop there.
		let remaining_gas = handle.remaining_gas();
		let forwarded_gas = match remaining_gas.checked_sub(log_cost) {
			Some(remaining) => remaining,
			None => {
				let log = log_subcall_failed(handle.code_address());
				handle.record_log_costs(&[&log])?;
				log.record(handle)?;

				return Err(PrecompileFailure::Error {
					exit_status: ExitError::OutOfGas,
				});
			}
		};

		let (reason, output) = handle.call(
			address,
			transfer,
			call_data.into(),
			Some(forwarded_gas),
			false,
			&sub_context,
		);

		// Logs
		// We reserved enough gas so this should not OOG.
		match reason {
			ExitReason::Fatal(exit_status) => Err(PrecompileFailure::Fatal { exit_status }),
			ExitReason::Revert(exit_status) => {
				let log = log_subcall_failed(handle.code_address());
				handle.record_log_costs(&[&log])?;
				log.record(handle)?;

				Err(PrecompileFailure::Revert {
					exit_status,
					output,
				})
			}
			ExitReason::Error(exit_status) => {
				let log = log_subcall_failed(handle.code_address());
				handle.record_log_costs(&[&log])?;
				log.record(handle)?;

				Err(PrecompileFailure::Error { exit_status })
			}
			ExitReason::Succeed(_) => {
				let log = log_subcall_succeeded(handle.code_address());
				handle.record_log_costs(&[&log])?;
				log.record(handle)?;

				Ok(())
			}
		}
	}
}

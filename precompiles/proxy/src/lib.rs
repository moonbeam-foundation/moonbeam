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

use evm::ExitReason;
use fp_evm::{Context, PrecompileFailure, PrecompileHandle, Transfer};
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use pallet_evm::AddressMapping;
use pallet_proxy::Call as ProxyCall;
use pallet_proxy::Pallet as ProxyPallet;
use precompile_utils::prelude::*;
use precompile_utils::{
	data::{Address, String},
	precompile_set::SelectorFilter,
};
use sp_core::H160;
use sp_core::U256;
use sp_runtime::{
	codec::Decode,
	traits::{ConstU32, StaticLookup, Zero},
};
use sp_std::marker::PhantomData;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct OnlyIsProxy<Runtime>(PhantomData<Runtime>);

impl<Runtime> SelectorFilter for OnlyIsProxy<Runtime>
where
	Runtime: pallet_proxy::Config + pallet_evm::Config + frame_system::Config,
	<<Runtime as pallet_proxy::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin:
		From<Option<Runtime::AccountId>>,
	<Runtime as pallet_proxy::Config>::ProxyType: Decode + EvmProxyCallFilter,
	<Runtime as frame_system::Config>::RuntimeCall:
		Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<<Runtime as frame_system::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin:
		From<Option<Runtime::AccountId>>,
	<Runtime as frame_system::Config>::RuntimeCall: From<ProxyCall<Runtime>>,
{
	fn is_allowed(_caller: H160, selector: Option<u32>) -> bool {
		match selector {
			None => false,
			Some(selector) => {
				ProxyPrecompileCall::<Runtime>::is_proxy_selectors().contains(&selector)
			}
		}
	}

	fn description() -> String {
		"Allowed for all callers only for selector 'is_proxy'".into()
	}
}

#[derive(Debug)]
pub struct OnlyIsProxyAndProxy<Runtime>(PhantomData<Runtime>);

impl<Runtime> SelectorFilter for OnlyIsProxyAndProxy<Runtime>
where
	Runtime: pallet_proxy::Config + pallet_evm::Config + frame_system::Config,
	<<Runtime as pallet_proxy::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin:
		From<Option<Runtime::AccountId>>,
	<Runtime as pallet_proxy::Config>::ProxyType: Decode + EvmProxyCallFilter,
	<Runtime as frame_system::Config>::RuntimeCall:
		Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<<Runtime as frame_system::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin:
		From<Option<Runtime::AccountId>>,
	<Runtime as frame_system::Config>::RuntimeCall: From<ProxyCall<Runtime>>,
{
	fn is_allowed(_caller: H160, selector: Option<u32>) -> bool {
		match selector {
			None => false,
			Some(selector) => {
				ProxyPrecompileCall::<Runtime>::is_proxy_selectors().contains(&selector)
					|| ProxyPrecompileCall::<Runtime>::proxy_selectors().contains(&selector)
					|| ProxyPrecompileCall::<Runtime>::proxy_force_type_selectors()
						.contains(&selector)
			}
		}
	}

	fn description() -> String {
		"Allowed for all callers only for selectors 'is_proxy', 'proxy', 'proxy_force_type'".into()
	}
}

pub const CALL_DATA_LIMIT: u32 = 2u32.pow(16);

type GetCallDataLimit = ConstU32<CALL_DATA_LIMIT>;

pub struct EvmSubCall {
	pub to: Address,
	pub value: U256,
	pub call_data: BoundedBytes<ConstU32<CALL_DATA_LIMIT>>,
}

/// A trait to filter if an evm subcall is allowed to be executed by a proxy account.
/// This trait should be implemented by the `ProxyType` type configured in pallet proxy.
pub trait EvmProxyCallFilter: Sized + Send + Sync {
	/// If returns `false`, then the subcall will not be executed and the evm transaction will
	/// revert with error message "CallFiltered".
	fn is_evm_proxy_call_allowed(&self, _call: &EvmSubCall, _recipient_has_code: bool) -> bool {
		false
	}
}

/// A precompile to wrap the functionality from pallet-proxy.
pub struct ProxyPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> ProxyPrecompile<Runtime>
where
	Runtime: pallet_proxy::Config + pallet_evm::Config + frame_system::Config,
	<<Runtime as pallet_proxy::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin:
		From<Option<Runtime::AccountId>>,
	<Runtime as pallet_proxy::Config>::ProxyType: Decode + EvmProxyCallFilter,
	<Runtime as frame_system::Config>::RuntimeCall:
		Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<<Runtime as frame_system::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin:
		From<Option<Runtime::AccountId>>,
	<Runtime as frame_system::Config>::RuntimeCall: From<ProxyCall<Runtime>>,
{
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
	/// - `call_to`: Recipient of the call to be made by the `real` account.
	/// - `call_data`: Data of the call to be made by the `real` account.
	#[precompile::public("proxy(address,address,bytes)")]
	#[precompile::payable]
	fn proxy(
		handle: &mut impl PrecompileHandle,
		real: Address,
		call_to: Address,
		call_data: BoundedBytes<GetCallDataLimit>,
	) -> EvmResult {
		let evm_subcall = EvmSubCall {
			to: call_to,
			value: handle.context().apparent_value,
			call_data,
		};

		Self::inner_proxy(handle, real, None, evm_subcall)
	}

	/// Dispatch the given subcall (`call_to`, `call_data`) from an account that the sender is
	/// authorised for through `add_proxy`.
	///
	/// Parameters:
	/// - `real`: The account that the proxy will make a call on behalf of.
	/// - `force_proxy_type`: Specify the exact proxy type to be used and checked for this call.
	/// - `call_to`: Recipient of the call to be made by the `real` account.
	/// - `call_data`: Data of the call to be made by the `real` account.
	#[precompile::public("proxyForceType(address,uint8,address,bytes)")]
	#[precompile::public("proxy_force_type(address,uint8,address,bytes)")]
	#[precompile::payable]
	fn proxy_force_type(
		handle: &mut impl PrecompileHandle,
		real: Address,
		force_proxy_type: u8,
		call_to: Address,
		call_data: BoundedBytes<GetCallDataLimit>,
	) -> EvmResult {
		let proxy_type = Runtime::ProxyType::decode(&mut force_proxy_type.to_le_bytes().as_slice())
			.map_err(|_| {
				RevertReason::custom("Failed decoding value to ProxyType")
					.in_field("forceProxyType")
			})?;

		let evm_subcall = EvmSubCall {
			to: call_to,
			value: handle.context().apparent_value,
			call_data,
		};

		Self::inner_proxy(handle, real, Some(proxy_type), evm_subcall)
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
		real: Address,
		force_proxy_type: Option<<Runtime as pallet_proxy::Config>::ProxyType>,
		evm_subcall: EvmSubCall,
	) -> EvmResult {
		// Read proxy
		let real_account_id = Runtime::AddressMapping::into_account_id(real.clone().into());
		let who = Runtime::AddressMapping::into_account_id(handle.context().caller);
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let def =
			pallet_proxy::Pallet::<Runtime>::find_proxy(&real_account_id, &who, force_proxy_type)
				.map_err(|_| RevertReason::custom("Not proxy"))?;
		frame_support::ensure!(def.delay.is_zero(), revert("Unannounced"));

		// Read subcall recipient code
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let recipient_has_code =
			pallet_evm::AccountCodes::<Runtime>::decode_len(evm_subcall.to.0).unwrap_or(0) > 0;

		// Apply proxy type filter
		frame_support::ensure!(
			def.proxy_type
				.is_evm_proxy_call_allowed(&evm_subcall, recipient_has_code),
			revert("CallFiltered")
		);

		let EvmSubCall {
			to,
			value,
			call_data,
		} = evm_subcall;
		let address = to.0;

		let sub_context = Context {
			caller: real.0,
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

		let (reason, output) = handle.call(
			address,
			transfer,
			call_data.into(),
			Some(handle.remaining_gas()),
			false,
			&sub_context,
		);

		// Return subcall result
		match reason {
			ExitReason::Fatal(exit_status) => Err(PrecompileFailure::Fatal { exit_status }),
			ExitReason::Revert(exit_status) => Err(PrecompileFailure::Revert {
				exit_status,
				output,
			}),
			ExitReason::Error(exit_status) => Err(PrecompileFailure::Error { exit_status }),
			ExitReason::Succeed(_) => Ok(()),
		}
	}
}

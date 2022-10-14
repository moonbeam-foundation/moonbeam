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

use fp_evm::PrecompileHandle;
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use pallet_evm::AddressMapping;
use pallet_proxy::Call as ProxyCall;
use pallet_proxy::Pallet as ProxyPallet;
use precompile_utils::data::Address;
use precompile_utils::prelude::*;
use sp_runtime::{codec::Decode, traits::StaticLookup};
use sp_std::marker::PhantomData;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// A precompile to wrap the functionality from pallet-proxy.
pub struct ProxyPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> ProxyPrecompile<Runtime>
where
	Runtime: pallet_proxy::Config + pallet_evm::Config + frame_system::Config,
	<<Runtime as pallet_proxy::Config>::Call as Dispatchable>::Origin:
		From<Option<Runtime::AccountId>>,
	<Runtime as pallet_proxy::Config>::ProxyType: Decode,
	<Runtime as frame_system::Config>::Call:
		Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<<Runtime as frame_system::Config>::Call as Dispatchable>::Origin:
		From<Option<Runtime::AccountId>>,
	<Runtime as frame_system::Config>::Call: From<ProxyCall<Runtime>>,
{
	#[precompile::pre_check]
	fn pre_check(handle: &mut impl PrecompileHandle) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let caller_code = pallet_evm::Pallet::<Runtime>::account_codes(handle.context().caller);
		// Check that caller is not a smart contract s.t. no code is inserted into
		// pallet_evm::AccountCodes except if the caller is another precompile i.e. CallPermit
		if !(caller_code.is_empty() || &caller_code == &[0x60, 0x00, 0x60, 0x00, 0xfd]) {
			Err(revert("Batch not callable by smart contracts"))
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
}

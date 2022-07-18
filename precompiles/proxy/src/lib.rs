// Copyright 2019-2022 PureStake Inc.
// This file is 	part of Moonbeam.

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

use fp_evm::{Precompile, PrecompileFailure, PrecompileHandle, PrecompileOutput};
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use frame_support::sp_runtime::traits::Hash;
use pallet_evm::AddressMapping;
use pallet_proxy::Call as ProxyCall;
use parity_scale_codec::{Decode, DecodeLimit};
use precompile_utils::data::{Address, Bytes};
use precompile_utils::prelude::*;
use sp_core::{H160, H256};
use sp_std::{fmt::Debug, marker::PhantomData};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// Max recursion depth to enforce while decoding an extrinsic call.
const MAX_CALL_DECODE_DEPTH: u32 = 8;

#[generate_function_selector]
#[derive(Debug, PartialEq)]
pub enum Action {
	Proxy = "proxy(address,bytes[])",
	ProxyForceType = "proxyForceType(address,uint8,bytes[])",
	AddProxy = "addProxy(address,uint8,uint32)",
	RemoveProxy = "removeProxy(address,uint8,uint32)",
	RemoveProxies = "removeProxies()",
	Announce = "announce(address,bytes32)",
	RemoveAnnouncement = "removeAnnouncement(address,bytes32)",
	RejectAnnouncement = "rejectAnnouncement(address,bytes32)",
	ProxyAnnounced = "proxyAnnounced(address,address,bytes[])",
	ProxyForceTypeAnnounced = "proxyForceTypeAnnounced(address,address,uint8,bytes[])",
}

type DispatchCall<Runtime> = Result<
	(
		ProxyCall<Runtime>,
		<Runtime as frame_system::Config>::AccountId,
	),
	PrecompileFailure,
>;

/// A precompile to wrap the functionality from pallet-proxy.
pub struct ProxyWrapper<Runtime>(PhantomData<Runtime>);

impl<Runtime> Precompile for ProxyWrapper<Runtime>
where
	Runtime: pallet_proxy::Config + pallet_evm::Config + frame_system::Config,
	<<Runtime as pallet_proxy::Config>::Call as Dispatchable>::Origin:
		From<Option<Runtime::AccountId>>,
	<Runtime as pallet_proxy::Config>::ProxyType: TryFrom<u8>,
	<<Runtime as pallet_proxy::Config>::CallHasher as Hash>::Output: From<H256>,
	<Runtime as frame_system::Config>::Call:
		Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<<Runtime as frame_system::Config>::Call as Dispatchable>::Origin:
		From<Option<Runtime::AccountId>>,
	<Runtime as frame_system::Config>::Call: From<ProxyCall<Runtime>>,
	<Runtime as frame_system::Config>::Call: Decode,
{
	fn execute(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let selector = handle.read_selector()?;
		handle.check_function_modifier(FunctionModifier::NonPayable)?;

		let (call, origin) = match selector {
			Action::Proxy => Self::proxy(handle),
			Action::ProxyForceType => Self::proxy_force_type(handle),
			Action::AddProxy => Self::add_proxy(handle),
			Action::RemoveProxy => Self::remove_proxy(handle),
			Action::RemoveProxies => Self::remove_proxies(handle),
			Action::Announce => Self::announce(handle),
			Action::RemoveAnnouncement => Self::remove_announcement(handle),
			Action::RejectAnnouncement => Self::reject_announcement(handle),
			Action::ProxyAnnounced => Self::proxy_announced(handle),
			Action::ProxyForceTypeAnnounced => Self::proxy_force_type_announced(handle),
		}?;

		<RuntimeHelper<Runtime>>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}
}

impl<Runtime> ProxyWrapper<Runtime>
where
	Runtime: pallet_proxy::Config + pallet_evm::Config + frame_system::Config,
	<<Runtime as pallet_proxy::Config>::Call as Dispatchable>::Origin:
		From<Option<Runtime::AccountId>>,
	<Runtime as pallet_proxy::Config>::ProxyType: TryFrom<u8>,
	<<Runtime as pallet_proxy::Config>::CallHasher as Hash>::Output: From<H256>,
	<Runtime as frame_system::Config>::Call:
		Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<<Runtime as frame_system::Config>::Call as Dispatchable>::Origin:
		From<Option<Runtime::AccountId>>,
	<Runtime as frame_system::Config>::Call: From<ProxyCall<Runtime>>,
	<Runtime as frame_system::Config>::Call: Decode,
{
	fn decode_call(
		input: &mut EvmDataReader,
	) -> Result<Box<<Runtime as pallet_proxy::Config>::Call>, PrecompileFailure> {
		let wrapped_call: Vec<u8> = input.read::<Bytes>()?.into();
		<Runtime as frame_system::Config>::Call::decode_all_with_depth_limit(
			MAX_CALL_DECODE_DEPTH,
			&mut &wrapped_call[..],
		)
		.map(|c| Box::new(c.into()))
		.map_err(|_| revert("failed decoding wrapped call"))
	}

	/// Dispatch the given call from an account that the sender is authorised for through add_proxy.
	/// Removes any corresponding announcement(s).
	/// The dispatch origin for this call must be Signed.
	/// The most permissive proxy type is used to check for this call.
	///
	/// Parameters:
	/// * real: The account that the proxy will make a call on behalf of.
	/// * call: The call to be made by the real account.
	fn proxy(handle: &mut impl PrecompileHandle) -> DispatchCall<Runtime> {
		let mut input = handle.read_input()?;
		input.expect_arguments(2)?;

		let real: H160 = input.read::<Address>()?.into();
		let wrapped_call = Self::decode_call(&mut input)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = ProxyCall::<Runtime>::proxy {
			real: Runtime::AddressMapping::into_account_id(real),
			force_proxy_type: None,
			call: wrapped_call,
		}
		.into();

		Ok((call, origin))
	}

	/// Dispatch the given call from an account that the sender is authorised for through add_proxy.
	/// Removes any corresponding announcement(s).
	/// The dispatch origin for this call must be Signed.
	///
	/// Parameters:
	/// * real: The account that the proxy will make a call on behalf of.
	/// * force_proxy_type: Specify the exact proxy type to be used and checked for this call.
	/// * call: The call to be made by the real account.
	fn proxy_force_type(handle: &mut impl PrecompileHandle) -> DispatchCall<Runtime> {
		let mut input = handle.read_input()?;
		input.expect_arguments(3)?;

		let real: H160 = input.read::<Address>()?.into();
		let force_proxy_type = input
			.read::<u8>()?
			.try_into()
			.map_err(|_| revert("failed decoding proxy_type"))?;
		let wrapped_call = Self::decode_call(&mut input)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = ProxyCall::<Runtime>::proxy {
			real: Runtime::AddressMapping::into_account_id(real),
			force_proxy_type: Some(force_proxy_type),
			call: wrapped_call,
		}
		.into();

		Ok((call, origin))
	}

	/// Register a proxy account for the sender that is able to make calls on its behalf.
	/// The dispatch origin for this call must be Signed.
	///
	/// Parameters:
	/// * delegate: The account that the caller would like to make a proxy.
	/// * proxy_type: The permissions allowed for this proxy account.
	/// * delay: The announcement period required of the initial proxy. Will generally be zero.
	fn add_proxy(handle: &mut impl PrecompileHandle) -> DispatchCall<Runtime> {
		let mut input = handle.read_input()?;
		input.expect_arguments(3)?;

		let delegate: H160 = input.read::<Address>()?.into();
		let proxy_type = input
			.read::<u8>()?
			.try_into()
			.map_err(|_| revert("failed decoding proxy_type"))?;
		let delay = input.read::<u32>()?.into();

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = ProxyCall::<Runtime>::add_proxy {
			delegate: Runtime::AddressMapping::into_account_id(delegate),
			proxy_type,
			delay,
		}
		.into();

		Ok((call, origin))
	}

	/// Unregister a proxy account for the sender.
	/// The dispatch origin for this call must be Signed.
	///
	/// Parameters:
	/// * delegate: The account that the caller would like to remove as a proxy.
	/// * proxy_type: The permissions currently enabled for the removed proxy account.
	/// * delay: The announcement period required of the initial proxy. Will generally be zero.
	fn remove_proxy(handle: &mut impl PrecompileHandle) -> DispatchCall<Runtime> {
		let mut input = handle.read_input()?;
		input.expect_arguments(3)?;

		let delegate: H160 = input.read::<Address>()?.into();
		let proxy_type = input
			.read::<u8>()?
			.try_into()
			.map_err(|_| revert("failed decoding proxy_type"))?;
		let delay = input.read::<u32>()?.into();

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = ProxyCall::<Runtime>::remove_proxy {
			delegate: Runtime::AddressMapping::into_account_id(delegate),
			proxy_type,
			delay,
		}
		.into();

		Ok((call, origin))
	}

	/// Unregister all proxy accounts for the sender.
	/// The dispatch origin for this call must be Signed.
	/// WARNING: This may be called on accounts created by anonymous, however if done, then the
	/// unreserved fees will be inaccessible. All access to this account will be lost.
	fn remove_proxies(handle: &mut impl PrecompileHandle) -> DispatchCall<Runtime> {
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = ProxyCall::<Runtime>::remove_proxies {}.into();

		Ok((call, origin))
	}

	/// Publish the hash of a proxy-call that will be made in the future.
	/// This must be called some number of blocks before the corresponding proxy is attempted if the
	/// delay associated with the proxy relationship is greater than zero.
	/// No more than MaxPending announcements may be made at any one time.
	/// This will take a deposit of AnnouncementDepositFactor as well as AnnouncementDepositBase
	/// if there are no other pending announcements.
	/// The dispatch origin for this call must be Signed and a proxy of real.
	///
	/// Parameters:
	/// * real: The account that the proxy will make a call on behalf of.
	/// * call_hash: The hash of the call to be made by the real account.
	fn announce(handle: &mut impl PrecompileHandle) -> DispatchCall<Runtime> {
		let mut input = handle.read_input()?;
		input.expect_arguments(2)?;

		let real: H160 = input.read::<Address>()?.into();
		let call_hash = input.read::<H256>()?.into();

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = ProxyCall::<Runtime>::announce {
			real: Runtime::AddressMapping::into_account_id(real),
			call_hash,
		}
		.into();

		Ok((call, origin))
	}

	/// Remove a given announcement.
	/// May be called by a proxy account to remove a call they previously announced and return
	/// the deposit.
	/// The dispatch origin for this call must be Signed.
	///
	/// Parameters:
	/// * real: The account that the proxy will make a call on behalf of.
	/// * call_hash: The hash of the call to be made by the real account.
	fn remove_announcement(handle: &mut impl PrecompileHandle) -> DispatchCall<Runtime> {
		let mut input = handle.read_input()?;
		input.expect_arguments(2)?;

		let real: H160 = input.read::<Address>()?.into();
		let call_hash = input.read::<H256>()?.into();

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = ProxyCall::<Runtime>::remove_announcement {
			real: Runtime::AddressMapping::into_account_id(real),
			call_hash,
		}
		.into();

		Ok((call, origin))
	}

	/// Remove the given announcement of a delegate.
	/// May be called by a target (proxied) account to remove a call that one of their
	/// delegates (delegate) has announced they want to execute. The deposit is returned.
	/// The dispatch origin for this call must be Signed.
	///
	/// Parameters:
	/// * delegate: The account that previously announced the call.
	/// * call_hash: The hash of the call to be made.
	fn reject_announcement(handle: &mut impl PrecompileHandle) -> DispatchCall<Runtime> {
		let mut input = handle.read_input()?;
		input.expect_arguments(2)?;

		let delegate: H160 = input.read::<Address>()?.into();
		let call_hash = input.read::<H256>()?.into();

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = ProxyCall::<Runtime>::reject_announcement {
			delegate: Runtime::AddressMapping::into_account_id(delegate),
			call_hash,
		}
		.into();

		Ok((call, origin))
	}

	/// Dispatch the given call from an account that the sender is authorised for through add_proxy.
	/// Removes any corresponding announcement(s).
	/// The dispatch origin for this call must be Signed.
	///
	/// Parameters:
	/// * delegate: The account that previously announced the call.
	/// * real: The account that the proxy will make a call on behalf of.
	/// * call: The call to be made by the real account.
	fn proxy_announced(handle: &mut impl PrecompileHandle) -> DispatchCall<Runtime> {
		let mut input = handle.read_input()?;
		input.expect_arguments(3)?;

		let delegate: H160 = input.read::<Address>()?.into();
		let real: H160 = input.read::<Address>()?.into();
		let wrapped_call = Self::decode_call(&mut input)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = ProxyCall::<Runtime>::proxy_announced {
			delegate: Runtime::AddressMapping::into_account_id(delegate),
			real: Runtime::AddressMapping::into_account_id(real),
			force_proxy_type: None,
			call: wrapped_call,
		}
		.into();

		Ok((call, origin))
	}

	/// Dispatch the given call from an account that the sender is authorised for through add_proxy.
	/// Removes any corresponding announcement(s).
	/// The dispatch origin for this call must be Signed.
	///
	/// Parameters:
	/// * delegate: The account that previously announced the call.
	/// * real: The account that the proxy will make a call on behalf of.
	/// * force_proxy_type: Specify the exact proxy type to be used and checked for this call.
	/// * call: The call to be made by the real account.
	fn proxy_force_type_announced(handle: &mut impl PrecompileHandle) -> DispatchCall<Runtime> {
		let mut input = handle.read_input()?;
		input.expect_arguments(4)?;

		let delegate: H160 = input.read::<Address>()?.into();
		let real: H160 = input.read::<Address>()?.into();
		let force_proxy_type = input
			.read::<u8>()?
			.try_into()
			.map_err(|_| revert("failed decoding proxy_type"))?;
		let wrapped_call = Self::decode_call(&mut input)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = ProxyCall::<Runtime>::proxy_announced {
			delegate: Runtime::AddressMapping::into_account_id(delegate),
			real: Runtime::AddressMapping::into_account_id(real),
			force_proxy_type: Some(force_proxy_type),
			call: wrapped_call,
		}
		.into();

		Ok((call, origin))
	}
}

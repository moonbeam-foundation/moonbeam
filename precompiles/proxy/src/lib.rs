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
use precompile_utils::data::{Address, Bytes};
use precompile_utils::prelude::*;
use sp_core::{H160, H256};
use sp_std::{fmt::Debug, marker::PhantomData};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[generate_function_selector]
#[derive(Debug, PartialEq)]
pub enum Action {
	Proxy = "proxy(address,bytes[])",
	ProxyForceType = "proxyForceType(address,uint32,bytes[])",
	AddProxy = "addProxy(address,uint32,uint32)",
	RemoveProxy = "removeProxy(address,uint32,uint32)",
	RemoveProxies = "removeProxies()",
	Anonymous = "createAnonymous(uint32,uint32,uint16)",
	KillAnonymous = "killAnonymous(address,uint32,uint16,uint32,uint32)",
	Announce = "announce(address,bytes32)",
	RemoveAnnouncement = "removeAnnouncement(address,bytes32)",
	RejectAnnouncement = "rejectAnnouncement(address,bytes32)",
	ProxyAnnounced = "proxyAnnounced(address,address,uint64,bytes[])",
	ProxyForceTypeAnnounced = "proxyForceTypeAnnounced(address,address,uint64,bytes[])",
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
	<Runtime as pallet_proxy::Config>::Call: From<Vec<u8>>,
	<Runtime as pallet_proxy::Config>::ProxyType: From<u32>,
	<<Runtime as pallet_proxy::Config>::CallHasher as Hash>::Output: From<H256>,
	<Runtime as frame_system::Config>::Call:
		Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<<Runtime as frame_system::Config>::Call as Dispatchable>::Origin:
		From<Option<Runtime::AccountId>>,
	<Runtime as frame_system::Config>::Call: From<ProxyCall<Runtime>>,
{
	fn execute(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let selector = handle.read_selector()?;
		handle.check_function_modifier(match selector {
			_ => FunctionModifier::NonPayable,
		})?;

		let (call, origin) = match selector {
			Action::Proxy => Self::proxy(handle),
			Action::ProxyForceType => Self::proxy_force_type(handle),
			Action::AddProxy => Self::add_proxy(handle),
			Action::RemoveProxy => Self::remove_proxy(handle),
			Action::RemoveProxies => Self::remove_proxies(handle),
			Action::Anonymous => Self::anonymous(handle),
			Action::KillAnonymous => Self::kill_anonymous(handle),
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
	<Runtime as pallet_proxy::Config>::Call: From<Vec<u8>>,
	<Runtime as pallet_proxy::Config>::ProxyType: From<u32>,
	<<Runtime as pallet_proxy::Config>::CallHasher as Hash>::Output: From<H256>,
	<Runtime as frame_system::Config>::Call:
		Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<<Runtime as frame_system::Config>::Call as Dispatchable>::Origin:
		From<Option<Runtime::AccountId>>,
	<Runtime as frame_system::Config>::Call: From<ProxyCall<Runtime>>,
{
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
		let wrapped_call: Vec<u8> = input.read::<Bytes>()?.into();

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = ProxyCall::<Runtime>::proxy {
			real: Runtime::AddressMapping::into_account_id(real),
			force_proxy_type: None,
			call: Box::new(wrapped_call.into()),
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
		let force_proxy_type = input.read::<u32>()?.into();
		let wrapped_call: Vec<u8> = input.read::<Bytes>()?.into();

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = ProxyCall::<Runtime>::proxy {
			real: Runtime::AddressMapping::into_account_id(real),
			force_proxy_type: Some(force_proxy_type),
			call: Box::new(wrapped_call.into()),
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
		let proxy_type = input.read::<u32>()?.into();
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
		let proxy_type = input.read::<u32>()?.into();
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

	/// Spawn a fresh new account that is guaranteed to be otherwise inaccessible, and initialize
	///	it with a proxy of proxy_type for origin sender.
	/// Requires a Signed origin.
	/// * proxy_type: The type of the proxy that the sender will be registered as over the new
	///		account. This will almost always be the most permissive ProxyType possible to allow
	///		for maximum flexibility.
	/// * index: A disambiguation index, in case this is called multiple times in the same
	///		transaction (e.g. with utility::batch). Unless you're using batch you probably just
	///		want to use 0.
	/// * delay: The announcement period required of the initial proxy. Will generally be zero.
	/// Fails with Duplicate if this has already been called in this transaction, from the same
	///	sender, with the same parameters.
	/// Fails if there are insufficient funds to pay for deposit.
	fn anonymous(handle: &mut impl PrecompileHandle) -> DispatchCall<Runtime> {
		let mut input = handle.read_input()?;
		input.expect_arguments(3)?;

		let proxy_type = input.read::<u32>()?.into();
		let delay = input.read::<u32>()?.into();
		let index = input.read::<u16>()?.into();

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = ProxyCall::<Runtime>::anonymous {
			proxy_type,
			delay,
			index,
		}
		.into();

		Ok((call, origin))
	}

	/// Removes a previously spawned anonymous proxy.
	/// WARNING: All access to this account will be lost. Any funds held in it will be inaccessible.
	/// Requires a Signed origin, and the sender account must have been created by a call to
	///	anonymous with corresponding parameters.
	/// * spawner: The account that originally called anonymous to create this account.
	/// * proxy_type: The proxy type originally passed to anonymous.
	/// * index: The disambiguation index originally passed to anonymous. Probably 0.
	/// * height: The height of the chain when the call to anonymous was processed.
	/// * ext_index: The extrinsic index in which the call to anonymous was processed.
	/// Fails with `NoPermission` in case the caller is not a previously created anonymous account
	///	whose anonymous call has corresponding parameters.
	fn kill_anonymous(handle: &mut impl PrecompileHandle) -> DispatchCall<Runtime> {
		let mut input = handle.read_input()?;
		input.expect_arguments(5)?;

		let spawner: H160 = input.read::<Address>()?.into();
		let proxy_type = input.read::<u32>()?.into();
		let index = input.read::<u16>()?.into();
		let height = input.read::<u32>()?.into();
		let ext_index = input.read::<u32>()?.into();

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = ProxyCall::<Runtime>::kill_anonymous {
			spawner: Runtime::AddressMapping::into_account_id(spawner),
			proxy_type,
			index,
			height,
			ext_index,
		}
		.into();

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
		let wrapped_call: Vec<u8> = input.read::<Bytes>()?.into();

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = ProxyCall::<Runtime>::proxy_announced {
			delegate: Runtime::AddressMapping::into_account_id(delegate),
			real: Runtime::AddressMapping::into_account_id(real),
			force_proxy_type: None,
			call: Box::new(wrapped_call.into()),
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
		let force_proxy_type = input.read::<u32>()?.into();
		let wrapped_call: Vec<u8> = input.read::<Bytes>()?.into();

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = ProxyCall::<Runtime>::proxy_announced {
			delegate: Runtime::AddressMapping::into_account_id(delegate),
			real: Runtime::AddressMapping::into_account_id(real),
			force_proxy_type: Some(force_proxy_type),
			call: Box::new(wrapped_call.into()),
		}
		.into();

		Ok((call, origin))
	}
}

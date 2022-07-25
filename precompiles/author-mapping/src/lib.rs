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

//! Precompile to interact with pallet author mapping through an evm precompile.

#![cfg_attr(not(feature = "std"), no_std)]
#![feature(assert_matches)]

use fp_evm::{PrecompileHandle, PrecompileOutput};
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use pallet_author_mapping::Call as AuthorMappingCall;
use pallet_evm::{AddressMapping, Precompile};
use precompile_utils::prelude::*;
use sp_core::crypto::UncheckedFrom;
use sp_core::H256;
use sp_std::{fmt::Debug, marker::PhantomData};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[generate_function_selector]
#[derive(Debug, PartialEq)]
pub enum Action {
	AddAssociation = "add_association(bytes32)",
	UpdateAssociation = "update_association(bytes32,bytes32)",
	ClearAssociation = "clear_association(bytes32)",
	RemoveKeys = "remove_keys()",
	SetKeys = "set_keys(bytes)",
}

/// A precompile to wrap the functionality from pallet author mapping.
pub struct AuthorMappingWrapper<Runtime>(PhantomData<Runtime>);

impl<Runtime> Precompile for AuthorMappingWrapper<Runtime>
where
	Runtime: pallet_author_mapping::Config + pallet_evm::Config + frame_system::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<AuthorMappingCall<Runtime>>,
	Runtime::Hash: From<H256>,
{
	fn execute(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		log::trace!(target: "author-mapping-precompile", "In author mapping wrapper");

		let selector = handle.read_selector()?;

		handle.check_function_modifier(FunctionModifier::NonPayable)?;

		match selector {
			// Dispatchables
			Action::AddAssociation => Self::add_association(handle),
			Action::UpdateAssociation => Self::update_association(handle),
			Action::ClearAssociation => Self::clear_association(handle),
			Action::RemoveKeys => Self::remove_keys(handle),
			Action::SetKeys => Self::set_keys(handle),
		}
	}
}

impl<Runtime> AuthorMappingWrapper<Runtime>
where
	Runtime: pallet_author_mapping::Config + pallet_evm::Config + frame_system::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<AuthorMappingCall<Runtime>>,
	Runtime::Hash: From<H256>,
{
	// The dispatchable wrappers are next. They dispatch a Substrate inner Call.
	fn add_association(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;

		// Bound check
		input.expect_arguments(1)?;

		let nimbus_id = sp_core::sr25519::Public::unchecked_from(input.read::<H256>()?).into();

		log::trace!(
			target: "author-mapping-precompile",
			"Associating author id {:?}", nimbus_id
		);

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = AuthorMappingCall::<Runtime>::add_association { nimbus_id };

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	fn update_association(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;
		// Bound check
		input.expect_arguments(2)?;

		let old_nimbus_id = sp_core::sr25519::Public::unchecked_from(input.read::<H256>()?).into();
		let new_nimbus_id = sp_core::sr25519::Public::unchecked_from(input.read::<H256>()?).into();

		log::trace!(
			target: "author-mapping-precompile",
			"Updating author id {:?} for {:?}", old_nimbus_id, new_nimbus_id
		);

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = AuthorMappingCall::<Runtime>::update_association {
			old_nimbus_id,
			new_nimbus_id,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	fn clear_association(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;
		// Bound check
		input.expect_arguments(1)?;
		let nimbus_id = sp_core::sr25519::Public::unchecked_from(input.read::<H256>()?).into();

		log::trace!(
			target: "author-mapping-precompile",
			"Clearing author id {:?}", nimbus_id
		);

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = AuthorMappingCall::<Runtime>::clear_association { nimbus_id };

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	fn remove_keys(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		log::trace!(
			target: "author-mapping-precompile",
			"Removing keys"
		);

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = AuthorMappingCall::<Runtime>::remove_keys {};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	fn set_keys(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = AuthorMappingCall::<Runtime>::set_keys {
			// Taking all input minus selector (4 bytes)
			keys: handle.input()[4..].to_vec(),
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}
}

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

//! Precompile to interact with pallet democracy through an evm precompile.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(test, feature(assert_matches))]

use fp_evm::PrecompileHandle;
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use frame_support::ensure;
use frame_support::traits::{ConstU32};
use pallet_evm::AddressMapping;
use pallet_multisig::Call as MultisigCall;
use parity_scale_codec::DecodeLimit as _;
use precompile_utils::prelude::*;
use sp_core::{Get, H160,};
use sp_std::{
	marker::PhantomData,
	vec::Vec,
};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

type MultisigOf<Runtime> = pallet_multisig::Pallet<Runtime>;

type GetEncodedCallLimit = ConstU32<{ 2u32.pow(16) }>;
type DecodeLimit = ConstU32<8>;

pub struct MultisigPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
#[precompile::test_concrete_types(mock::Runtime)]
impl<Runtime> MultisigPrecompile<Runtime>
where
	Runtime: pallet_multisig::Config + pallet_evm::Config,
	<Runtime as frame_system::Config>::RuntimeCall:
		Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<<Runtime as pallet_multisig::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin:
		From<Option<Runtime::AccountId>>,
	<Runtime as pallet_multisig::Config>::RuntimeCall: From<MultisigCall<Runtime>>,
	Runtime::AccountId: Into<H160>,
{
	#[precompile::public("multiAccountId(address[],uint16)")]
	#[precompile::public("multi_account_id(address[],uint16)")]
	fn multi_account_id(
		handle: &mut impl PrecompileHandle,
		other_signatories: Vec<Address>,
		threshold: u16,
	) -> EvmResult<Address> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let sender = handle.context().caller;
		let signers_accounts = Self::ensure_sorted_and_insert(other_signatories, sender)?;

		//build the new multisig address
		let new_multi = MultisigOf::<Runtime>::multi_account_id(&signers_accounts, threshold);
		log::trace!(target: "multisig-precompile", "New multisig account is {:?}", new_multi);

		Ok(Address(new_multi.into()))
	}

	#[precompile::public("asMultiThreshold1(address[],bytes)")]
	#[precompile::public("as_multi_threshold_1(address[],bytes)")]
	fn as_multi_threshold_1(
		handle: &mut impl PrecompileHandle,
		other_signatories: Vec<Address>,
		call: BoundedBytes<GetEncodedCallLimit>,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Decode the incoming call
		let call: Vec<_> = call.into();
		let pallet_multisig_call: <Runtime as pallet_multisig::Config>::RuntimeCall =
			<Runtime as pallet_multisig::Config>::RuntimeCall::decode_with_depth_limit(
				DecodeLimit::get(),
				&mut &*call,
			)
			.map_err(|_| RevertReason::custom("Failed to decode call").in_field("call"))?
			.into();

		// Convert call to the expected type by pallet-multisig
		let call_to_dispatch = Box::new(pallet_multisig_call);

		// Convert other_signatories addresses into valid AccountIds
		let other_signatories: Vec<_> = other_signatories
			.into_iter()
			.map(|x| Runtime::AddressMapping::into_account_id(x.0))
			.collect();

		// Take sender's AccountId
		let caller = Runtime::AddressMapping::into_account_id(handle.context().caller);

		// Dispatch the call
		let origin = <Runtime as frame_system::Config>::RuntimeOrigin::from(Some(caller).into());
		ensure!(
			MultisigOf::<Runtime>::as_multi_threshold_1(
				origin,
				other_signatories,
				call_to_dispatch
			)
			.is_ok(),
			revert("error dispatching as_multi_threshold_1")
		);

		log::trace!(target: "multisig-precompile", "as_multi_threshold_1 dispatched");

		Ok(())
	}

	// Helper function
	fn ensure_sorted_and_insert(
		other_signatories: Vec<Address>,
		sender: H160,
	) -> EvmResult<Vec<<Runtime as frame_system::Config>::AccountId>> {
		//convert sender address into a valid AccountId
		let sender = Runtime::AddressMapping::into_account_id(sender);

		//convert the other sigantories addresses into valid AccountIds
		let mut signers_accounts: Vec<_> = other_signatories
			.into_iter()
			.map(|x| Runtime::AddressMapping::into_account_id(x.0))
			.collect();

		//ensure addresses are sorted
		let mut maybe_last = None;
		let mut index = 0;
		for item in signers_accounts.iter() {
			if let Some(last) = maybe_last {
				ensure!(last < item, revert("Signatories out of order"));
			}
			if item <= &sender {
				ensure!(item != &sender, revert("Sender in signatories"));
				index += 1;
			}
			maybe_last = Some(item);
		}

		//insert the sender at first position
		signers_accounts.insert(index, sender);
		Ok(signers_accounts)
	}
}

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
#![feature(assert_matches)]

use fp_evm::PrecompileHandle;
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use frame_support::ensure;
use frame_support::traits::{Currency};
use pallet_multisig::{
 Call as MultisigCall
};
use pallet_evm::AddressMapping;
use precompile_utils::prelude::*;
use sp_core::{H160, H256, U256};
use sp_std::{
	convert::{TryFrom, TryInto},
	fmt::Debug,
	marker::PhantomData,
	vec::Vec,
};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

type BalanceOf<Runtime> = <<Runtime as pallet_multisig::Config>::Currency as Currency<
	<Runtime as frame_system::Config>::AccountId,
>>::Balance;

type MultisigOf<Runtime> = pallet_multisig::Pallet<Runtime>;

//TODO: implement selector logs

pub struct MultisigPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
#[precompile::test_concrete_types(mock::Runtime)]
impl<Runtime> MultisigPrecompile<Runtime>
where
	Runtime: pallet_multisig::Config
		+ pallet_evm::Config
		+ frame_system::Config,
	BalanceOf<Runtime>: TryFrom<U256> + TryInto<u128> + Into<U256> + Debug + EvmData,
	<Runtime as frame_system::Config>::RuntimeCall:
		Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<<Runtime as frame_system::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin:
		From<Option<Runtime::AccountId>>,
	<Runtime as frame_system::Config>::RuntimeCall: From<MultisigCall<Runtime>>,
	Runtime::Hash: From<H256> + Into<H256>,
	Runtime::BlockNumber: Into<U256>,
	Runtime::AccountId: Into<H160>,
{
	
	#[precompile::public("multiAccountId(address[],uint16)")]
	#[precompile::public("multi_account_id(address[],uint16)")]
	fn multi_account_id(handle: &mut impl PrecompileHandle, other_signatories: Vec<Address>, threshold: u16) -> EvmResult<Address> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		
		//convert sender address into a valid AccountId
		let sender = Runtime::AddressMapping::into_account_id(handle.context().caller);

		//convert the other sigantories addresses to valid AccountIds
		let mut signers_accounts: Vec<_> = other_signatories.into_iter().map(|x| Runtime::AddressMapping::into_account_id(x.0)).collect();

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

		//build the new multisig address
		let new_multi = MultisigOf::<Runtime>::multi_account_id(&signers_accounts, threshold);
		log::trace!(target: "multisig-precompile", "New multisig account is {:?}", new_multi);

		Ok(Address(new_multi.into()))
	}
}
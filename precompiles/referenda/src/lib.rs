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
use frame_support::traits::{schedule::DispatchTime, OriginTrait};
use pallet_evm::AddressMapping;
use pallet_referenda::Call as ReferendaCall;
use precompile_utils::prelude::*;
use sp_core::H256;
use sp_std::marker::PhantomData;

// #[cfg(test)]
// mod mock;
// #[cfg(test)]
// mod tests;

/// A precompile to wrap the functionality from pallet-referenda.
pub struct ReferendaPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> ReferendaPrecompile<Runtime>
where
	Runtime: pallet_referenda::Config + pallet_evm::Config + frame_system::Config,
	<<Runtime as pallet_referenda::Config>::Call as Dispatchable>::Origin:
		From<Option<Runtime::AccountId>>,
	<<Runtime as frame_system::Config>::Origin as OriginTrait>::PalletsOrigin:
		From<pallet_governance_origins::Origin>,
	<Runtime as frame_system::Config>::Hash: TryFrom<H256>,
	<Runtime as frame_system::Config>::Call:
		Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<<Runtime as frame_system::Config>::Call as Dispatchable>::Origin:
		From<Option<Runtime::AccountId>>,
	<Runtime as frame_system::Config>::Call: From<ReferendaCall<Runtime>>,
{
	#[precompile::pre_check]
	fn pre_check(handle: &mut impl PrecompileHandle) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let caller_code = pallet_evm::Pallet::<Runtime>::account_codes(handle.context().caller);
		// Check that caller is not a smart contract s.t. no code is inserted into
		// pallet_evm::AccountCodes except if the caller is another precompile i.e. CallPermit
		// TODO: review if this is necessary
		if !(caller_code.is_empty() || &caller_code == &[0x60, 0x00, 0x60, 0x00, 0xfd]) {
			Err(revert("Referenda not callable by smart contracts"))
		} else {
			Ok(())
		}
	}

	/// Propose a referendum on a privileged action.
	///
	/// Parameters:
	/// * proposal_origin: The origin from which the proposal should be executed.
	/// * proposal_hash: Hash of the proposal preimage.
	/// * at: If true then AT block_number, else AFTER block_number
	/// * block_number: Inner block number for DispatchTime
	#[precompile::public("submit(uint8,bytes32,bool,uint32)")]
	fn submit(
		handle: &mut impl PrecompileHandle,
		proposal_origin: u8,
		proposal_hash: H256,
		at: bool,
		block_number: u32,
	) -> EvmResult {
		let proposal_origin: pallet_governance_origins::Origin = proposal_origin
			.try_into()
			.map_err(|_| revert("Origin does not exist for u8"))?;
		let proposal_hash: Runtime::Hash = proposal_hash
			.try_into()
			.map_err(|_| revert("Proposal hash input is not H256"))?;
		let enactment_moment: DispatchTime<Runtime::BlockNumber> = if at {
			DispatchTime::At(block_number.into())
		} else {
			DispatchTime::After(block_number.into())
		};

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let call = ReferendaCall::<Runtime>::submit {
			proposal_origin: Box::new(proposal_origin.into()),
			proposal_hash,
			enactment_moment,
		}
		.into();

		<RuntimeHelper<Runtime>>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	/// Post the Decision Deposit for a referendum.
	///
	/// Parameters:
	/// * index: The index of the submitted referendum whose Decision Deposit is yet to be posted.
	#[precompile::public("placeDecisionDeposit(uint32)")]
	fn place_decision_deposit(handle: &mut impl PrecompileHandle, index: u32) -> EvmResult {
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let call = ReferendaCall::<Runtime>::place_decision_deposit { index }.into();

		<RuntimeHelper<Runtime>>::try_dispatch(handle, Some(origin).into(), call)?;
		Ok(())
	}

	/// Refund the Decision Deposit for a closed referendum back to the depositor.
	///
	/// Parameters:
	/// * index: The index of a closed referendum whose Decision Deposit has not yet been refunded.
	#[precompile::public("refundDecisionDeposit(uint32)")]
	fn refund_decision_deposit(handle: &mut impl PrecompileHandle, index: u32) -> EvmResult {
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let call = ReferendaCall::<Runtime>::refund_decision_deposit { index }.into();

		<RuntimeHelper<Runtime>>::try_dispatch(handle, Some(origin).into(), call)?;
		Ok(())
	}
}

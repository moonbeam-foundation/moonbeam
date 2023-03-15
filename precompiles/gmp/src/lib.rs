// Copyright 2019-2023 PureStake Inc.
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

//! Precompile to receive GMP callbacks and forward to XCM

#![cfg_attr(not(feature = "std"), no_std)]

use evm::ExitReason;
use fp_evm::{Context, ExitRevert, PrecompileFailure, PrecompileHandle};
use frame_support::{
	codec::Decode,
	dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
	traits::ConstU32,
};
use pallet_evm::AddressMapping;
use precompile_utils::prelude::*;
use sp_core::{H160, U256};
use sp_std::{marker::PhantomData, str::FromStr, vec::Vec};
use types::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod types;

pub type SystemCallOf<Runtime> = <Runtime as frame_system::Config>::RuntimeCall;
pub const CALL_DATA_LIMIT: u32 = 2u32.pow(16);
type GetCallDataLimit = ConstU32<CALL_DATA_LIMIT>;

// Wormhole fn selectors
const PARSE_VM_SELECTOR: u32 = 0xa9e11893_u32; // parseVM(bytes)
const PARSE_AND_VERIFY_VM_SELECTOR: u32 = 0xc0fd8bde_u32; // parseAndVerifyVM(bytes)
const COMPLETE_TRANSFER_WITH_PAYLOAD_SELECTOR: u32 = 0xc0fd8bde_u32; // completeTransferWithPayload(bytes)

/// Gmp precompile.
#[derive(Debug, Clone)]
pub struct GmpPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> GmpPrecompile<Runtime>
where
	Runtime: pallet_evm::Config + frame_system::Config + pallet_xcm::Config,
	SystemCallOf<Runtime>: Dispatchable<PostInfo = PostDispatchInfo> + Decode + GetDispatchInfo,
	<<Runtime as frame_system::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin:
		From<Option<Runtime::AccountId>>,
	<Runtime as frame_system::Config>::RuntimeCall: From<pallet_xcm::Call<Runtime>>,
{
	#[precompile::public("wormholeTransferERC20(bytes)")]
	pub fn wormhole_transfer_erc20(
		handle: &mut impl PrecompileHandle,
		wormhole_vaa: BoundedBytes<GetCallDataLimit>,
	) -> EvmResult {
		log::warn!(target: "gmp-precompile", "wormhole_vaa: {:?}", wormhole_vaa.clone());

		// TODO: need to pull this from storage or config somewhere
		//
		// Moonbase core bridge: 0xa5B7D85a8f27dd7907dc8FdC21FA5657D5E2F901
		// Moonbase token bridge: 0xbc976D4b9D57E57c3cA52e1Fd136C45FF7955A96
		let wormhole = H160::from_str("0xa5B7D85a8f27dd7907dc8FdC21FA5657D5E2F901")
			.map_err(|_| RevertReason::custom("invalid wormhole contract address"))?;

		// TODO: need our own address (preferably without looking at storage)
		let this_contract = H160::from_str("0x0000000000000000000000000000000000000815")
			.map_err(|_| RevertReason::custom("invalid precompile address"))?;

		// Complete a "Contract Controlled Transfer" with the given Wormhole VAA.
		// We need to invoke Wormhole's completeTransferWithPayload function, passing it the VAA,
		// then use the returned payload to decide what to do.
		let sub_context = Context {
			caller: this_contract,
			address: wormhole,
			apparent_value: U256::zero(), // TODO: any reason to pass value on, or reject txns with value?
		};

		log::warn!(target: "gmp-precompile", "calling Wormhole completeTransferWithPayload on {}...", wormhole);
		let (reason, output) = handle.call(
			wormhole,
			None,
			EvmDataWriter::new_with_selector(COMPLETE_TRANSFER_WITH_PAYLOAD_SELECTOR)
				.write(wormhole_vaa)
				.build(),
			handle.gas_limit(), // TODO
			false,
			&sub_context,
		);

		log::warn!(target: "gmp-precompile", "reason: {:?}", reason);
		log::warn!(target: "gmp-precompile", "output: {:?}", output);

		match reason {
			ExitReason::Fatal(exit_status) => return Err(PrecompileFailure::Fatal { exit_status }),
			ExitReason::Revert(exit_status) => {
				return Err(PrecompileFailure::Revert {
					exit_status,
					output,
				})
			}
			ExitReason::Error(exit_status) => return Err(PrecompileFailure::Error { exit_status }),
			ExitReason::Succeed(_) => (),
		};

		// TODO: we should now have funds for this account, custodied by this precompile itself.
		//       next we need to see where the user wants to send them by inspecting the payload.
		//
		// TODO: Wormhole might have transfered unsupported tokens; we should handle this case
		//       gracefully (maybe that's as simple as reverting)
		let user_action = parse_user_action(&output).map_err(|e| PrecompileFailure::Revert {
			exit_status: ExitRevert::Reverted,
			output: e.into(),
		})?;
		let call = match user_action {
			VersionedUserAction::V1(action) => {
				// TODO: make XCM transfer here (use xtokens?)
				/*
				let xcm = "fixme";

				pallet_xcm::Call::<Runtime>::send {
					dest: Box::new(xcm::VersionedMultiLocation::V1(action.destination)),
					message: Box::new(xcm),
				}
				*/
			}
		};

		// TODO: proper origin
		// let origin = Runtime::AddressMapping::into_account_id(this_contract);
		// RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}
}

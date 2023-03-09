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

use evm::{ExitError, ExitReason};
use fp_evm::{Context, ExitRevert, Log, PrecompileFailure, PrecompileHandle, Transfer};
use frame_support::traits::ConstU32;
use precompile_utils::{costs::call_cost, prelude::*};
use sp_core::{H160, U256};
use sp_std::{iter::repeat, marker::PhantomData, str::FromStr, vec, vec::Vec};
use types::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod types;

pub const CALL_DATA_LIMIT: u32 = 2u32.pow(16);
type GetCallDataLimit = ConstU32<CALL_DATA_LIMIT>;

/// Gmp precompile.
#[derive(Debug, Clone)]
pub struct GmpPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> GmpPrecompile<Runtime>
where
	Runtime: pallet_evm::Config,
{
	#[precompile::public("wormholeTransferERC20(bytes)")]
	pub fn wormhole_transfer_erc20(
		handle: &mut impl PrecompileHandle,
		wormhole_vaa: BoundedBytes<GetCallDataLimit>,
	) -> EvmResult {
		// TODO: need to pull this from storage or config somewhere
		let wormhole = H160::from_str("FIXME")
			.map_err(|_| RevertReason::custom("invalid wormhole contract address"))?;

		// TODO: need our own address
		let this_contract = H160::from_str("").expect("fixme"); // TODO: need our own precompile address here

		// Complete a "Contract Controlled Transfer" with the given Wormhole VAA.
		// We need to invoke Wormhole's completeTransferWithPayload function, passing it the VAA,
		// then use the returned payload to decide what to do.
		let sub_context = Context {
			caller: this_contract,
			address: wormhole,
			apparent_value: U256::zero(), // TODO: any reason to pass value on, or reject txns with value?
		};

		// TODO: construct calldata suitable for wormhole contract
		let calldata = wormhole_vaa.into();

		let (reason, output) = handle.call(
			wormhole,
			None,
			calldata,
			handle.gas_limit(), // TODO
			false,
			&sub_context,
		);

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
			VersionedUserAction::v1(action) => {
				// TODO: make XCM transfer here (use xtokens?)
				let xcm = "fixme";

				pallet_xcm::Call::<Runtime>::send {
					dest: Box::new(action.destination),
					message: Box::new(xcm),
				}
			}
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}
}

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
use parity_scale_codec::DecodeLimit;
use precompile_utils::prelude::*;
use sp_core::{H160, U256};
use sp_std::boxed::Box;
use sp_std::{marker::PhantomData, str::FromStr, vec::Vec};
use types::*;
use xcm::opaque::latest::WeightLimit;
use xcm::VersionedMultiLocation;
use xcm_primitives::AccountIdToCurrencyId;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod types;

pub type SystemCallOf<Runtime> = <Runtime as frame_system::Config>::RuntimeCall;
pub type CurrencyIdOf<Runtime> = <Runtime as orml_xtokens::Config>::CurrencyId;
pub type XBalanceOf<Runtime> = <Runtime as orml_xtokens::Config>::Balance;
pub const CALL_DATA_LIMIT: u32 = 2u32.pow(16);
type GetCallDataLimit = ConstU32<CALL_DATA_LIMIT>;

// fn selectors
const PARSE_VM_SELECTOR: u32 = 0xa9e11893_u32; // parseVM(bytes)
const PARSE_TRANSFER_WITH_PAYLOAD_SELECTOR: u32 = 0xea63738d_u32; // parseTransferWithPayload(bytes)
const COMPLETE_TRANSFER_WITH_PAYLOAD_SELECTOR: u32 = 0xc3f511c1_u32; // completeTransferWithPayload(bytes)
const WRAPPED_ASSET_SELECTOR: u32 = 0x1ff1e286_u32; // wrappedAsset(uint16,bytes32)
const BALANCE_OF_SELECTOR: u32 = 0x70a08231_u32; // balanceOf(address)

/// Gmp precompile.
#[derive(Debug, Clone)]
pub struct GmpPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> GmpPrecompile<Runtime>
where
	Runtime: pallet_evm::Config + frame_system::Config + pallet_xcm::Config + orml_xtokens::Config,
	SystemCallOf<Runtime>: Dispatchable<PostInfo = PostDispatchInfo> + Decode + GetDispatchInfo,
	<<Runtime as frame_system::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin:
		From<Option<Runtime::AccountId>>,
	<Runtime as frame_system::Config>::RuntimeCall: From<orml_xtokens::Call<Runtime>>,
	Runtime: AccountIdToCurrencyId<Runtime::AccountId, CurrencyIdOf<Runtime>>,
	XBalanceOf<Runtime>: TryFrom<U256> + Into<U256> + EvmData,
{
	#[precompile::public("wormholeTransferERC20(bytes)")]
	pub fn wormhole_transfer_erc20(
		handle: &mut impl PrecompileHandle,
		wormhole_vaa: BoundedBytes<GetCallDataLimit>,
	) -> EvmResult {
		log::debug!(target: "gmp-precompile", "wormhole_vaa: {:?}", wormhole_vaa.clone());

		// TODO: need to pull this from storage or config somewhere
		//
		// Moonbase core bridge: 0xa5B7D85a8f27dd7907dc8FdC21FA5657D5E2F901
		// Moonbase token bridge: 0xbc976D4b9D57E57c3cA52e1Fd136C45FF7955A96
		// Deployment in "Test local Wormhole" ts test: 0x5cc307268a1393ab9a764a20dace848ab8275c46
		let wormhole = H160::from_str("0x5cc307268a1393ab9a764a20dace848ab8275c46")
			.map_err(|_| RevertReason::custom("invalid wormhole contract address"))?;

		let wormhole_bridge = H160::from_str("0x7d4567b7257cf869b01a47e8cf0edb3814bdb963")
			.map_err(|_| RevertReason::custom("invalid wormhole bridge contract address"))?;

		let wormhole_bridge_impl = H160::from_str("0x7d4567b7257cf869b01a47e8cf0edb3814bdb963")
			.map_err(|_| RevertReason::custom("invalid wormhole bridge impl contract address"))?;

		// get the wormhole VM from the provided VAA. Unfortunately, this forces us to parse
		// the VAA twice -- this seems to be a restriction imposed from the Wormhole contract design
		let output = Self::call(
			handle,
			wormhole,
			EvmDataWriter::new_with_selector(PARSE_VM_SELECTOR)
				.write(wormhole_vaa.clone())
				.build(),
		)?;
		let mut reader = EvmDataReader::new(&output[..]);
		let wormhole_vm: WormholeVM = reader.read()?;

		// get the bridge transfer data from the wormhole VM payload
		let output = Self::call(
			handle,
			wormhole_bridge,
			EvmDataWriter::new_with_selector(PARSE_TRANSFER_WITH_PAYLOAD_SELECTOR)
				.write(wormhole_vm.payload)
				.build(),
		)?;
		let mut reader = EvmDataReader::new(&output[..]);
		let transfer_with_payload: WormholeTransferWithPayloadData = reader.read()?;

		// get the wrapper for this asset by calling wrappedAsset()
		// TODO: this should only be done if needed (when token chain == our chain)
		let output = Self::call(
			handle,
			wormhole_bridge,
			EvmDataWriter::new_with_selector(WRAPPED_ASSET_SELECTOR)
				.write(transfer_with_payload.token_chain)
				.write(transfer_with_payload.token_address)
				.build(),
		)?;
		let mut reader = EvmDataReader::new(&output[..]);
		let wrapped_address: Address = reader.read()?;
		log::debug!(target: "gmp-precompile", "wrapped token address: {:?}", wrapped_address);

		// query our "before" balance (our being this precompile)
		let output = Self::call(
			handle,
			wrapped_address.into(),
			EvmDataWriter::new_with_selector(BALANCE_OF_SELECTOR)
				.write(Address::from(handle.code_address()))
				.build(),
		)?;
		let mut reader = EvmDataReader::new(&output[..]);
		let before_amount: U256 = reader.read()?;
		log::debug!(target: "gmp-precompile", "before balance: {}", before_amount);

		// our inner-most payload should be a VersionedUserAction
		let user_action = VersionedUserAction::decode_with_depth_limit(
			32,
			&mut transfer_with_payload.payload.as_bytes(),
		)
		.map_err(|_| RevertReason::Custom("Invalid GMP Payload".into()))?;
		log::debug!(target: "gmp-precompile", "user action: {:?}", user_action);

		// inspect the token the user wants to use: make sure it is XCM-capable
		let asset_address: H160 = transfer_with_payload
			.token_address
			.try_into()
			.map_err(|_| revert("Asset address is not a H160"))?;
		let currency_account_id = Runtime::AddressMapping::into_account_id(wrapped_address.into());

		let currency_id: <Runtime as orml_xtokens::Config>::CurrencyId =
			Runtime::account_to_currency_id(currency_account_id)
				.ok_or(revert("Unsupported asset, not a valid currency id"))?;

		let amount = transfer_with_payload
			.amount
			.try_into()
			.map_err(|_| revert("Amount overflows balance"))?;

		log::debug!(target: "gmp-precompile", "attempt to transfer {:?} of {:?}", amount, currency_id);

		// TODO: now check before balance

		// Complete a "Contract Controlled Transfer" with the given Wormhole VAA.
		// We need to invoke Wormhole's completeTransferWithPayload function, passing it the VAA,
		// then use the returned payload to decide what to do.
		let sub_context = Context {
			caller: handle.code_address(), // TODO: can we trust this to always be "this precompile"?
			address: wormhole_bridge_impl,
			apparent_value: U256::zero(), // TODO: any reason to pass value on, or reject txns with value?
		};

		log::debug!(target: "gmp-precompile", "calling Wormhole completeTransferWithPayload on {}...", wormhole_bridge_impl);
		let (reason, output) = handle.call(
			wormhole_bridge_impl,
			None,
			EvmDataWriter::new_with_selector(COMPLETE_TRANSFER_WITH_PAYLOAD_SELECTOR)
				.write(wormhole_vaa)
				.build(),
			handle.gas_limit(), // TODO
			false,
			&sub_context,
		);

		ensure_exit_reason_success(reason, &output[..])?;

		// query our "after" balance (our being this precompile)
		let output = Self::call(
			handle,
			wrapped_address.into(),
			EvmDataWriter::new_with_selector(BALANCE_OF_SELECTOR)
				.write(Address::from(handle.code_address()))
				.build(),
		)?;
		let mut reader = EvmDataReader::new(&output[..]);
		let after_amount: U256 = reader.read()?;
		log::debug!(target: "gmp-precompile", "after balance: {}", after_amount);

		let amount_transfered = after_amount.saturating_sub(before_amount);

		// TODO: review
		if amount_transfered < transfer_with_payload.amount {
			log::debug!(target: "gmp-precompile", "insufficient funds bridged ({} < {})", amount_transfered, transfer_with_payload.amount);
			return Err(PrecompileFailure::Revert {
				exit_status: ExitRevert::Reverted,
				output: "insufficient funds bridged".into(),
			});
		}

		// TODO: Wormhole might have transfered unsupported tokens; we should handle this case
		//       gracefully (maybe that's as simple as reverting)

		// TODO:
		let weight_limit: u64 = 1_000_000_000_000u64;

		log::debug!(target: "gmp-precompile", "sending XCM via xtokens::transfer...");
		let call: orml_xtokens::Call<Runtime> = match user_action {
			VersionedUserAction::V1(action) => orml_xtokens::Call::<Runtime>::transfer {
				currency_id,
				amount,
				dest: Box::new(VersionedMultiLocation::V1(action.destination)),
				dest_weight_limit: WeightLimit::Limited(weight_limit),
			},
		};

		log::debug!(target: "gmp-precompile", "sending xcm {:?}", call);

		// TODO: proper origin
		let origin = Runtime::AddressMapping::into_account_id(handle.code_address());
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call).map_err(|e| {
			log::debug!(target: "gmp-precompile", "error sending XCM: {:?}", e);
			e
		})?;

		Ok(())
	}

	/// call the given contract / function selector and return its output. Returns Err if the EVM
	/// exit reason is not Succeed.
	fn call(
		handle: &mut impl PrecompileHandle,
		contract_address: H160,
		call_data: Vec<u8>,
	) -> EvmResult<Vec<u8>> {
		let sub_context = Context {
			caller: handle.code_address(),
			address: contract_address,
			apparent_value: U256::zero(),
		};

		log::debug!(
			target: "gmp-precompile",
			"calling {} ...", contract_address,
		);

		let (reason, output) = handle.call(
			contract_address,
			None,
			call_data,
			handle.gas_limit(), // TODO
			false,
			&sub_context,
		);

		ensure_exit_reason_success(reason, &output[..])?;

		Ok(output)
	}
}

fn ensure_exit_reason_success(reason: ExitReason, output: &[u8]) -> EvmResult<()> {
	log::trace!(target: "gmp-precompile", "reason: {:?}", reason);
	log::trace!(target: "gmp-precompile", "output: {:x?}", &output[..]);

	match reason {
		ExitReason::Fatal(exit_status) => Err(PrecompileFailure::Fatal { exit_status }),
		ExitReason::Revert(exit_status) => Err(PrecompileFailure::Revert {
			exit_status,
			output: output.into(),
		}),
		ExitReason::Error(exit_status) => Err(PrecompileFailure::Error { exit_status }),
		ExitReason::Succeed(_) => Ok(()),
	}
}

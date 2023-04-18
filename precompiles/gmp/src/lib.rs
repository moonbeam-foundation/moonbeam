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
use fp_evm::{Context, PrecompileFailure, PrecompileHandle};
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
use sp_std::{marker::PhantomData, vec::Vec};
use types::*;
use xcm::{opaque::latest::WeightLimit, VersionedMultiLocation};
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
const PARSE_VM_SELECTOR: u32 = 0xa9e11893_u32;
const PARSE_TRANSFER_WITH_PAYLOAD_SELECTOR: u32 = 0xea63738d_u32;
const COMPLETE_TRANSFER_WITH_PAYLOAD_SELECTOR: u32 = 0xc3f511c1_u32;
const WRAPPED_ASSET_SELECTOR: u32 = 0x1ff1e286_u32;
const BALANCE_OF_SELECTOR: u32 = 0x70a08231_u32;

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

		// tally up gas cost:
		// 2 reads for contract addresses
		// 2500 as fudge for computation, esp. payload decoding (TODO: benchmark?)
		let initial_gas = 2500 + 2 * RuntimeHelper::<Runtime>::db_read_gas_cost();
		log::warn!("initial_gas: {:?}", initial_gas);
		handle.record_cost(initial_gas)?;

		let wormhole = storage::CoreAddress::get()
			.ok_or(RevertReason::custom("invalid wormhole core address"))?;

		let wormhole_bridge = storage::BridgeAddress::get()
			.ok_or(RevertReason::custom("invalid wormhole bridge address"))?;

		log::trace!(target: "gmp-precompile", "core contract: {:?}", wormhole);
		log::trace!(target: "gmp-precompile", "bridge contract: {:?}", wormhole_bridge);

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

		let currency_account_id = Runtime::AddressMapping::into_account_id(wrapped_address.into());

		let currency_id: <Runtime as orml_xtokens::Config>::CurrencyId =
			Runtime::account_to_currency_id(currency_account_id)
				.ok_or(revert("Unsupported asset, not a valid currency id"))?;

		// Complete a "Contract Controlled Transfer" with the given Wormhole VAA.
		// We need to invoke Wormhole's completeTransferWithPayload function, passing it the VAA.
		// Upon success, it should have transferred tokens to this precompile's address.
		Self::call(
			handle,
			wormhole_bridge,
			EvmDataWriter::new_with_selector(COMPLETE_TRANSFER_WITH_PAYLOAD_SELECTOR)
				.write(wormhole_vaa)
				.build(),
		)?;

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

		let amount_transferred = after_amount.saturating_sub(before_amount);
		let amount = amount_transferred
			.try_into()
			.map_err(|_| revert("Amount overflows balance"))?;

		log::debug!(target: "gmp-precompile", "sending XCM via xtokens::transfer...");
		let call: orml_xtokens::Call<Runtime> = match user_action {
			VersionedUserAction::V1(action) => orml_xtokens::Call::<Runtime>::transfer {
				currency_id,
				amount,
				dest: Box::new(VersionedMultiLocation::V3(action.destination)),
				dest_weight_limit: WeightLimit::Unlimited,
			},
		};

		log::debug!(target: "gmp-precompile", "sending xcm {:?}", call);

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

		let (reason, output) =
			handle.call(contract_address, None, call_data, None, false, &sub_context);

		ensure_exit_reason_success(reason, &output[..])?;

		Ok(output)
	}
}

fn ensure_exit_reason_success(reason: ExitReason, output: &[u8]) -> EvmResult<()> {
	log::trace!(target: "gmp-precompile", "reason: {:?}", reason);
	log::trace!(target: "gmp-precompile", "output: {:x?}", output);

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

/// We use pallet storage in our precompile by implementing a StorageInstance for each item we need
/// to store.
/// twox_128("gmp") => 0xb7f047395bba5df0367b45771c00de50
/// twox_128("CoreAddress") => 0x59ff23ff65cc809711800d9d04e4b14c
/// twox_128("BridgeAddress") => 0xc1586bde54b249fb7f521faf831ade45
mod storage {
	use super::*;
	use frame_support::{
		storage::types::{OptionQuery, StorageValue},
		traits::StorageInstance,
	};

	// storage for the core contract
	pub struct CoreAddressStorageInstance;
	impl StorageInstance for CoreAddressStorageInstance {
		const STORAGE_PREFIX: &'static str = "CoreAddress";
		fn pallet_prefix() -> &'static str {
			"gmp"
		}
	}
	pub type CoreAddress = StorageValue<CoreAddressStorageInstance, H160, OptionQuery>;

	// storage for the bridge contract
	pub struct BridgeAddressStorageInstance;
	impl StorageInstance for BridgeAddressStorageInstance {
		const STORAGE_PREFIX: &'static str = "BridgeAddress";
		fn pallet_prefix() -> &'static str {
			"gmp"
		}
	}
	pub type BridgeAddress = StorageValue<BridgeAddressStorageInstance, H160, OptionQuery>;
}

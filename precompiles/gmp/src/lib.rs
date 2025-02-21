// Copyright 2019-2025 PureStake Inc.
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

use account::SYSTEM_ACCOUNT_SIZE;
use evm::ExitReason;
use fp_evm::{Context, ExitRevert, PrecompileFailure, PrecompileHandle};
use frame_support::{
	dispatch::{GetDispatchInfo, PostDispatchInfo},
	sp_runtime::traits::Zero,
	traits::ConstU32,
};
use pallet_evm::AddressMapping;
use parity_scale_codec::{Decode, DecodeLimit};
use precompile_utils::{prelude::*, solidity::revert::revert_as_bytes};
use sp_core::{H160, U256};
use sp_runtime::traits::{Convert, Dispatchable};
use sp_std::boxed::Box;
use sp_std::{marker::PhantomData, vec::Vec};
use types::*;
use xcm::opaque::latest::{Asset, AssetId, Fungibility, WeightLimit};
use xcm::{VersionedAssets, VersionedLocation};
use xcm_primitives::{split_location_into_chain_part_and_beneficiary, AccountIdToCurrencyId};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod types;

pub type SystemCallOf<Runtime> = <Runtime as frame_system::Config>::RuntimeCall;
pub type CurrencyIdOf<Runtime> = <Runtime as pallet_xcm_transactor::Config>::CurrencyId;
pub type CurrencyIdToLocationOf<Runtime> =
	<Runtime as pallet_xcm_transactor::Config>::CurrencyIdToLocation;

pub const CALL_DATA_LIMIT: u32 = 2u32.pow(16);
type GetCallDataLimit = ConstU32<CALL_DATA_LIMIT>;

// fn selectors
const PARSE_VM_SELECTOR: u32 = 0xa9e11893_u32;
const PARSE_TRANSFER_WITH_PAYLOAD_SELECTOR: u32 = 0xea63738d_u32;
const COMPLETE_TRANSFER_WITH_PAYLOAD_SELECTOR: u32 = 0xc3f511c1_u32;
const WRAPPED_ASSET_SELECTOR: u32 = 0x1ff1e286_u32;
const CHAIN_ID_SELECTOR: u32 = 0x9a8a0592_u32;
const BALANCE_OF_SELECTOR: u32 = 0x70a08231_u32;
const TRANSFER_SELECTOR: u32 = 0xa9059cbb_u32;

/// Gmp precompile.
#[derive(Debug, Clone)]
pub struct GmpPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> GmpPrecompile<Runtime>
where
	Runtime: pallet_evm::Config
		+ frame_system::Config
		+ pallet_xcm::Config
		+ pallet_xcm_transactor::Config,
	SystemCallOf<Runtime>: Dispatchable<PostInfo = PostDispatchInfo> + Decode + GetDispatchInfo,
	<<Runtime as frame_system::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin:
		From<Option<Runtime::AccountId>>,
	<Runtime as frame_system::Config>::RuntimeCall: From<pallet_xcm::Call<Runtime>>,
	Runtime: AccountIdToCurrencyId<Runtime::AccountId, CurrencyIdOf<Runtime>>,
	<Runtime as pallet_evm::Config>::AddressMapping: AddressMapping<Runtime::AccountId>,
{
	#[precompile::public("wormholeTransferERC20(bytes)")]
	pub fn wormhole_transfer_erc20(
		handle: &mut impl PrecompileHandle,
		wormhole_vaa: BoundedBytes<GetCallDataLimit>,
	) -> EvmResult {
		log::debug!(target: "gmp-precompile", "wormhole_vaa: {:?}", wormhole_vaa.clone());

		// tally up gas cost:
		// 1 read for enabled flag
		// 2 reads for contract addresses
		// 2500 as fudge for computation, esp. payload decoding (TODO: benchmark?)
		handle.record_cost(2500)?;
		// CoreAddress: AccountId(20)
		handle.record_db_read::<Runtime>(20)?;
		// BridgeAddress: AccountId(20)
		handle.record_db_read::<Runtime>(20)?;
		// PrecompileEnabled: AccountId(1)
		handle.record_db_read::<Runtime>(1)?;

		ensure_enabled()?;

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
			solidity::encode_with_selector(PARSE_VM_SELECTOR, wormhole_vaa.clone()),
		)?;
		let wormhole_vm: WormholeVM = solidity::decode_return_value(&output[..])?;

		// get the bridge transfer data from the wormhole VM payload
		let output = Self::call(
			handle,
			wormhole_bridge,
			solidity::encode_with_selector(
				PARSE_TRANSFER_WITH_PAYLOAD_SELECTOR,
				wormhole_vm.payload,
			),
		)?;
		let transfer_with_payload: WormholeTransferWithPayloadData =
			solidity::decode_return_value(&output[..])?;

		// get the chainId that is "us" according to the bridge
		let output = Self::call(
			handle,
			wormhole_bridge,
			solidity::encode_with_selector(CHAIN_ID_SELECTOR, ()),
		)?;
		let chain_id: U256 = solidity::decode_return_value(&output[..])?;
		log::debug!(target: "gmp-precompile", "our chain id: {:?}", chain_id);

		// if the token_chain is not equal to our chain_id, we expect a wrapper ERC20
		let asset_erc20_address = if chain_id == transfer_with_payload.token_chain.into() {
			Address::from(H160::from(transfer_with_payload.token_address))
		} else {
			// get the wrapper for this asset by calling wrappedAsset()
			let output = Self::call(
				handle,
				wormhole_bridge,
				solidity::encode_with_selector(
					WRAPPED_ASSET_SELECTOR,
					(
						transfer_with_payload.token_chain,
						transfer_with_payload.token_address,
					),
				),
			)?;
			let wrapped_asset: Address = solidity::decode_return_value(&output[..])?;
			log::debug!(target: "gmp-precompile", "wrapped token address: {:?}", wrapped_asset);

			wrapped_asset
		};

		// query our "before" balance (our being this precompile)
		let output = Self::call(
			handle,
			asset_erc20_address.into(),
			solidity::encode_with_selector(BALANCE_OF_SELECTOR, Address(handle.code_address())),
		)?;
		let before_amount: U256 = solidity::decode_return_value(&output[..])?;
		log::debug!(target: "gmp-precompile", "before balance: {}", before_amount);

		// our inner-most payload should be a VersionedUserAction
		let user_action = VersionedUserAction::decode_with_depth_limit(
			32,
			&mut transfer_with_payload.payload.as_bytes(),
		)
		.map_err(|_| RevertReason::Custom("Invalid GMP Payload".into()))?;
		log::debug!(target: "gmp-precompile", "user action: {:?}", user_action);

		let currency_account_id =
			Runtime::AddressMapping::into_account_id(asset_erc20_address.into());

		let currency_id: CurrencyIdOf<Runtime> =
			Runtime::account_to_currency_id(currency_account_id)
				.ok_or(revert("Unsupported asset, not a valid currency id"))?;

		// Complete a "Contract Controlled Transfer" with the given Wormhole VAA.
		// We need to invoke Wormhole's completeTransferWithPayload function, passing it the VAA.
		// Upon success, it should have transferred tokens to this precompile's address.
		Self::call(
			handle,
			wormhole_bridge,
			solidity::encode_with_selector(COMPLETE_TRANSFER_WITH_PAYLOAD_SELECTOR, wormhole_vaa),
		)?;

		// query our "after" balance (our being this precompile)
		let output = Self::call(
			handle,
			asset_erc20_address.into(),
			solidity::encode_with_selector(
				BALANCE_OF_SELECTOR,
				Address::from(handle.code_address()),
			),
		)?;
		let after_amount: U256 = solidity::decode_return_value(&output[..])?;
		log::debug!(target: "gmp-precompile", "after balance: {}", after_amount);

		let amount_transferred = after_amount.saturating_sub(before_amount);
		let amount = amount_transferred
			.try_into()
			.map_err(|_| revert("Amount overflows balance"))?;

		log::debug!(target: "gmp-precompile", "sending XCM via xtokens::transfer...");
		let call: Option<pallet_xcm::Call<Runtime>> = match user_action {
			VersionedUserAction::V1(action) => {
				log::debug!(target: "gmp-precompile", "Payload: V1");

				let asset = Asset {
					fun: Fungibility::Fungible(amount),
					id: AssetId(
						<CurrencyIdToLocationOf<Runtime>>::convert(currency_id)
							.ok_or(revert("Cannot convert CurrencyId into xcm asset"))?,
					),
				};

				let (chain_part, beneficiary) = split_location_into_chain_part_and_beneficiary(
					action
						.destination
						.try_into()
						.map_err(|_| revert("Invalid destination"))?,
				)
				.ok_or(revert("Invalid destination"))?;

				Some(pallet_xcm::Call::<Runtime>::transfer_assets {
					dest: Box::new(VersionedLocation::V4(chain_part)),
					beneficiary: Box::new(VersionedLocation::V4(beneficiary)),
					assets: Box::new(VersionedAssets::V4(asset.into())),
					fee_asset_item: 0,
					weight_limit: WeightLimit::Unlimited,
				})
			}
			VersionedUserAction::V2(action) => {
				log::debug!(target: "gmp-precompile", "Payload: V2");
				// if the specified fee is more than the amount being transferred, we'll be nice to
				// the sender and pay them the entire amount.
				let fee = action.fee.min(amount_transferred);

				if fee > U256::zero() {
					let output = Self::call(
						handle,
						asset_erc20_address.into(),
						solidity::encode_with_selector(
							TRANSFER_SELECTOR,
							(Address::from(handle.context().caller), fee),
						),
					)?;
					let transferred: bool = solidity::decode_return_value(&output[..])?;

					if !transferred {
						return Err(RevertReason::custom("failed to transfer() fee").into());
					}
				}

				let fee = fee
					.try_into()
					.map_err(|_| revert("Fee amount overflows balance"))?;

				log::debug!(
					target: "gmp-precompile",
					"deducting fee from transferred amount {:?} - {:?} = {:?}",
					amount, fee, (amount - fee)
				);

				let remaining = amount.saturating_sub(fee);

				if !remaining.is_zero() {
					let asset = Asset {
						fun: Fungibility::Fungible(remaining),
						id: AssetId(
							<CurrencyIdToLocationOf<Runtime>>::convert(currency_id)
								.ok_or(revert("Cannot convert CurrencyId into xcm asset"))?,
						),
					};

					let (chain_part, beneficiary) = split_location_into_chain_part_and_beneficiary(
						action
							.destination
							.try_into()
							.map_err(|_| revert("Invalid destination"))?,
					)
					.ok_or(revert("Invalid destination"))?;

					Some(pallet_xcm::Call::<Runtime>::transfer_assets {
						dest: Box::new(VersionedLocation::V4(chain_part)),
						beneficiary: Box::new(VersionedLocation::V4(beneficiary)),
						assets: Box::new(VersionedAssets::V4(asset.into())),
						fee_asset_item: 0,
						weight_limit: WeightLimit::Unlimited,
					})
				} else {
					None
				}
			}
		};

		if let Some(call) = call {
			log::debug!(target: "gmp-precompile", "sending xcm {:?}", call);
			let origin = Runtime::AddressMapping::into_account_id(handle.code_address());
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				call,
				SYSTEM_ACCOUNT_SIZE,
			)
			.map_err(|e| {
				log::debug!(target: "gmp-precompile", "error sending XCM: {:?}", e);
				e
			})?;
		} else {
			log::debug!(target: "gmp-precompile", "no call provided, no XCM transfer");
		}

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
			"calling {} from {} ...", contract_address, sub_context.caller,
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

pub fn is_enabled() -> bool {
	match storage::PrecompileEnabled::get() {
		Some(enabled) => enabled,
		_ => false,
	}
}

fn ensure_enabled() -> EvmResult<()> {
	if is_enabled() {
		Ok(())
	} else {
		Err(PrecompileFailure::Revert {
			exit_status: ExitRevert::Reverted,
			output: revert_as_bytes("GMP Precompile is not enabled"),
		})
	}
}

/// We use pallet storage in our precompile by implementing a StorageInstance for each item we need
/// to store.
/// twox_128("gmp") => 0xb7f047395bba5df0367b45771c00de50
/// twox_128("CoreAddress") => 0x59ff23ff65cc809711800d9d04e4b14c
/// twox_128("BridgeAddress") => 0xc1586bde54b249fb7f521faf831ade45
/// twox_128("PrecompileEnabled") => 0x2551bba17abb82ef3498bab688e470b8
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

	// storage for precompile enabled
	// None or Some(false) both mean that the precompile is disabled; only Some(true) means enabled.
	pub struct PrecompileEnabledStorageInstance;
	impl StorageInstance for PrecompileEnabledStorageInstance {
		const STORAGE_PREFIX: &'static str = "PrecompileEnabled";
		fn pallet_prefix() -> &'static str {
			"gmp"
		}
	}
	pub type PrecompileEnabled = StorageValue<PrecompileEnabledStorageInstance, bool, OptionQuery>;
}

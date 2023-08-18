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

pub mod fungible;
pub mod generic;

pub use crate::weights::generic::WeightInfo;
use core::cmp::min;
use frame_support::weights::Weight;
use fungible::WeightInfo as XcmFungibleWeight;
use generic::SubstrateWeight as XcmGeneric;
use sp_std::prelude::*;
use xcm::{
	latest::{prelude::*, Weight as XCMWeight},
	DoubleEncoded,
};
use xcm_primitives::MAX_ASSETS;

trait WeighMultiAssets {
	fn weigh_multi_assets(&self, weight: Weight) -> XCMWeight;
}

trait WeighMultiAssetsFilter {
	fn weigh_multi_assets_filter(&self, weight: Weight) -> XCMWeight;
}

impl WeighMultiAssetsFilter for MultiAssetFilter {
	fn weigh_multi_assets_filter(&self, weight: Weight) -> XCMWeight {
		match self {
			Self::Definite(assets) => {
				weight.saturating_mul(assets.inner().into_iter().count() as u64)
			}
			Self::Wild(AllCounted(count) | AllOfCounted { count, .. }) => {
				weight.saturating_mul(min(MAX_ASSETS, *count) as u64)
			}
			Self::Wild(All | AllOf { .. }) => weight.saturating_mul(MAX_ASSETS as u64),
		}
	}
}

impl WeighMultiAssets for MultiAssets {
	fn weigh_multi_assets(&self, weight: Weight) -> XCMWeight {
		weight.saturating_mul(self.inner().into_iter().count() as u64)
	}
}

pub struct XcmWeight<Runtime, Call>(core::marker::PhantomData<(Runtime, Call)>);
impl<Runtime, Call> XcmWeightInfo<Call> for XcmWeight<Runtime, Call>
where
	Runtime: frame_system::Config + pallet_erc20_xcm_bridge::Config,
{
	fn withdraw_asset(assets: &MultiAssets) -> XCMWeight {
		assets.inner().iter().fold(Weight::zero(), |acc, asset| {
			acc.saturating_add(XcmFungibleWeight::<Runtime>::withdraw_asset(&asset))
		})
	}
	// Currently there is no trusted reserve
	fn reserve_asset_deposited(_assets: &MultiAssets) -> XCMWeight {
		XcmFungibleWeight::<Runtime>::reserve_asset_deposited()
	}
	fn receive_teleported_asset(assets: &MultiAssets) -> XCMWeight {
		assets.weigh_multi_assets(XcmFungibleWeight::<Runtime>::receive_teleported_asset())
	}
	fn query_response(
		_query_id: &u64,
		_response: &Response,
		_max_weight: &Weight,
		_querier: &Option<MultiLocation>,
	) -> XCMWeight {
		XcmGeneric::<Runtime>::query_response()
	}
	fn transfer_asset(assets: &MultiAssets, _dest: &MultiLocation) -> XCMWeight {
		assets.inner().iter().fold(Weight::zero(), |acc, asset| {
			acc.saturating_add(XcmFungibleWeight::<Runtime>::transfer_asset(&asset))
		})
	}
	fn transfer_reserve_asset(
		assets: &MultiAssets,
		_dest: &MultiLocation,
		_xcm: &Xcm<()>,
	) -> XCMWeight {
		assets.inner().iter().fold(Weight::zero(), |acc, asset| {
			acc.saturating_add(XcmFungibleWeight::<Runtime>::transfer_reserve_asset(&asset))
		})
	}
	fn transact(
		_origin_type: &OriginKind,
		_require_weight_at_most: &Weight,
		_call: &DoubleEncoded<Call>,
	) -> XCMWeight {
		XcmGeneric::<Runtime>::transact()
	}
	fn hrmp_new_channel_open_request(
		_sender: &u32,
		_max_message_size: &u32,
		_max_capacity: &u32,
	) -> XCMWeight {
		// XCM Executor does not currently support HRMP channel operations
		Weight::MAX
	}
	fn hrmp_channel_accepted(_recipient: &u32) -> XCMWeight {
		// XCM Executor does not currently support HRMP channel operations
		Weight::MAX
	}
	fn hrmp_channel_closing(_initiator: &u32, _sender: &u32, _recipient: &u32) -> XCMWeight {
		// XCM Executor does not currently support HRMP channel operations
		Weight::MAX
	}
	fn clear_origin() -> XCMWeight {
		XcmGeneric::<Runtime>::clear_origin()
	}
	fn descend_origin(_who: &InteriorMultiLocation) -> XCMWeight {
		XcmGeneric::<Runtime>::descend_origin()
	}
	fn report_error(_query_response_info: &QueryResponseInfo) -> XCMWeight {
		XcmGeneric::<Runtime>::report_error()
	}
	fn deposit_asset(assets: &MultiAssetFilter, _dest: &MultiLocation) -> XCMWeight {
		assets.weigh_multi_assets_filter(XcmFungibleWeight::<Runtime>::deposit_asset())
	}
	fn deposit_reserve_asset(
		assets: &MultiAssetFilter,
		_dest: &MultiLocation,
		_xcm: &Xcm<()>,
	) -> XCMWeight {
		assets.weigh_multi_assets_filter(XcmFungibleWeight::<Runtime>::deposit_reserve_asset())
	}
	fn exchange_asset(
		_give: &MultiAssetFilter,
		_receive: &MultiAssets,
		_maximal: &bool,
	) -> XCMWeight {
		Weight::MAX
	}
	fn initiate_reserve_withdraw(
		_assets: &MultiAssetFilter,
		_reserve: &MultiLocation,
		_xcm: &Xcm<()>,
	) -> XCMWeight {
		// This is not correct. initiate reserve withdraw does not to that many db reads
		// the only thing it does based on number of assets is a take from a local variable
		//assets.weigh_multi_assets(XcmGeneric::<Runtime>::initiate_reserve_withdraw())
		XcmGeneric::<Runtime>::initiate_reserve_withdraw()
	}
	fn initiate_teleport(
		_assets: &MultiAssetFilter,
		_dest: &MultiLocation,
		_xcm: &Xcm<()>,
	) -> XCMWeight {
		XcmFungibleWeight::<Runtime>::initiate_teleport()
	}
	fn report_holding(_response_info: &QueryResponseInfo, _assets: &MultiAssetFilter) -> Weight {
		XcmGeneric::<Runtime>::report_holding()
	}
	fn buy_execution(_fees: &MultiAsset, _weight_limit: &WeightLimit) -> XCMWeight {
		XcmGeneric::<Runtime>::buy_execution()
	}
	fn refund_surplus() -> XCMWeight {
		XcmGeneric::<Runtime>::refund_surplus()
	}
	fn set_error_handler(_xcm: &Xcm<Call>) -> XCMWeight {
		XcmGeneric::<Runtime>::set_error_handler()
	}
	fn set_appendix(_xcm: &Xcm<Call>) -> XCMWeight {
		XcmGeneric::<Runtime>::set_appendix()
	}
	fn clear_error() -> XCMWeight {
		XcmGeneric::<Runtime>::clear_error()
	}
	fn claim_asset(_assets: &MultiAssets, _ticket: &MultiLocation) -> XCMWeight {
		XcmGeneric::<Runtime>::claim_asset()
	}
	fn trap(_code: &u64) -> XCMWeight {
		XcmGeneric::<Runtime>::trap()
	}
	fn subscribe_version(_query_id: &QueryId, _max_response_weight: &Weight) -> XCMWeight {
		XcmGeneric::<Runtime>::subscribe_version()
	}
	fn unsubscribe_version() -> XCMWeight {
		XcmGeneric::<Runtime>::unsubscribe_version()
	}
	fn burn_asset(assets: &MultiAssets) -> Weight {
		assets.weigh_multi_assets(XcmGeneric::<Runtime>::burn_asset())
	}
	fn expect_asset(assets: &MultiAssets) -> Weight {
		assets.weigh_multi_assets(XcmGeneric::<Runtime>::expect_asset())
	}
	fn expect_origin(_origin: &Option<MultiLocation>) -> Weight {
		XcmGeneric::<Runtime>::expect_origin()
	}
	fn expect_error(_error: &Option<(u32, XcmError)>) -> Weight {
		XcmGeneric::<Runtime>::expect_error()
	}
	fn expect_transact_status(_transact_status: &MaybeErrorCode) -> Weight {
		XcmGeneric::<Runtime>::expect_transact_status()
	}
	fn query_pallet(_module_name: &Vec<u8>, _response_info: &QueryResponseInfo) -> Weight {
		XcmGeneric::<Runtime>::query_pallet()
	}
	fn expect_pallet(
		_index: &u32,
		_name: &Vec<u8>,
		_module_name: &Vec<u8>,
		_crate_major: &u32,
		_min_crate_minor: &u32,
	) -> Weight {
		XcmGeneric::<Runtime>::expect_pallet()
	}
	fn report_transact_status(_response_info: &QueryResponseInfo) -> Weight {
		XcmGeneric::<Runtime>::report_transact_status()
	}
	fn clear_transact_status() -> Weight {
		XcmGeneric::<Runtime>::clear_transact_status()
	}
	fn universal_origin(_: &Junction) -> Weight {
		Weight::MAX
	}
	fn export_message(_: &NetworkId, _: &Junctions, _: &Xcm<()>) -> Weight {
		Weight::MAX
	}
	fn lock_asset(_: &MultiAsset, _: &MultiLocation) -> Weight {
		Weight::MAX
	}
	fn unlock_asset(_: &MultiAsset, _: &MultiLocation) -> Weight {
		Weight::MAX
	}
	fn note_unlockable(_: &MultiAsset, _: &MultiLocation) -> Weight {
		Weight::MAX
	}
	fn request_unlock(_: &MultiAsset, _: &MultiLocation) -> Weight {
		Weight::MAX
	}
	fn set_fees_mode(_: &bool) -> Weight {
		XcmGeneric::<Runtime>::set_fees_mode()
	}
	fn set_topic(_topic: &[u8; 32]) -> Weight {
		XcmGeneric::<Runtime>::set_topic()
	}
	fn clear_topic() -> Weight {
		XcmGeneric::<Runtime>::clear_topic()
	}
	fn alias_origin(_: &MultiLocation) -> Weight {
		// XCM Executor does not currently support alias origin operations
		Weight::MAX
	}
	fn unpaid_execution(_: &WeightLimit, _: &Option<MultiLocation>) -> Weight {
		XcmGeneric::<Runtime>::unpaid_execution()
	}
}

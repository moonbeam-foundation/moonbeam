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

mod moonbeam_xcm_benchmarks_fungible;
mod moonbeam_xcm_benchmarks_generic;

use crate::weights::moonbeam_xcm_benchmarks_generic::WeightInfo;
use frame_support::weights::Weight;
use moonbeam_xcm_benchmarks_fungible::WeightInfo as XcmFungibleWeight;
use moonbeam_xcm_benchmarks_generic::SubstrateWeight as XcmGeneric;
use sp_std::prelude::*;
use xcm::{
	latest::{prelude::*, Weight as XCMWeight},
	DoubleEncoded,
};

trait WeighMultiAssets {
	fn weigh_multi_assets(&self, weight: Weight) -> XCMWeight;
}

trait WeighMultiAssetsFilter {
	fn weigh_multi_assets_filter(&self, max_assests: u32, weight: Weight) -> XCMWeight;
}

impl WeighMultiAssetsFilter for MultiAssetFilter {
	fn weigh_multi_assets_filter(&self, max_assests: u32, weight: Weight) -> XCMWeight {
		let weight = match self {
			Self::Definite(assets) => {
				weight.saturating_mul(assets.inner().into_iter().count() as u64)
			}
			Self::Wild(_) => weight.saturating_mul(max_assests as u64),
		};
		weight.ref_time()
	}
}

impl WeighMultiAssets for MultiAssets {
	fn weigh_multi_assets(&self, weight: Weight) -> XCMWeight {
		weight
			.saturating_mul(self.inner().into_iter().count() as u64)
			.ref_time()
	}
}

pub struct XcmWeight<Runtime, Call>(core::marker::PhantomData<(Runtime, Call)>);
impl<Runtime, Call> XcmWeightInfo<Call> for XcmWeight<Runtime, Call>
where
	Runtime: frame_system::Config + pallet_erc20_xcm_bridge::Config,
{
	fn withdraw_asset(assets: &MultiAssets) -> XCMWeight {
		assets.inner().iter().fold(0, |acc, asset| {
			acc.saturating_add(XcmFungibleWeight::<Runtime>::withdraw_asset(&asset).ref_time())
		})
	}
	// Currently there is no trusted reserve
	fn reserve_asset_deposited(_assets: &MultiAssets) -> XCMWeight {
		XcmFungibleWeight::<Runtime>::reserve_asset_deposited().ref_time()
	}
	fn receive_teleported_asset(assets: &MultiAssets) -> XCMWeight {
		assets.weigh_multi_assets(XcmFungibleWeight::<Runtime>::receive_teleported_asset())
	}
	fn query_response(_query_id: &u64, _response: &Response, _max_weight: &u64) -> XCMWeight {
		XcmGeneric::<Runtime>::query_response().ref_time()
	}
	fn transfer_asset(assets: &MultiAssets, _dest: &MultiLocation) -> XCMWeight {
		assets.inner().iter().fold(0, |acc, asset| {
			acc.saturating_add(XcmFungibleWeight::<Runtime>::transfer_asset(&asset).ref_time())
		})
	}
	fn transfer_reserve_asset(
		assets: &MultiAssets,
		_dest: &MultiLocation,
		_xcm: &Xcm<()>,
	) -> XCMWeight {
		assets.inner().iter().fold(0, |acc, asset| {
			acc.saturating_add(
				XcmFungibleWeight::<Runtime>::transfer_reserve_asset(&asset).ref_time(),
			)
		})
	}
	fn transact(
		_origin_type: &OriginKind,
		_require_weight_at_most: &u64,
		_call: &DoubleEncoded<Call>,
	) -> XCMWeight {
		XcmGeneric::<Runtime>::transact().ref_time()
	}
	fn hrmp_new_channel_open_request(
		_sender: &u32,
		_max_message_size: &u32,
		_max_capacity: &u32,
	) -> XCMWeight {
		// XCM Executor does not currently support HRMP channel operations
		Weight::MAX.ref_time()
	}
	fn hrmp_channel_accepted(_recipient: &u32) -> XCMWeight {
		// XCM Executor does not currently support HRMP channel operations
		Weight::MAX.ref_time()
	}
	fn hrmp_channel_closing(_initiator: &u32, _sender: &u32, _recipient: &u32) -> XCMWeight {
		// XCM Executor does not currently support HRMP channel operations
		Weight::MAX.ref_time()
	}
	fn clear_origin() -> XCMWeight {
		XcmGeneric::<Runtime>::clear_origin().ref_time()
	}
	fn descend_origin(_who: &InteriorMultiLocation) -> XCMWeight {
		XcmGeneric::<Runtime>::descend_origin().ref_time()
	}
	fn report_error(
		_query_id: &QueryId,
		_dest: &MultiLocation,
		_max_response_weight: &u64,
	) -> XCMWeight {
		XcmGeneric::<Runtime>::report_error().ref_time()
	}

	fn deposit_asset(
		assets: &MultiAssetFilter,
		max_assets: &u32,
		_dest: &MultiLocation,
	) -> XCMWeight {
		assets.weigh_multi_assets_filter(*max_assets, XcmFungibleWeight::<Runtime>::deposit_asset())
	}
	fn deposit_reserve_asset(
		assets: &MultiAssetFilter,
		max_assets: &u32,
		_dest: &MultiLocation,
		_xcm: &Xcm<()>,
	) -> XCMWeight {
		assets.weigh_multi_assets_filter(
			*max_assets,
			XcmFungibleWeight::<Runtime>::deposit_reserve_asset(),
		)
	}
	fn exchange_asset(_give: &MultiAssetFilter, _receive: &MultiAssets) -> XCMWeight {
		Weight::MAX.ref_time()
	}
	fn initiate_reserve_withdraw(
		_assets: &MultiAssetFilter,
		_reserve: &MultiLocation,
		_xcm: &Xcm<()>,
	) -> XCMWeight {
		// This is not correct. initiate reserve withdraw does not to that many db reads
		// the only thing it does based on number of assets is a take from a local variable
		//assets.weigh_multi_assets(XcmGeneric::<Runtime>::initiate_reserve_withdraw())
		XcmGeneric::<Runtime>::initiate_reserve_withdraw().ref_time()
	}
	fn initiate_teleport(
		_assets: &MultiAssetFilter,
		_dest: &MultiLocation,
		_xcm: &Xcm<()>,
	) -> XCMWeight {
		XcmFungibleWeight::<Runtime>::initiate_teleport().ref_time()
	}
	fn query_holding(
		_query_id: &u64,
		_dest: &MultiLocation,
		_assets: &MultiAssetFilter,
		_max_response_weight: &u64,
	) -> XCMWeight {
		XcmGeneric::<Runtime>::query_holding().ref_time()
	}
	fn buy_execution(_fees: &MultiAsset, _weight_limit: &WeightLimit) -> XCMWeight {
		XcmGeneric::<Runtime>::buy_execution().ref_time()
	}
	fn refund_surplus() -> XCMWeight {
		XcmGeneric::<Runtime>::refund_surplus().ref_time()
	}
	fn set_error_handler(_xcm: &Xcm<Call>) -> XCMWeight {
		XcmGeneric::<Runtime>::set_error_handler().ref_time()
	}
	fn set_appendix(_xcm: &Xcm<Call>) -> XCMWeight {
		XcmGeneric::<Runtime>::set_appendix().ref_time()
	}
	fn clear_error() -> XCMWeight {
		XcmGeneric::<Runtime>::clear_error().ref_time()
	}
	fn claim_asset(_assets: &MultiAssets, _ticket: &MultiLocation) -> XCMWeight {
		XcmGeneric::<Runtime>::claim_asset().ref_time()
	}
	fn trap(_code: &u64) -> XCMWeight {
		XcmGeneric::<Runtime>::trap().ref_time()
	}
	fn subscribe_version(_query_id: &QueryId, _max_response_weight: &u64) -> XCMWeight {
		XcmGeneric::<Runtime>::subscribe_version().ref_time()
	}
	fn unsubscribe_version() -> XCMWeight {
		XcmGeneric::<Runtime>::unsubscribe_version().ref_time()
	}
}

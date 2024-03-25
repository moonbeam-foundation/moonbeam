// Copyright 2024 Moonbeam Foundation.
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

use fp_evm::PrecompileHandle;
use frame_support::{
	dispatch::{GetDispatchInfo, PostDispatchInfo},
	traits::{ConstU32, Currency},
	//pallet_prelude::{ConstU32, Get},
};
use pallet_evm::AddressMapping;
use precompile_utils::prelude::*;

use sp_core::{H160, H256, U256};
use sp_runtime::traits::Dispatchable;
use sp_std::{
	convert::{TryFrom, TryInto},
	fmt::Debug,
	marker::PhantomData,
};
use sp_weights::Weight;
use xcm::{
	latest::{Asset, AssetId, Assets, Fungibility, Location},
	prelude::WeightLimit::*,
	VersionedAssets, VersionedLocation,
};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub const MAX_ASSETS_ARRAY_LIMIT: u32 = 2;

type GetArrayLimit = ConstU32<MAX_ASSETS_ARRAY_LIMIT>;

type BalanceOf<T> = <<T as pallet_xcm::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;

pub struct PalletXcmPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> PalletXcmPrecompile<Runtime>
where
	Runtime: pallet_xcm::Config + pallet_evm::Config + frame_system::Config,
	BalanceOf<Runtime>: TryFrom<U256> + TryInto<u128> + Debug + solidity::Codec,
	<Runtime as frame_system::Config>::RuntimeCall:
		Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<<Runtime as frame_system::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin:
		From<Option<Runtime::AccountId>>,
	<Runtime as frame_system::Config>::RuntimeCall: From<pallet_xcm::Call<Runtime>>,
{
	/* 	"transactThroughSigned(\
	(uint8,bytes[]),\
	address,\
	(uint64,uint64),\
	bytes,\
	uint256,\
	(uint64,uint64),\
	bool)" */

	#[precompile::public(
		"transferAssets(\
		(uint8,bytes[]),\
		(uint8,bytes[]),\
		((uint8,bytes[]),uint256)[],\
		uint32,\
		(uint64,uint64))"
	)]
	//#[precompile::public("transfer_assets()")]
	fn transfer_assets(
		handle: &mut impl PrecompileHandle,
		dest: Location,
		beneficiary: Location,
		assets: BoundedVec<(Location, Convert<U256, u128>), GetArrayLimit>,
		fee_asset_item: u32,
		weight: Weight,
	) -> EvmResult {
		// TODO: record proper cost
		handle.record_cost(1000)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let assets: Vec<_> = assets.into();

		let xcm_assets: Assets = assets
			.into_iter()
			.map(|asset| Asset {
				id: AssetId(asset.0),
				fun: Fungibility::Fungible(asset.1.converted()),
			})
			.collect::<Vec<Asset>>()
			.into();

		let weight_limit = match weight.ref_time() {
			u64::MAX => Unlimited,
			_ => Limited(weight),
		};

		let call = pallet_xcm::Call::<Runtime>::transfer_assets {
			dest: Box::new(VersionedLocation::V4(dest)),
			beneficiary: Box::new(VersionedLocation::V4(beneficiary)),
			assets: Box::new(VersionedAssets::V4(xcm_assets)),
			fee_asset_item,
			weight_limit,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;
		Ok(())
	}
}

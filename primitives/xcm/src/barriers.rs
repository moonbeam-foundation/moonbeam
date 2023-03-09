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

use frame_support::{ensure, traits::Contains};
/// Allows execution from `origin` if it is contained in `T` (i.e. `T::Contains(origin)`) taking
/// payments into account and if it starts with DescendOrigin.
///
/// Only allows for `DescendOrigin` + `WithdrawAsset`, + `BuyExecution`
use sp_std::marker::PhantomData;
use xcm::latest::prelude::*;
use xcm::latest::{
	MultiLocation, Weight,
	WeightLimit::{Limited, Unlimited},
	Xcm,
};
use xcm_executor::traits::ShouldExecute;

/// Barrier allowing a top level paid message with DescendOrigin instruction
/// first
pub struct AllowTopLevelPaidExecutionDescendOriginFirst<T>(PhantomData<T>);
impl<T: Contains<MultiLocation>> ShouldExecute for AllowTopLevelPaidExecutionDescendOriginFirst<T> {
	fn should_execute<Call>(
		origin: &MultiLocation,
		message: &mut Xcm<Call>,
		max_weight: u64,
		_weight_credit: &mut u64,
	) -> Result<(), ()> {
		log::trace!(
			target: "xcm::barriers",
			"AllowTopLevelPaidExecutionDescendOriginFirst origin:
			{:?}, message: {:?}, max_weight: {:?}, weight_credit: {:?}",
			origin, message, max_weight, _weight_credit,
		);
		ensure!(T::contains(origin), ());
		let mut iter = message.0.iter_mut();
		// Make sure the first instruction is DescendOrigin
		iter.next()
			.filter(|instruction| matches!(instruction, DescendOrigin(_)))
			.ok_or(())?;

		// Then WithdrawAsset
		iter.next()
			.filter(|instruction| matches!(instruction, WithdrawAsset(_)))
			.ok_or(())?;

		// Then BuyExecution
		let i = iter.next().ok_or(())?;
		match i {
			BuyExecution {
				weight_limit: Limited(ref mut weight),
				..
			} if *weight >= max_weight => {
				*weight = max_weight;
				Ok(())
			}
			BuyExecution {
				ref mut weight_limit,
				..
			} if weight_limit == &Unlimited => {
				*weight_limit = Limited(max_weight);
				Ok(())
			}
			_ => Err(()),
		}
	}
}

/// Make sure that not withdrawable assets are handle properly according to the XCM design:
/// - forbid any ReserveAssetDeposited instruction that contains not withdrawable asset(s) as
/// parameter.
/// - Morph some specific messages patterns that try to Withdraw then Deposit not withdrawable
/// asset(s).
pub struct NotWithdrawableAssetsBarrier<IsWithdrawable, InnerBarrier>(
	core::marker::PhantomData<(IsWithdrawable, InnerBarrier)>,
);

impl<IsWithdrawable: crate::IsWithdrawable, InnerBarrier: ShouldExecute> ShouldExecute
	for NotWithdrawableAssetsBarrier<IsWithdrawable, InnerBarrier>
{
	fn should_execute<Call>(
		origin: &MultiLocation,
		instructions: &mut Xcm<Call>,
		max_weight: Weight,
		weight_credit: &mut Weight,
	) -> Result<(), ()> {
		InnerBarrier::should_execute(origin, instructions, max_weight, weight_credit)?;
		let maybe_morphed_message = match &instructions.0[..] {
			&[WithdrawAsset(ref assets), ClearOrigin, BuyExecution {
				ref fees,
				ref weight_limit,
			}, DepositAsset {
				assets: ref filter,
				ref beneficiary,
				..
			}] => morph_standard_to_reserve_message::<Call, IsWithdrawable>(
				assets,
				fees,
				weight_limit,
				filter,
				beneficiary,
			),
			_ => None,
		};

		if let Some(mut morphed_message) = maybe_morphed_message {
			for i in 0..morphed_message.len() {
				core::mem::swap(&mut instructions.0[i], &mut morphed_message[i]);
			}
		}

		if instructions.0.iter().any(|instruction| {
			matches!(
				instruction, ReserveAssetDeposited(assets) if assets.inner().iter()
				.any(|asset| IsWithdrawable::is_withdrawable(&asset))
			)
		}) {
			Err(())
		} else {
			Ok(())
		}
	}
}

fn morph_standard_to_reserve_message<Call, IsWithdrawable: crate::IsWithdrawable>(
	assets: &MultiAssets,
	fees: &MultiAsset,
	weight_limit: &WeightLimit,
	deposit_filter: &MultiAssetFilter,
	beneficiary: &MultiLocation,
) -> Option<sp_std::vec::Vec<Instruction<Call>>> {
	let mut not_withdrawable_assets = sp_std::vec::Vec::new();
	let mut withdrawable_assets = sp_std::vec::Vec::new();
	for asset in assets.inner() {
		if IsWithdrawable::is_withdrawable(&asset) {
			not_withdrawable_assets.push(asset.clone());
		} else {
			withdrawable_assets.push(asset.clone());
		}
	}
	if not_withdrawable_assets.is_empty() {
		None
	} else {
		Some(sp_std::vec![
			WithdrawAsset(MultiAssets::from(withdrawable_assets)),
			BuyExecution {
				fees: fees.clone(),
				weight_limit: weight_limit.clone(),
			},
			TransferAsset {
				assets: MultiAssets::from(not_withdrawable_assets),
				beneficiary: beneficiary.clone(),
			},
			DepositAsset {
				assets: deposit_filter.clone(),
				beneficiary: beneficiary.clone(),
				max_assets: 2,
			},
		])
	}
}

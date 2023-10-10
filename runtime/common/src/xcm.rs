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

use frame_support::{traits::ProcessMessageError, weights::Weight};
use xcm::v3::{
	Instruction::{
		self, BuyExecution, ClaimAsset, ClearOrigin, ReceiveTeleportedAsset, ReserveAssetDeposited,
		WithdrawAsset,
	},
	MultiLocation,
	WeightLimit::{Limited, Unlimited},
};
use xcm_builder::{CreateMatcher, MatchXcm};
use xcm_executor::traits::ShouldExecute;

/// Allows execution from all origins taking payment into account.
///
/// Only allows for `TeleportAsset`, `WithdrawAsset`, `ClaimAsset` and
/// `ReserveAssetDeposit` XCMs because they are the only ones that place assets
/// in the Holding Register to pay for execution. This is almost equal to
/// [`xcm_builder::AllowTopLevelPaidExecutionFrom<T>`] except that it allows for
/// multiple assets and is not generic to allow all origins.

pub struct AllowTopLevelPaidExecution;
impl ShouldExecute for AllowTopLevelPaidExecution {
	fn should_execute<RuntimeCall>(
		_origin: &MultiLocation,
		instructions: &mut [Instruction<RuntimeCall>],
		max_weight: Weight,
		_properties: &mut xcm_executor::traits::Properties,
	) -> Result<(), ProcessMessageError> {
		let end = instructions.len().min(5);
		instructions[..end]
			.matcher()
			.match_next_inst(|inst| match inst {
				ReceiveTeleportedAsset(..) | ReserveAssetDeposited(..) => Ok(()),
				WithdrawAsset(..) => Ok(()),
				ClaimAsset { .. } => Ok(()),
				_ => Err(ProcessMessageError::BadFormat),
			})?
			.skip_inst_while(|inst| matches!(inst, ClearOrigin))?
			.match_next_inst(|inst| {
				let res = match inst {
					BuyExecution {
						weight_limit: Limited(ref mut weight),
						..
					} if weight.all_gte(max_weight) => {
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
					_ => Err(ProcessMessageError::Overweight(max_weight)),
				};
				res
			})?;

		Ok(())
	}
}

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

use frame_support::{ensure, pallet_prelude::Weight, traits::Contains};
/// Allows execution from `origin` if it is contained in `T` (i.e. `T::Contains(origin)`) taking
/// payments into account and if it starts with DescendOrigin.
///
/// Only allows for `DescendOrigin` + `WithdrawAsset`, + `BuyExecution`
use sp_std::marker::PhantomData;
use xcm::latest::{
	prelude::*,
	MultiLocation,
	WeightLimit::{Limited, Unlimited},
};
use xcm_executor::traits::ShouldExecute;

/// Deny executing the xcm message if it matches any of the Deny filter regardless of anything else.
/// If it passes the Deny, and matches one of the Allow cases then it is let through.
pub struct DenyThenTry<Deny, Allow>(PhantomData<Deny>, PhantomData<Allow>)
where
	Deny: ShouldExecute,
	Allow: ShouldExecute;

impl<Deny, Allow> ShouldExecute for DenyThenTry<Deny, Allow>
where
	Deny: ShouldExecute,
	Allow: ShouldExecute,
{
	fn should_execute<RuntimeCall>(
		origin: &MultiLocation,
		message: &mut [Instruction<RuntimeCall>],
		max_weight: Weight,
		weight_credit: &mut Weight,
	) -> Result<(), ()> {
		Deny::should_execute(origin, message, max_weight, weight_credit)?;
		Allow::should_execute(origin, message, max_weight, weight_credit)
	}
}

/// Deny initiation of any teleport and withdrawal from non local account
pub struct DenyTeleportAndWithdrawFromNonLocalOrigin;
impl ShouldExecute for DenyTeleportAndWithdrawFromNonLocalOrigin {
	fn should_execute<RuntimeCall>(
		origin: &MultiLocation,
		message: &mut [Instruction<RuntimeCall>],
		_max_weight: Weight,
		_weight_credit: &mut Weight,
	) -> Result<(), ()> {
		for instr in message {
			match instr {
				// Deny teleport
				InitiateTeleport { .. } => {
					return Err(()); // Deny
				}
				// Allow InitiateReserveWithdraw only from a local account
				InitiateReserveWithdraw { .. }
					if matches!(
						origin,
						MultiLocation {
							parents: 0,
							interior: X1(AccountKey20 { network: None, .. })
						}
					) =>
				{
					return Err(()); // Deny
				}
				_ => continue,
			}
		}
		Ok(())
	}
}

/// Barrier allowing a top level paid message with DescendOrigin instruction
/// first
pub struct AllowTopLevelPaidExecutionDescendOriginFirst<T>(PhantomData<T>);
impl<T: Contains<MultiLocation>> ShouldExecute for AllowTopLevelPaidExecutionDescendOriginFirst<T> {
	fn should_execute<Call>(
		origin: &MultiLocation,
		message: &mut [Instruction<Call>],
		max_weight: Weight,
		_weight_credit: &mut Weight,
	) -> Result<(), ()> {
		log::trace!(
			target: "xcm::barriers",
			"AllowTopLevelPaidExecutionDescendOriginFirst origin:
			{:?}, message: {:?}, max_weight: {:?}, weight_credit: {:?}",
			origin, message, max_weight, _weight_credit,
		);
		ensure!(T::contains(origin), ());
		let mut iter = message.iter_mut();
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
			} if weight.all_gte(max_weight) => {
				weight.set_ref_time(max_weight.ref_time());
				weight.set_proof_size(max_weight.proof_size());
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

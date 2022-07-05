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

use frame_support::{ensure, traits::Contains, weights::Weight};
/// Allows execution from `origin` if it is contained in `T` (i.e. `T::Contains(origin)`) taking
/// payments into account and if it starts with DescendOrigin.
///
/// Only allows for `DescendOrigin` + `WithdrawAsset`, + `BuyExecution`
use sp_std::marker::PhantomData;
use xcm::latest::{
	prelude::{BuyExecution, DescendOrigin, WithdrawAsset},
	MultiLocation,
	WeightLimit::{Limited, Unlimited},
	Xcm,
};
use xcm_executor::traits::ShouldExecute;

pub struct AllowDescendOriginFromLocal<T>(PhantomData<T>);
impl<T: Contains<MultiLocation>> ShouldExecute for AllowDescendOriginFromLocal<T> {
	fn should_execute<Call>(
		origin: &MultiLocation,
		message: &mut Xcm<Call>,
		max_weight: Weight,
		_weight_credit: &mut Weight,
	) -> Result<(), ()> {
		log::trace!(
			target: "xcm::barriers",
			"AllowTopLevelPaidExecutionFromLocal origin:
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

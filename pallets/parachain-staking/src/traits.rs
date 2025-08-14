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

//! traits for parachain-staking

use crate::weights::WeightInfo;
use frame_support::{dispatch::PostDispatchInfo, pallet_prelude::Weight};
use sp_runtime::DispatchErrorWithPostInfo;

pub trait OnCollatorPayout<AccountId, Balance> {
	fn on_collator_payout(
		for_round: crate::RoundIndex,
		collator_id: AccountId,
		amount: Balance,
	) -> Weight;
}
impl<AccountId, Balance> OnCollatorPayout<AccountId, Balance> for () {
	fn on_collator_payout(
		_for_round: crate::RoundIndex,
		_collator_id: AccountId,
		_amount: Balance,
	) -> Weight {
		Weight::zero()
	}
}

pub trait OnNewRound {
	fn on_new_round(round_index: crate::RoundIndex) -> Weight;
}
impl OnNewRound for () {
	fn on_new_round(_round_index: crate::RoundIndex) -> Weight {
		Weight::zero()
	}
}

/// Defines the behavior to payout the collator's reward.
pub trait PayoutCollatorReward<Runtime: crate::Config> {
	fn payout_collator_reward(
		round_index: crate::RoundIndex,
		collator_id: Runtime::AccountId,
		amount: crate::BalanceOf<Runtime>,
	) -> Weight;
}

/// Defines the default behavior for paying out the collator's reward. The amount is directly
/// deposited into the collator's account.
impl<Runtime: crate::Config> PayoutCollatorReward<Runtime> for () {
	fn payout_collator_reward(
		for_round: crate::RoundIndex,
		collator_id: Runtime::AccountId,
		amount: crate::BalanceOf<Runtime>,
	) -> Weight {
		crate::Pallet::<Runtime>::mint_collator_reward(for_round, collator_id, amount)
	}
}

pub trait OnInactiveCollator<Runtime: crate::Config> {
	fn on_inactive_collator(
		collator_id: Runtime::AccountId,
		round: crate::RoundIndex,
	) -> Result<Weight, DispatchErrorWithPostInfo<PostDispatchInfo>>;
}

impl<Runtime: crate::Config> OnInactiveCollator<Runtime> for () {
	fn on_inactive_collator(
		collator_id: <Runtime>::AccountId,
		_round: crate::RoundIndex,
	) -> Result<Weight, DispatchErrorWithPostInfo<PostDispatchInfo>> {
		crate::Pallet::<Runtime>::go_offline_inner(collator_id)?;
		Ok(<Runtime as crate::Config>::WeightInfo::go_offline(
			crate::MAX_CANDIDATES,
		))
	}
}

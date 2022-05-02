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

//! traits for parachain-staking

pub trait OnCollatorPayout<AccountId, Balance> {
	fn on_collator_payout(
		for_round: crate::RoundIndex,
		collator_id: AccountId,
		amount: Balance,
	) -> frame_support::pallet_prelude::Weight;
}
impl<AccountId, Balance> OnCollatorPayout<AccountId, Balance> for () {
	fn on_collator_payout(
		_for_round: crate::RoundIndex,
		_collator_id: AccountId,
		_amount: Balance,
	) -> frame_support::pallet_prelude::Weight {
		0
	}
}

pub trait OnNewRound {
	fn on_new_round(round_index: crate::RoundIndex) -> frame_support::pallet_prelude::Weight;
}
impl OnNewRound for () {
	fn on_new_round(_round_index: crate::RoundIndex) -> frame_support::pallet_prelude::Weight {
		0
	}
}

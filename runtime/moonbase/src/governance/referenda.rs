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

//! # Gov2 config
//! Includes runtime configs for these substrate pallets:
//! 1. pallet-conviction-voting
//! 2. pallet-whitelist
//! 3. pallet-referenda

use super::*;
use frame_support::traits::{EitherOf, MapSuccess};
use frame_system::EnsureRootWithSuccess;
use moonbeam_runtime_common::weights as moonbase_weights;
use sp_runtime::traits::Replace;

parameter_types! {
	pub const VoteLockingPeriod: BlockNumber = 1 * DAYS;
}

impl pallet_conviction_voting::Config for Runtime {
	type WeightInfo = moonbase_weights::pallet_conviction_voting::WeightInfo<Runtime>;
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type Polls = Referenda;
	type MaxTurnout = frame_support::traits::TotalIssuanceOf<Balances, Self::AccountId>;
	// Maximum number of concurrent votes an account may have
	type MaxVotes = ConstU32<20>;
	// Minimum period of vote locking
	type VoteLockingPeriod = VoteLockingPeriod;
}

parameter_types! {
	pub const AlarmInterval: BlockNumber = 1;
	pub const SubmissionDeposit: Balance = 10 * UNIT * SUPPLY_FACTOR;
	pub const UndecidingTimeout: BlockNumber = 21 * DAYS;
}

pub type GeneralAdminOrRoot = EitherOf<EnsureRoot<AccountId>, origins::GeneralAdmin>;

/// The policy allows for Root or FastGeneralAdmin.
pub type FastGeneralAdminOrRoot = EitherOf<EnsureRoot<AccountId>, origins::FastGeneralAdmin>;

impl custom_origins::Config for Runtime {}

// The purpose of this pallet is to queue calls to be dispatched as by root later => the Dispatch
// origin corresponds to the Gov2 Whitelist track.
impl pallet_whitelist::Config for Runtime {
	type WeightInfo = moonbase_weights::pallet_whitelist::WeightInfo<Runtime>;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type WhitelistOrigin = EitherOf<
		EnsureRootWithSuccess<Self::AccountId, ConstU16<65535>>,
		MapSuccess<
			pallet_collective::EnsureProportionAtLeast<
				Self::AccountId,
				OpenTechCommitteeInstance,
				5,
				9,
			>,
			Replace<ConstU16<6>>,
		>,
	>;
	type DispatchWhitelistedOrigin = EitherOf<EnsureRoot<Self::AccountId>, WhitelistedCaller>;
	type Preimages = Preimage;
}

pallet_referenda::impl_tracksinfo_get!(TracksInfo, Balance, BlockNumber);

impl pallet_referenda::Config for Runtime {
	type WeightInfo = moonbase_weights::pallet_referenda::WeightInfo<Runtime>;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type Scheduler = Scheduler;
	type Currency = Balances;
	type SubmitOrigin = frame_system::EnsureSigned<AccountId>;
	type CancelOrigin = EitherOf<EnsureRoot<Self::AccountId>, ReferendumCanceller>;
	type KillOrigin = EitherOf<EnsureRoot<Self::AccountId>, ReferendumKiller>;
	type Slash = Treasury;
	type Votes = pallet_conviction_voting::VotesOf<Runtime>;
	type Tally = pallet_conviction_voting::TallyOf<Runtime>;
	type SubmissionDeposit = SubmissionDeposit;
	type MaxQueued = ConstU32<100>;
	type UndecidingTimeout = UndecidingTimeout;
	type AlarmInterval = AlarmInterval;
	type Tracks = TracksInfo;
	type Preimages = Preimage;
}

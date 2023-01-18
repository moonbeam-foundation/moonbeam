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

//! Councils for Gov1 and Gov2

use super::*;

pub type CouncilInstance = pallet_collective::Instance1;
pub type TechCommitteeInstance = pallet_collective::Instance2;
pub type TreasuryCouncilInstance = pallet_collective::Instance3;
pub type OpenTechCommitteeInstance = pallet_collective::Instance4;

impl pallet_collective::Config<CouncilInstance> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeEvent = RuntimeEvent;
	type Proposal = RuntimeCall;
	/// The maximum amount of time (in blocks) for council members to vote on motions.
	/// Motions may end in fewer blocks if enough votes are cast to determine the result.
	type MotionDuration = ConstU32<{ 3 * DAYS }>;
	/// The maximum number of proposals that can be open in the council at once.
	type MaxProposals = ConstU32<100>;
	/// The maximum number of council members.
	type MaxMembers = ConstU32<100>;
	type DefaultVote = pallet_collective::MoreThanMajorityThenPrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}

impl pallet_collective::Config<TechCommitteeInstance> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeEvent = RuntimeEvent;
	type Proposal = RuntimeCall;
	/// The maximum amount of time (in blocks) for technical committee members to vote on motions.
	/// Motions may end in fewer blocks if enough votes are cast to determine the result.
	type MotionDuration = ConstU32<{ 3 * DAYS }>;
	/// The maximum number of proposals that can be open in the technical committee at once.
	type MaxProposals = ConstU32<100>;
	/// The maximum number of technical committee members.
	type MaxMembers = ConstU32<100>;
	type DefaultVote = pallet_collective::MoreThanMajorityThenPrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}

impl pallet_collective::Config<TreasuryCouncilInstance> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeEvent = RuntimeEvent;
	type Proposal = RuntimeCall;
	/// The maximum amount of time (in blocks) for treasury council members to vote on motions.
	/// Motions may end in fewer blocks if enough votes are cast to determine the result.
	type MotionDuration = ConstU32<{ 3 * DAYS }>;
	/// The maximum number of proposals that can be open in the treasury council at once.
	type MaxProposals = ConstU32<20>;
	/// The maximum number of treasury council members.
	type MaxMembers = ConstU32<9>;
	type DefaultVote = pallet_collective::MoreThanMajorityThenPrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}

impl pallet_collective::Config<OpenTechCommitteeInstance> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeEvent = RuntimeEvent;
	type Proposal = RuntimeCall;
	/// The maximum amount of time (in blocks) for technical committee members to vote on motions.
	/// Motions may end in fewer blocks if enough votes are cast to determine the result.
	type MotionDuration = ConstU32<{ 14 * DAYS }>;
	/// The maximum number of proposals that can be open in the technical committee at once.
	type MaxProposals = ConstU32<100>;
	/// The maximum number of technical committee members.
	type MaxMembers = ConstU32<100>;
	type DefaultVote = pallet_collective::MoreThanMajorityThenPrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}

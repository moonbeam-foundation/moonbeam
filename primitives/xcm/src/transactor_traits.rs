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

use cumulus_primitives_core::{
	relay_chain::{self, HrmpChannelId},
	ParaId,
};
use sp_std::vec::Vec;
use xcm::latest::{Error as XcmError, MultiLocation};

// The utility calls that need to be implemented as part of
// using a derivative account from a certain account
#[derive(Debug, PartialEq, Eq)]
pub enum UtilityAvailableCalls {
	AsDerivative(u16, Vec<u8>),
}

// The hrmp calls that need to be implemented as part of
// HRMP management process
#[derive(Debug, PartialEq, Eq)]
pub enum HrmpAvailableCalls {
	InitOpenChannel(ParaId, u32, u32),
	AcceptOpenChannel(ParaId),
	CloseChannel(HrmpChannelId),
	CancelOpenRequest(HrmpChannelId, u32),
}

// Trait that the ensures we can encode a call with utility functions.
// With this trait we ensure that the user cannot control entirely the call
// to be performed in the destination chain. It only can control the call inside
// the as_derivative extrinsic, and thus, this call can only be dispatched from the
// derivative account
pub trait UtilityEncodeCall {
	fn encode_call(self, call: UtilityAvailableCalls) -> Vec<u8>;
}

// Trait that the ensures we can encode a call with hrmp functions.
// With this trait we ensure that the user cannot control entirely the call.
// to be performed in the destination chain.
pub trait HrmpEncodeCall {
	fn hrmp_encode_call(call: HrmpAvailableCalls) -> Result<Vec<u8>, XcmError>;
}

impl HrmpEncodeCall for () {
	fn hrmp_encode_call(_call: HrmpAvailableCalls) -> Result<Vec<u8>, XcmError> {
		Err(XcmError::Unimplemented)
	}
}

// Trait to ensure we can retrieve the destination if a given type
// It must implement UtilityEncodeCall
pub trait XcmTransact: UtilityEncodeCall {
	/// Encode call from the relay.
	fn destination(self) -> MultiLocation;
}

pub enum AvailableStakeCalls {
	Bond(
		relay_chain::Balance,
		pallet_staking::RewardDestination<relay_chain::AccountId>,
	),
	BondExtra(relay_chain::Balance),
	Unbond(relay_chain::Balance),
	WithdrawUnbonded(u32),
	Validate(pallet_staking::ValidatorPrefs),
	Nominate(Vec<relay_chain::AccountId>),
	Chill,
	SetPayee(pallet_staking::RewardDestination<relay_chain::AccountId>),
	SetController,
	Rebond(relay_chain::Balance),
}

pub trait StakeEncodeCall {
	/// Encode call from the relay.
	fn encode_call(call: AvailableStakeCalls) -> Vec<u8>;
}

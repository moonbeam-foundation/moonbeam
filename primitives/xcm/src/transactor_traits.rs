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

use sp_std::vec::Vec;
use xcm::latest::MultiLocation;

// The utility calls that need to be implemented as part of
// this pallet
#[derive(Debug, PartialEq, Eq)]
pub enum UtilityAvailableCalls {
	AsDerivative(u16, Vec<u8>),
}

// Trait that the ensures we can encode a call with utility functions.
// With this trait we ensure that the user cannot control entirely the call
// to be performed in the destination chain. It only can control the call inside
// the as_derivative extrinsic, and thus, this call can only be dispatched from the
// derivative account
pub trait UtilityEncodeCall {
	fn encode_call(self, call: UtilityAvailableCalls) -> Vec<u8>;
}

// Trait to ensure we can retrieve the destination if a given type
// It must implement UtilityEncodeCall
// We separate this in two traits to be able to implement UtilityEncodeCall separately
// for different runtimes of our choice
pub trait XcmTransact: UtilityEncodeCall {
	/// Encode call from the relay.
	fn destination(self) -> MultiLocation;
}

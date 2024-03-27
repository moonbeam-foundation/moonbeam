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

//! # Encode Module
//!
//! ## Overview
//!
//! Module to provide `StakeEncodeCall`, `HrmpEncodeCall` and `UtilityEncodeCall` implementations
//! for the Xcm Transactor pallet

#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::pallet_prelude::*;
use sp_runtime::traits::{AccountIdLookup, StaticLookup};
use sp_std::prelude::*;
use xcm_primitives::{
	AvailableStakeCalls, HrmpAvailableCalls, HrmpEncodeCall, StakeEncodeCall,
	UtilityAvailableCalls, UtilityEncodeCall,
};

pub use crate::pallet::*;

pub use crate::weights::WeightInfo;

impl<T: Config> UtilityEncodeCall for Pallet<T> {
	fn encode_call(self, call: UtilityAvailableCalls) -> Vec<u8> {
		match call {
			UtilityAvailableCalls::AsDerivative(a, b) => {
				let mut encoded_call: Vec<u8> = Vec::new();
				// pallet index
				encoded_call.push(RelayIndices::<T>::get().utility);
				// call index
				encoded_call.push(RelayIndices::<T>::get().as_derivative);
				// encoded argument
				encoded_call.append(&mut a.encode());
				encoded_call.append(&mut b.clone());
				encoded_call
			}
		}
	}
}

impl<T: Config> HrmpEncodeCall for Pallet<T> {
	fn hrmp_encode_call(call: HrmpAvailableCalls) -> Result<Vec<u8>, xcm::latest::Error> {
		match call {
			HrmpAvailableCalls::InitOpenChannel(a, b, c) => {
				let mut encoded_call: Vec<u8> = Vec::new();
				// pallet index
				encoded_call.push(RelayIndices::<T>::get().hrmp);
				// call index
				encoded_call.push(RelayIndices::<T>::get().init_open_channel);
				// encoded arguments
				encoded_call.append(&mut a.encode());
				encoded_call.append(&mut b.encode());
				encoded_call.append(&mut c.encode());
				Ok(encoded_call)
			}
			HrmpAvailableCalls::AcceptOpenChannel(a) => {
				let mut encoded_call: Vec<u8> = Vec::new();
				// pallet index
				encoded_call.push(RelayIndices::<T>::get().hrmp);
				// call index
				encoded_call.push(RelayIndices::<T>::get().accept_open_channel);
				// encoded argument
				encoded_call.append(&mut a.encode());
				Ok(encoded_call)
			}
			HrmpAvailableCalls::CloseChannel(a) => {
				let mut encoded_call: Vec<u8> = Vec::new();
				// pallet index
				encoded_call.push(RelayIndices::<T>::get().hrmp);
				// call index
				encoded_call.push(RelayIndices::<T>::get().close_channel);
				// encoded argument
				encoded_call.append(&mut a.encode());
				Ok(encoded_call)
			}
			HrmpAvailableCalls::CancelOpenRequest(a, b) => {
				let mut encoded_call: Vec<u8> = Vec::new();
				// pallet index
				encoded_call.push(RelayIndices::<T>::get().hrmp);
				// call index
				encoded_call.push(RelayIndices::<T>::get().cancel_open_request);
				// encoded argument
				encoded_call.append(&mut a.encode());
				encoded_call.append(&mut b.encode());
				Ok(encoded_call)
			}
		}
	}
}

fn encode_compact_arg<T: parity_scale_codec::HasCompact>(input: T) -> Vec<u8> {
	#[derive(Encode)]
	struct CompactWrapper<T: parity_scale_codec::HasCompact> {
		#[codec(compact)]
		input: T,
	}
	CompactWrapper { input }.encode()
}

impl<T: Config> StakeEncodeCall for Pallet<T> {
	fn encode_call(call: AvailableStakeCalls) -> Vec<u8> {
		match call {
			AvailableStakeCalls::Bond(b, c) => {
				let mut encoded_call: Vec<u8> = Vec::new();
				// pallet index
				encoded_call.push(RelayIndices::<T>::get().staking);
				// call index
				encoded_call.push(RelayIndices::<T>::get().bond);
				// encoded arguments
				encoded_call.append(&mut encode_compact_arg(b));
				encoded_call.append(&mut c.encode());
				encoded_call
			}

			AvailableStakeCalls::BondExtra(a) => {
				let mut encoded_call: Vec<u8> = Vec::new();
				// pallet index
				encoded_call.push(RelayIndices::<T>::get().staking);
				// call index
				encoded_call.push(RelayIndices::<T>::get().bond_extra);
				// encoded argument
				encoded_call.append(&mut encode_compact_arg(a));
				encoded_call
			}

			AvailableStakeCalls::Unbond(a) => {
				let mut encoded_call: Vec<u8> = Vec::new();
				// pallet index
				encoded_call.push(RelayIndices::<T>::get().staking);
				// call index
				encoded_call.push(RelayIndices::<T>::get().unbond);
				// encoded argument
				encoded_call.append(&mut encode_compact_arg(a));
				encoded_call
			}

			AvailableStakeCalls::WithdrawUnbonded(a) => {
				let mut encoded_call: Vec<u8> = Vec::new();
				// pallet index
				encoded_call.push(RelayIndices::<T>::get().staking);
				// call index
				encoded_call.push(RelayIndices::<T>::get().withdraw_unbonded);
				// encoded argument
				encoded_call.append(&mut a.encode());
				encoded_call
			}

			AvailableStakeCalls::Validate(a) => {
				let mut encoded_call: Vec<u8> = Vec::new();
				// pallet index
				encoded_call.push(RelayIndices::<T>::get().staking);
				// call index
				encoded_call.push(RelayIndices::<T>::get().validate);
				// encoded argument
				encoded_call.append(&mut a.encode());
				encoded_call
			}

			AvailableStakeCalls::Chill => {
				let mut encoded_call: Vec<u8> = Vec::new();
				// pallet index
				encoded_call.push(RelayIndices::<T>::get().staking);
				// call index
				encoded_call.push(RelayIndices::<T>::get().chill);
				encoded_call
			}

			AvailableStakeCalls::SetPayee(a) => {
				let mut encoded_call: Vec<u8> = Vec::new();
				// pallet index
				encoded_call.push(RelayIndices::<T>::get().staking);
				// call index
				encoded_call.push(RelayIndices::<T>::get().set_payee);
				// encoded argument
				encoded_call.append(&mut a.encode());
				encoded_call
			}

			AvailableStakeCalls::SetController => {
				let mut encoded_call: Vec<u8> = Vec::new();
				// pallet index
				encoded_call.push(RelayIndices::<T>::get().staking);
				// call index
				encoded_call.push(RelayIndices::<T>::get().set_controller);
				encoded_call
			}

			AvailableStakeCalls::Rebond(a) => {
				let mut encoded_call: Vec<u8> = Vec::new();
				// pallet index
				encoded_call.push(RelayIndices::<T>::get().staking);
				// call index
				encoded_call.push(RelayIndices::<T>::get().rebond);
				// encoded argument
				encoded_call.append(&mut encode_compact_arg(a));
				encoded_call
			}

			AvailableStakeCalls::Nominate(a) => {
				let mut encoded_call: Vec<u8> = Vec::new();
				// pallet index
				encoded_call.push(RelayIndices::<T>::get().staking);
				// call index
				encoded_call.push(RelayIndices::<T>::get().nominate);
				let nominated: Vec<
					<AccountIdLookup<sp_runtime::AccountId32, ()> as StaticLookup>::Source,
				> = a.iter().map(|add| (*add).clone().into()).collect();
				encoded_call.append(&mut nominated.encode());
				encoded_call
			}
		}
	}
}

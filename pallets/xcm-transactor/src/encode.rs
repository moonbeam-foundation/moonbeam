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

//! # Xcm Transactor Module
//!
//! ## Overview
//!
//! Module to provide transact capabilities on other chains
//!
//! In this pallet we will make distinctions between sovereign, derivative accounts and
//! multilocation-based derived accounts. The first is the account the parachain controls
//! in the destination chain, the second is an account derived from the
//! sovereign account itself, e.g., by hashing it with an index, while the third is an account
//! derived from the multilocation of a use in this chain (tipically, hashing the ML).
//! Such distinction is important since we want to keep the integrity of the sovereign account
//!
//! This pallet provides three ways of sending Transact operations to anothe chain
//!
//! - transact_through_derivative: Transact through an address derived from this chains sovereign
//! 	account in the destination chain. For the transaction to successfully be dispatched in the
//! 	destination chain, pallet-utility needs to be installed and at least paid xcm message
//! 	execution should be allowed (and WithdrawAsset,BuyExecution and Transact messages allowed)
//! 	in the destination chain
//!
//!
//!
//! 	The transactions are dispatched from a derivative account
//! 	of the sovereign account
//! 	This pallet only stores the index of the derivative account used, but
//! 	not the derivative account itself. The only assumption this pallet makes
//! 	is the existence of the pallet_utility pallet in the destination chain
//! 	through the XcmTransact trait.
//!
//! 	All calls will be wrapped around utility::as_derivative. This makes sure
//! 	the inner call is executed from the derivative account and not the sovereign
//! 	account itself.
//!
//! 	Index registration happens through DerivativeAddressRegistrationOrigin.
//! 	This derivative account can be funded by external users to
//! 	ensure it has enough funds to make the calls
//!
//! - transact_through_sovereign: Transact through the sovereign account representing this chain.
//! 	For the transaction to successfully be dispatched in the destination chain, at least paid
//! 	xcm message execution should be allowed (and WithdrawAsset,BuyExecution and Transact
//! 	messages allowed) in the destination chain. Only callable by Root
//!
//! - transact_through_signed: Transact through an account derived from the multilocation
//! 	representing the signed user making the call. We ensure this by prepending DescendOrigin as
//! 	the first instruction of the XCM message. For the transaction to successfully be dispatched
//! 	in the destination chain, at least descended paid xcm message execution should be allowed
//! 	(and DescendOrigin + WithdrawAsset + BuyExecution + Transact messages allowed) in the
//! 	destination chain. Additionally, a ML-based derivation mechanism needs to be implemented
//! 	in the destination chain.

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
				encoded_call.push(RelayIndices::<T>::get().pallets.utility);
				// call index
				encoded_call.push(RelayIndices::<T>::get().calls.utility.as_derivative);
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
				encoded_call.push(RelayIndices::<T>::get().pallets.hrmp);
				// call index
				encoded_call.push(RelayIndices::<T>::get().calls.hrmp.init_open_channel);
				// encoded arguments
				encoded_call.append(&mut a.encode());
				encoded_call.append(&mut b.encode());
				encoded_call.append(&mut c.encode());
				Ok(encoded_call)
			}
			HrmpAvailableCalls::AcceptOpenChannel(a) => {
				let mut encoded_call: Vec<u8> = Vec::new();
				// pallet index
				encoded_call.push(RelayIndices::<T>::get().pallets.hrmp);
				// call index
				encoded_call.push(RelayIndices::<T>::get().calls.hrmp.accept_open_channel);
				// encoded argument
				encoded_call.append(&mut a.encode());
				Ok(encoded_call)
			}
			HrmpAvailableCalls::CloseChannel(a) => {
				let mut encoded_call: Vec<u8> = Vec::new();
				// pallet index
				encoded_call.push(RelayIndices::<T>::get().pallets.hrmp);
				// call index
				encoded_call.push(RelayIndices::<T>::get().calls.hrmp.close_channel);
				// encoded argument
				encoded_call.append(&mut a.encode());
				Ok(encoded_call)
			}
			HrmpAvailableCalls::CancelOpenRequest(a, b) => {
				let mut encoded_call: Vec<u8> = Vec::new();
				// pallet index
				encoded_call.push(RelayIndices::<T>::get().pallets.hrmp);
				// call index
				encoded_call.push(RelayIndices::<T>::get().calls.hrmp.cancel_open_request);
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
				encoded_call.push(RelayIndices::<T>::get().pallets.staking);
				// call index
				encoded_call.push(RelayIndices::<T>::get().calls.staking.bond);
				// encoded arguments
				encoded_call.append(&mut encode_compact_arg(b));
				encoded_call.append(&mut c.encode());
				encoded_call
			}

			AvailableStakeCalls::BondExtra(a) => {
				let mut encoded_call: Vec<u8> = Vec::new();
				// pallet index
				encoded_call.push(RelayIndices::<T>::get().pallets.staking);
				// call index
				encoded_call.push(RelayIndices::<T>::get().calls.staking.bond_extra);
				// encoded argument
				encoded_call.append(&mut encode_compact_arg(a));
				encoded_call
			}

			AvailableStakeCalls::Unbond(a) => {
				let mut encoded_call: Vec<u8> = Vec::new();
				// pallet index
				encoded_call.push(RelayIndices::<T>::get().pallets.staking);
				// call index
				encoded_call.push(RelayIndices::<T>::get().calls.staking.unbond);
				// encoded argument
				encoded_call.append(&mut encode_compact_arg(a));
				encoded_call
			}

			AvailableStakeCalls::WithdrawUnbonded(a) => {
				let mut encoded_call: Vec<u8> = Vec::new();
				// pallet index
				encoded_call.push(RelayIndices::<T>::get().pallets.staking);
				// call index
				encoded_call.push(RelayIndices::<T>::get().calls.staking.withdraw_unbonded);
				// encoded argument
				encoded_call.append(&mut a.encode());
				encoded_call
			}

			AvailableStakeCalls::Validate(a) => {
				let mut encoded_call: Vec<u8> = Vec::new();
				// pallet index
				encoded_call.push(RelayIndices::<T>::get().pallets.staking);
				// call index
				encoded_call.push(RelayIndices::<T>::get().calls.staking.validate);
				// encoded argument
				encoded_call.append(&mut a.encode());
				encoded_call
			}

			AvailableStakeCalls::Chill => {
				let mut encoded_call: Vec<u8> = Vec::new();
				// pallet index
				encoded_call.push(RelayIndices::<T>::get().pallets.staking);
				// call index
				encoded_call.push(RelayIndices::<T>::get().calls.staking.chill);
				encoded_call
			}

			AvailableStakeCalls::SetPayee(a) => {
				let mut encoded_call: Vec<u8> = Vec::new();
				// pallet index
				encoded_call.push(RelayIndices::<T>::get().pallets.staking);
				// call index
				encoded_call.push(RelayIndices::<T>::get().calls.staking.set_payee);
				// encoded argument
				encoded_call.append(&mut a.encode());
				encoded_call
			}

			AvailableStakeCalls::SetController => {
				let mut encoded_call: Vec<u8> = Vec::new();
				// pallet index
				encoded_call.push(RelayIndices::<T>::get().pallets.staking);
				// call index
				encoded_call.push(RelayIndices::<T>::get().calls.staking.set_controller);
				encoded_call
			}

			AvailableStakeCalls::Rebond(a) => {
				let mut encoded_call: Vec<u8> = Vec::new();
				// pallet index
				encoded_call.push(RelayIndices::<T>::get().pallets.staking);
				// call index
				encoded_call.push(RelayIndices::<T>::get().calls.staking.rebond);
				// encoded argument
				encoded_call.append(&mut encode_compact_arg(a));
				encoded_call
			}

			AvailableStakeCalls::Nominate(a) => {
				let mut encoded_call: Vec<u8> = Vec::new();
				// pallet index
				encoded_call.push(RelayIndices::<T>::get().pallets.staking);
				// call index
				encoded_call.push(RelayIndices::<T>::get().calls.staking.nominate);
				let nominated: Vec<
					<AccountIdLookup<sp_runtime::AccountId32, ()> as StaticLookup>::Source,
				> = a.iter().map(|add| (*add).clone().into()).collect();
				encoded_call.append(&mut nominated.encode());
				encoded_call
			}
		}
	}
}

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

//! # Encode Module
//!
//! ## Overview
//!
//! Module to provide `StakeEncodeCall`, `HrmpEncodeCall` and `UtilityEncodeCall` implementations
//! for the Xcm Transactor pallet

use frame_support::pallet_prelude::*;
use parity_scale_codec::EncodeLike;
use sp_runtime::traits::{AccountIdLookup, StaticLookup};
use sp_std::prelude::*;
use xcm_primitives::{
	AvailableStakeCalls, HrmpAvailableCalls, HrmpEncodeCall, UtilityAvailableCalls,
};

pub use crate::pallet::*;

use crate::chain_indices::{AssetHubIndices, ChainIndices, RelayChainIndices};
pub use crate::weights::WeightInfo;

impl<T: Config> Pallet<T> {
	/// Get relay chain indices from ChainIndicesMap
	///
	/// This is used by HRMP and Utility encode functions which are relay-only operations.
	/// Searches ChainIndicesMap for the Relay variant, falling back to legacy storage.
	fn get_relay_indices() -> RelayChainIndices {
		// Search ChainIndicesMap for the Relay variant
		// We can't use a direct key lookup because the pallet doesn't know which
		// transactor value represents Relay (that's configured at runtime level)
		for (_transactor, chain_indices) in ChainIndicesMap::<T>::iter() {
			if let ChainIndices::Relay(indices) = chain_indices {
				return indices;
			}
		}

		// Fallback to old storage for backwards compatibility
		// This should only happen during migration or if storage is corrupted
		RelayIndices::<T>::get()
	}
}

impl<T: Config> Pallet<T> {
	pub fn encode_utility_call<Transactor: xcm_primitives::XcmTransact>(
		_transactor: Transactor,
		call: UtilityAvailableCalls,
	) -> Vec<u8> {
		let relay_indices = Self::get_relay_indices();
		match call {
			UtilityAvailableCalls::AsDerivative(a, b) => {
				let mut encoded_call: Vec<u8> = Vec::new();
				// pallet index
				encoded_call.push(relay_indices.utility);
				// call index
				encoded_call.push(relay_indices.as_derivative);
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
		let relay_indices = Self::get_relay_indices();
		let encoded_call = match call {
			HrmpAvailableCalls::InitOpenChannel(a, b, c) => {
				let mut encoded_call: Vec<u8> = Vec::new();
				// pallet index
				encoded_call.push(relay_indices.hrmp);
				// call index
				encoded_call.push(relay_indices.init_open_channel);
				// encoded arguments
				encoded_call.append(&mut a.encode());
				encoded_call.append(&mut b.encode());
				encoded_call.append(&mut c.encode());
				encoded_call
			}
			HrmpAvailableCalls::AcceptOpenChannel(a) => {
				let mut encoded_call: Vec<u8> = Vec::new();
				// pallet index
				encoded_call.push(relay_indices.hrmp);
				// call index
				encoded_call.push(relay_indices.accept_open_channel);
				// encoded argument
				encoded_call.append(&mut a.encode());
				encoded_call
			}
			HrmpAvailableCalls::CloseChannel(a) => {
				let mut encoded_call: Vec<u8> = Vec::new();
				// pallet index
				encoded_call.push(relay_indices.hrmp);
				// call index
				encoded_call.push(relay_indices.close_channel);
				// encoded argument
				encoded_call.append(&mut a.encode());
				encoded_call
			}
			HrmpAvailableCalls::CancelOpenRequest(a, b) => {
				let mut encoded_call: Vec<u8> = Vec::new();
				// pallet index
				encoded_call.push(relay_indices.hrmp);
				// call index
				encoded_call.push(relay_indices.cancel_open_request);
				// encoded argument
				encoded_call.append(&mut a.encode());
				encoded_call.append(&mut b.encode());
				encoded_call
			}
		};

		Ok(encoded_call)
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

impl<T: Config> Pallet<T> {
	pub fn encode_stake_call<Transactor: xcm_primitives::XcmTransact + Clone>(
		transactor: &Transactor,
		call: AvailableStakeCalls,
	) -> Result<Vec<u8>, xcm::latest::Error>
	where
		Transactor: Encode + Decode + EncodeLike<T::Transactor>,
	{
		// Get the chain indices for the specified transactor
		let chain_indices = ChainIndicesMap::<T>::get(transactor.clone())
			.ok_or(xcm::latest::Error::Unimplemented)?;

		let encoded = match chain_indices {
			ChainIndices::Relay(relay_indices) => {
				Self::encode_relay_stake_call(&relay_indices, call)
			}
			ChainIndices::AssetHub(assethub_indices) => {
				Self::encode_assethub_stake_call(&assethub_indices, call)
			}
		};

		Ok(encoded)
	}
}

// Legacy implementation kept for backwards compatibility
impl<T: Config> Pallet<T> {
	#[deprecated(note = "Use StakeEncodeCall::encode_call with transactor parameter instead")]
	pub fn encode_call_legacy(call: AvailableStakeCalls) -> Vec<u8> {
		let relay_indices = Self::get_relay_indices();
		Self::encode_relay_stake_call(&relay_indices, call)
	}
}

impl<T: Config> Pallet<T> {
	/// Encode staking call for Relay Chain using specific indices
	fn encode_relay_stake_call(
		indices: &crate::chain_indices::RelayChainIndices,
		call: AvailableStakeCalls,
	) -> Vec<u8> {
		match call {
			AvailableStakeCalls::Bond(b, c) => {
				let mut encoded_call = Vec::new();
				encoded_call.push(indices.staking);
				encoded_call.push(indices.bond);
				encoded_call.append(&mut encode_compact_arg(b));
				encoded_call.append(&mut c.encode());
				encoded_call
			}
			AvailableStakeCalls::BondExtra(a) => {
				let mut encoded_call = Vec::new();
				encoded_call.push(indices.staking);
				encoded_call.push(indices.bond_extra);
				encoded_call.append(&mut encode_compact_arg(a));
				encoded_call
			}
			AvailableStakeCalls::Unbond(a) => {
				let mut encoded_call = Vec::new();
				encoded_call.push(indices.staking);
				encoded_call.push(indices.unbond);
				encoded_call.append(&mut encode_compact_arg(a));
				encoded_call
			}
			AvailableStakeCalls::WithdrawUnbonded(a) => {
				let mut encoded_call = Vec::new();
				encoded_call.push(indices.staking);
				encoded_call.push(indices.withdraw_unbonded);
				encoded_call.append(&mut a.encode());
				encoded_call
			}
			AvailableStakeCalls::Validate(a) => {
				let mut encoded_call = Vec::new();
				encoded_call.push(indices.staking);
				encoded_call.push(indices.validate);
				encoded_call.append(&mut a.encode());
				encoded_call
			}
			AvailableStakeCalls::Chill => {
				let mut encoded_call = Vec::new();
				encoded_call.push(indices.staking);
				encoded_call.push(indices.chill);
				encoded_call
			}
			AvailableStakeCalls::SetPayee(a) => {
				let mut encoded_call = Vec::new();
				encoded_call.push(indices.staking);
				encoded_call.push(indices.set_payee);
				encoded_call.append(&mut a.encode());
				encoded_call
			}
			AvailableStakeCalls::SetController => {
				let mut encoded_call = Vec::new();
				encoded_call.push(indices.staking);
				encoded_call.push(indices.set_controller);
				encoded_call
			}
			AvailableStakeCalls::Rebond(a) => {
				let mut encoded_call = Vec::new();
				encoded_call.push(indices.staking);
				encoded_call.push(indices.rebond);
				encoded_call.append(&mut encode_compact_arg(a));
				encoded_call
			}
			AvailableStakeCalls::Nominate(a) => {
				let mut encoded_call = Vec::new();
				encoded_call.push(indices.staking);
				encoded_call.push(indices.nominate);
				let nominated: Vec<
					<AccountIdLookup<sp_runtime::AccountId32, ()> as StaticLookup>::Source,
				> = a.iter().map(|add| (*add).clone().into()).collect();
				encoded_call.append(&mut nominated.encode());
				encoded_call
			}
		}
	}

	/// Encode staking call for AssetHub using specific indices
	fn encode_assethub_stake_call(indices: &AssetHubIndices, call: AvailableStakeCalls) -> Vec<u8> {
		match call {
			AvailableStakeCalls::Bond(b, c) => {
				let mut encoded_call = Vec::new();
				encoded_call.push(indices.staking);
				encoded_call.push(indices.bond);
				encoded_call.append(&mut encode_compact_arg(b));
				encoded_call.append(&mut c.encode());
				encoded_call
			}
			AvailableStakeCalls::BondExtra(a) => {
				let mut encoded_call = Vec::new();
				encoded_call.push(indices.staking);
				encoded_call.push(indices.bond_extra);
				encoded_call.append(&mut encode_compact_arg(a));
				encoded_call
			}
			AvailableStakeCalls::Unbond(a) => {
				let mut encoded_call = Vec::new();
				encoded_call.push(indices.staking);
				encoded_call.push(indices.unbond);
				encoded_call.append(&mut encode_compact_arg(a));
				encoded_call
			}
			AvailableStakeCalls::WithdrawUnbonded(a) => {
				let mut encoded_call = Vec::new();
				encoded_call.push(indices.staking);
				encoded_call.push(indices.withdraw_unbonded);
				encoded_call.append(&mut a.encode());
				encoded_call
			}
			AvailableStakeCalls::Validate(a) => {
				// Note: Validate may not be supported on AssetHub
				// This encodes it anyway, but it may fail on the destination chain
				let mut encoded_call = Vec::new();
				encoded_call.push(indices.staking);
				encoded_call.push(indices.validate);
				encoded_call.append(&mut a.encode());
				encoded_call
			}
			AvailableStakeCalls::Chill => {
				let mut encoded_call = Vec::new();
				encoded_call.push(indices.staking);
				encoded_call.push(indices.chill);
				encoded_call
			}
			AvailableStakeCalls::SetPayee(a) => {
				let mut encoded_call = Vec::new();
				encoded_call.push(indices.staking);
				encoded_call.push(indices.set_payee);
				encoded_call.append(&mut a.encode());
				encoded_call
			}
			AvailableStakeCalls::SetController => {
				// Note: SetController is deprecated in newer Substrate versions
				let mut encoded_call = Vec::new();
				encoded_call.push(indices.staking);
				encoded_call.push(indices.set_controller);
				encoded_call
			}
			AvailableStakeCalls::Rebond(a) => {
				let mut encoded_call = Vec::new();
				encoded_call.push(indices.staking);
				encoded_call.push(indices.rebond);
				encoded_call.append(&mut encode_compact_arg(a));
				encoded_call
			}
			AvailableStakeCalls::Nominate(a) => {
				let mut encoded_call = Vec::new();
				encoded_call.push(indices.staking);
				encoded_call.push(indices.nominate);
				let nominated: Vec<
					<AccountIdLookup<sp_runtime::AccountId32, ()> as StaticLookup>::Source,
				> = a.iter().map(|add| (*add).clone().into()).collect();
				encoded_call.append(&mut nominated.encode());
				encoded_call
			}
		}
	}
}

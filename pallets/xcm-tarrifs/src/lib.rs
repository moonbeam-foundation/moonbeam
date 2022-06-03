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

//! # XCM Tarrifs Pallet
//!
//! XCM Tarrifs are a way of self-imposing a tax levied by a remote chain for that remote chain.
//!
//! This is accomplished by registering a tarrif for a given destination chain. This registered
//! terrif information will be used any time an XCM fragment is sent to that chain by charging the
//! specified fee and placing into an account reserved for that chain.
//!
//! This account is special much like a treasury account in that it has no private keys and can only
//! be controled via XCM instruuctions originating from that chain.

//! XXX Some other ideas...
//! * could allow terrif to be set only by remote chain
//!
//! TODO:
//! * how to handle ED?

#[pallet]
pub mod pallet {

	pub struct Tarrif {
		asset: MultiAsset,
		amount_per_weight: Option<u128>,
		amount_per_msg: Option<u128>,
	}

	// XXX pretend this is a type that uniquely represents a chain and can be used to derive an
	// account id
	pub type ChainId;


	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	/// Mapping of chainId to Tarrif
	#[pallet::storage]
	#[pallet::getter(fn tarrifs)]
	pub type Tarrifs<T: Config> = StorageMap<_, Blake2_128Concat, T::ChainId, Tarrif>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register a tarrif for a remote chain
		#[pallet::weight(0)]
		pub fn register_tarrif(
			origin: OriginFor<T>,
			chainId: T::ChainId,
			tarrif: &Tarrif
		) -> DispatchResult {
			// TODO
		}

		/// Send the fees to the chain's sovereign account
		#[pallet::weight(0)]
		pub fn withdraw_to_sovereign(
			origin: OriginFor<T>,
			chainId: T::ChainId,
			assets: MultiAssets
		) -> DispatchResult {
			// TODO:
			// 1. ensure that origin is the remote chain, this should only be possible through XCM
			// 2. derive remote chain's account id
			// 3. ensure each asset has minimum (make this whole thing atomic)
			// 4. transfer each asset
		}
	}
}

impl FeeManager for OurPallet {
	fn is_waived(_: Option<&MultiLocation>, reason: FeeReason) -> bool {
		// TODO: mod FeeManager::is_waived to take destination optionally

		// return whether or not we have a tarrif for this remote
	}

	fn handle_fee(origin: Option<&MultiLocation>, dest: Option<&MultiLocation>, fee: MultiAssets) {
		// TODO: how can we ensure `fee` has any assets we care about and enough of them? what does
		// XCM do to get these?
		//
		// 1. query tarrifs
		// 2. if exists, charge tarrif from fees
		// 3. emit event?
	}
}

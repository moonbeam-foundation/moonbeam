// This file is part of Substrate.

// Copyright (C) 2018-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Collection of common patterns for benchmarking comparison

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet;
pub use pallet::*;

#[cfg(any(test, feature = "runtime-benchmarks"))]
mod benchmarking;
#[cfg(test)]
pub(crate) mod mock;

pub mod weights;
use weights::WeightInfo;

#[pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::{pallet_prelude::*, WeightInfo};

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// Generated weights
		type WeightInfo: WeightInfo;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		U64Value(u64),
		U64Option(Option<u64>),
	}

	#[pallet::storage]
	#[pallet::getter(fn u64_value)]
	pub type U64Value<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn u64_option)]
	pub type U64Option<T: Config> = StorageValue<_, u64, OptionQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn get_u64_value(_origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			U64Value::<T>::get();
			Ok(().into())
		}
		#[pallet::weight(0)]
		pub fn put_u64_value(_origin: OriginFor<T>, input: u64) -> DispatchResultWithPostInfo {
			U64Value::<T>::put(input);
			Ok(().into())
		}
		#[pallet::weight(0)]
		pub fn get_put_u64_value(_origin: OriginFor<T>, input: u64) -> DispatchResultWithPostInfo {
			U64Value::<T>::get();
			U64Value::<T>::put(input);
			Ok(().into())
		}
		#[pallet::weight(0)]
		pub fn emit_u64_value_event(_origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			Self::deposit_event(Event::U64Value(0u64));
			Ok(().into())
		}
		#[pallet::weight(0)]
		pub fn get_emit_u64_value_event(_origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			Self::deposit_event(Event::U64Value(U64Value::<T>::get()));
			Ok(().into())
		}
		#[pallet::weight(0)]
		pub fn get_u64_option(_origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			U64Option::<T>::get();
			Ok(().into())
		}
		#[pallet::weight(0)]
		pub fn put_u64_option(_origin: OriginFor<T>, input: u64) -> DispatchResultWithPostInfo {
			U64Option::<T>::put(input);
			Ok(().into())
		}
		#[pallet::weight(0)]
		pub fn get_put_u64_option(_origin: OriginFor<T>, input: u64) -> DispatchResultWithPostInfo {
			U64Option::<T>::get();
			U64Option::<T>::put(input);
			Ok(().into())
		}
		#[pallet::weight(0)]
		pub fn emit_u64_option_event(_origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			Self::deposit_event(Event::U64Option(Some(1u64)));
			Ok(().into())
		}
		#[pallet::weight(0)]
		pub fn get_emit_u64_option_event(_origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			Self::deposit_event(Event::U64Option(U64Option::<T>::get()));
			Ok(().into())
		}
	}
}

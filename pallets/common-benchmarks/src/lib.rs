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
	use crate::WeightInfo;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

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

	#[pallet::storage]
	#[pallet::getter(fn u64_map)]
	pub type U64Map<T: Config> = StorageMap<_, Twox64Concat, u64, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn u64_double_map)]
	pub type U64DoubleMap<T: Config> =
		StorageDoubleMap<_, Twox64Concat, u64, Twox64Concat, u64, u64, ValueQuery>;

	// TODO: do different values cost different amounts for read/writes
	// --> structs with lots of fields vs a vec?
	// TODO: events separate and figure out how they compose with the types
	// TODO: add weight annotations to all the functions
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// VALUES
		#[pallet::weight(T::WeightInfo::get_u64_value())]
		pub fn get_u64_value(_origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			U64Value::<T>::get();
			Ok(().into())
		}
		#[pallet::weight(T::WeightInfo::put_u64_value())]
		pub fn put_u64_value(_origin: OriginFor<T>, input: u64) -> DispatchResultWithPostInfo {
			U64Value::<T>::put(input);
			Ok(().into())
		}
		#[pallet::weight(T::WeightInfo::get_put_u64_value())]
		pub fn get_put_u64_value(_origin: OriginFor<T>, input: u64) -> DispatchResultWithPostInfo {
			U64Value::<T>::get();
			U64Value::<T>::put(input);
			Ok(().into())
		}
		#[pallet::weight(T::WeightInfo::emit_u64_value_event())]
		pub fn emit_u64_value_event(_origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			Self::deposit_event(Event::U64Value(0u64));
			Ok(().into())
		}
		#[pallet::weight(T::WeightInfo::get_emit_u64_value_event())]
		pub fn get_emit_u64_value_event(_origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			Self::deposit_event(Event::U64Value(U64Value::<T>::get()));
			Ok(().into())
		}
		#[pallet::weight(T::WeightInfo::get_u64_option())]
		pub fn get_u64_option(_origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			U64Option::<T>::get();
			Ok(().into())
		}
		#[pallet::weight(T::WeightInfo::put_u64_option())]
		pub fn put_u64_option(_origin: OriginFor<T>, input: u64) -> DispatchResultWithPostInfo {
			U64Option::<T>::put(input);
			Ok(().into())
		}
		#[pallet::weight(T::WeightInfo::get_put_u64_option())]
		pub fn get_put_u64_option(_origin: OriginFor<T>, input: u64) -> DispatchResultWithPostInfo {
			U64Option::<T>::get();
			U64Option::<T>::put(input);
			Ok(().into())
		}
		#[pallet::weight(T::WeightInfo::emit_u64_option_event())]
		pub fn emit_u64_option_event(_origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			Self::deposit_event(Event::U64Option(Some(1u64)));
			Ok(().into())
		}
		#[pallet::weight(T::WeightInfo::get_emit_u64_option_event())]
		pub fn get_emit_u64_option_event(_origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			Self::deposit_event(Event::U64Option(U64Option::<T>::get()));
			Ok(().into())
		}
		// MAPS
		#[pallet::weight(T::WeightInfo::get_u64_map_value())]
		pub fn get_u64_map_value(_origin: OriginFor<T>, key: u64) -> DispatchResultWithPostInfo {
			U64Map::<T>::get(key);
			Ok(().into())
		}
		#[pallet::weight(T::WeightInfo::put_u64_map_value())]
		pub fn put_u64_map_value(
			_origin: OriginFor<T>,
			key: u64,
			value: u64,
		) -> DispatchResultWithPostInfo {
			U64Map::<T>::insert(key, value);
			Ok(().into())
		}
		#[pallet::weight(T::WeightInfo::get_put_u64_map_value())]
		pub fn get_put_u64_map_value(
			_origin: OriginFor<T>,
			key: u64,
			value: u64,
		) -> DispatchResultWithPostInfo {
			U64Map::<T>::get(key);
			U64Map::<T>::insert(key, value);
			Ok(().into())
		}
		// DOUBLE MAPS
		#[pallet::weight(T::WeightInfo::get_u64_double_map_value())]
		pub fn get_u64_double_map_value(
			_origin: OriginFor<T>,
			key_0: u64,
			key_1: u64,
		) -> DispatchResultWithPostInfo {
			U64DoubleMap::<T>::get(key_0, key_1);
			Ok(().into())
		}
		#[pallet::weight(T::WeightInfo::put_u64_double_map_value())]
		pub fn put_u64_double_map_value(
			_origin: OriginFor<T>,
			key_0: u64,
			key_1: u64,
			value: u64,
		) -> DispatchResultWithPostInfo {
			U64DoubleMap::<T>::insert(key_0, key_1, value);
			Ok(().into())
		}
		#[pallet::weight(T::WeightInfo::get_put_u64_double_map_value())]
		pub fn get_put_u64_double_map_value(
			_origin: OriginFor<T>,
			key_0: u64,
			key_1: u64,
			value: u64,
		) -> DispatchResultWithPostInfo {
			U64DoubleMap::<T>::get(key_0, key_1);
			U64DoubleMap::<T>::insert(key_0, key_1, value);
			Ok(().into())
		}
		// COUNTED MAPS
		// N MAPS
		// SIGNATURE VERIFICATION
	}
}

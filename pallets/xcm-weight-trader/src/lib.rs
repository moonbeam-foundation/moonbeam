// Copyright 2024 Moonbeam foundation
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

//! # A pallet to trade weight for XCM execution

#![allow(non_camel_case_types)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod weights;

pub use pallet::*;
pub use weights::WeightInfo;

use frame_support::pallet;
use frame_support::pallet_prelude::*;
use frame_support::traits::Contains;
use frame_support::weights::WeightToFee;
use frame_system::pallet_prelude::*;
use sp_runtime::traits::{Convert, Zero};
use xcm::latest::{
	Asset, AssetId as XcmAssetId, Error as XcmError, Fungibility, Location, XcmContext,
};
use xcm_executor::traits::{TransactAsset, WeightTrader};

#[pallet]
pub mod pallet {
	use super::*;

	/// Pallet for multi block migrations
	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Convert `T::AccountId` to `Location`.
		type AccountIdToLocation: Convert<Self::AccountId, Location>;

		/// Origin that is allowed to register a supported asset
		type AddSupportedAssetOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// A filter to forbid some XCM Location to be supported for fees.
		/// if you don't use it, put "Everything".
		type AssetLocationFilter: Contains<Location>;

		/// How to withdraw and deposit an asset.
		type AssetTransactor: TransactAsset;

		/// The native balance type.
		type Balance: TryInto<u128>;

		/// Origin that is allowed to edit a supported asset units per seconds
		type EditSupportedAssetOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// XCM Location for native curreny
		type NativeLocation: Get<Location>;

		/// Origin that is allowed to pause a supported asset
		type PauseSupportedAssetOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Origin that is allowed to remove a supported asset
		type RemoveSupportedAssetOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Origin that is allowed to unpause a supported asset
		type ResumeSupportedAssetOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// Convert a weight value into deductible native balance.
		type WeightToFee: WeightToFee<Balance = Self::Balance>;

		/// Account that will receive xcm fees
		type XcmFeesAccount: Get<Self::AccountId>;

		/// The benchmarks need a location that pass the filter AssetLocationFilter
		#[cfg(feature = "runtime-benchmarks")]
		type NotFilteredLocation: Get<Location>;
	}

	/// Stores all supported assets per XCM Location.
	/// The u128 is the number of units equal to one billion units of native currency.
	/// The boolean specify if the support for this asset is active
	#[pallet::storage]
	#[pallet::getter(fn supported_assets)]
	pub type SupportedAssets<T: Config> = StorageMap<_, Blake2_128Concat, Location, (bool, u128)>;

	#[pallet::error]
	pub enum Error<T> {
		/// The given asset was already added
		AssetAlreadyAdded,
		/// The given asset was already paused
		AssetAlreadyPaused,
		/// The given asset was not found
		AssetNotFound,
		/// The given asset is not paused
		AssetNotPaused,
		/// XCM location filtered
		XcmLocationFiltered,
		/// The units price cannot be zero
		UnitsCannotBeZero,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New supported asset is registered
		SupportedAssetAdded {
			location: Location,
			units_for_one_billion_native: u128,
		},
		/// Changed the amount of units we are charging per execution second for a given asset
		SupportedAssetEdited {
			location: Location,
			units_for_one_billion_native: u128,
		},
		/// Pause support for a given asset
		PauseAssetSupport { location: Location },
		/// Resume support for a given asset
		ResumeAssetSupport { location: Location },
		/// Supported asset type for fee payment removed
		SupportedAssetRemoved { location: Location },
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::add_asset())]
		pub fn add_asset(
			origin: OriginFor<T>,
			location: Location,
			units_for_one_billion_native: u128,
		) -> DispatchResult {
			T::AddSupportedAssetOrigin::ensure_origin(origin)?;

			ensure!(
				units_for_one_billion_native != 0,
				Error::<T>::UnitsCannotBeZero
			);
			ensure!(
				!SupportedAssets::<T>::contains_key(&location),
				Error::<T>::AssetAlreadyAdded
			);
			ensure!(
				T::AssetLocationFilter::contains(&location),
				Error::<T>::XcmLocationFiltered
			);

			SupportedAssets::<T>::insert(&location, (true, units_for_one_billion_native));

			Self::deposit_event(Event::SupportedAssetAdded {
				location,
				units_for_one_billion_native,
			});

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::edit_asset())]
		pub fn edit_asset(
			origin: OriginFor<T>,
			location: Location,
			units_for_one_billion_native: u128,
		) -> DispatchResult {
			T::EditSupportedAssetOrigin::ensure_origin(origin)?;

			ensure!(
				units_for_one_billion_native != 0,
				Error::<T>::UnitsCannotBeZero
			);
			ensure!(
				SupportedAssets::<T>::contains_key(&location),
				Error::<T>::AssetNotFound
			);

			SupportedAssets::<T>::insert(&location, (true, units_for_one_billion_native));

			Self::deposit_event(Event::SupportedAssetEdited {
				location,
				units_for_one_billion_native,
			});

			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::pause_asset_support())]
		pub fn pause_asset_support(origin: OriginFor<T>, location: Location) -> DispatchResult {
			T::PauseSupportedAssetOrigin::ensure_origin(origin)?;

			match SupportedAssets::<T>::get(&location) {
				Some((true, units_for_one_billion_native)) => {
					SupportedAssets::<T>::insert(&location, (false, units_for_one_billion_native));
					Self::deposit_event(Event::PauseAssetSupport { location });
					Ok(())
				}
				Some((false, _)) => Err(Error::<T>::AssetAlreadyPaused.into()),
				None => Err(Error::<T>::AssetNotFound.into()),
			}
		}

		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::resume_asset_support())]
		pub fn resume_asset_support(origin: OriginFor<T>, location: Location) -> DispatchResult {
			T::ResumeSupportedAssetOrigin::ensure_origin(origin)?;

			match SupportedAssets::<T>::get(&location) {
				Some((false, units_for_one_billion_native)) => {
					SupportedAssets::<T>::insert(&location, (true, units_for_one_billion_native));
					Self::deposit_event(Event::ResumeAssetSupport { location });
					Ok(())
				}
				Some((true, _)) => Err(Error::<T>::AssetNotPaused.into()),
				None => Err(Error::<T>::AssetNotFound.into()),
			}
		}

		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::remove_asset())]
		pub fn remove_asset(origin: OriginFor<T>, location: Location) -> DispatchResult {
			T::RemoveSupportedAssetOrigin::ensure_origin(origin)?;

			ensure!(
				SupportedAssets::<T>::contains_key(&location),
				Error::<T>::AssetNotFound
			);

			SupportedAssets::<T>::remove(&location);

			Self::deposit_event(Event::SupportedAssetRemoved { location });

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn get_asset_units_for_one_billion_native(location: &Location) -> Option<u128> {
			if let Some((true, ratio)) = SupportedAssets::<T>::get(location) {
				Some(ratio)
			} else {
				None
			}
		}
	}
}

struct Trader<T: crate::Config>(Weight, Option<Asset>, core::marker::PhantomData<T>);

impl<T: crate::Config> WeightTrader for Trader<T> {
	fn new() -> Self {
		Self(Weight::zero(), None, PhantomData)
	}
	fn buy_weight(
		&mut self,
		weight: Weight,
		payment: xcm_executor::AssetsInHolding,
		context: &XcmContext,
	) -> Result<xcm_executor::AssetsInHolding, XcmError> {
		log::trace!(target: "xcm::weight", "UsingComponents::buy_weight weight: {:?}, payment: {:?}, context: {:?}", weight, payment, context);

		// Can only call one time
		if self.1.is_some() {
			return Err(XcmError::NotWithdrawable);
		}

		// Consistency check for tests only, we should never panic in release mode
		debug_assert_eq!(self.0, Weight::zero());

		// We support only one fee asset per buy, so we take the first one.
		let first_asset = payment
			.clone()
			.fungible_assets_iter()
			.next()
			.ok_or(XcmError::AssetNotFound)?;

		match (first_asset.id, first_asset.fun) {
			(XcmAssetId(location), Fungibility::Fungible(_)) => {
				let amount: u128 = if location == <T as crate::Config>::NativeLocation::get() {
					<T as crate::Config>::WeightToFee::weight_to_fee(&weight)
						.try_into()
						.map_err(|_| XcmError::Overflow)?
				} else if let Some(units_for_one_billion_native) =
					Pallet::<T>::get_asset_units_for_one_billion_native(&location)
				{
					let native_amount: u128 =
						<T as crate::Config>::WeightToFee::weight_to_fee(&weight)
							.try_into()
							.map_err(|_| XcmError::Overflow)?;
					units_for_one_billion_native * native_amount / 1_000_000_000u128
				} else {
					return Err(XcmError::AssetNotFound);
				};

				// We dont need to proceed if the amount is 0
				// For cases (specially tests) where the asset is very cheap with respect
				// to the weight needed
				if amount.is_zero() {
					return Ok(payment);
				}

				let required = Asset {
					fun: Fungibility::Fungible(amount),
					id: XcmAssetId(location),
				};
				let unused = payment
					.checked_sub(required.clone())
					.map_err(|_| XcmError::TooExpensive)?;

				self.0 = weight;
				self.1 = Some(required);

				return Ok(unused);
			}
			_ => return Err(XcmError::AssetNotFound),
		}
	}

	fn refund_weight(&mut self, weight: Weight, context: &XcmContext) -> Option<Asset> {
		log::trace!(
			target: "xcm-weight-trader",
			"refund_weight weight: {:?}, context: {:?}, available weight: {:?}, asset: {:?}",
			weight,
			context,
			self.0,
			self.1
		);
		let maybe_asset_to_refound = if let Some(Asset {
			fun: Fungibility::Fungible(initial_amount),
			id: XcmAssetId(location),
		}) = &self.1
		{
			let weight = weight.min(self.0);
			let amount: u128 = <T as crate::Config>::WeightToFee::weight_to_fee(&weight)
				.try_into()
				.unwrap_or(u128::MAX);
			let final_amount = amount.min(*initial_amount);
			let amount_to_refound = initial_amount.saturating_sub(final_amount);
			self.0 -= weight;
			log::trace!(target: "xcm-weight-trader", "refund_weight amount to refund: {:?}", amount_to_refound);
			Some(Asset {
				fun: Fungibility::Fungible(amount_to_refound),
				id: XcmAssetId(location.clone()),
			})
		} else {
			None
		};
		self.1 = maybe_asset_to_refound.clone();
		maybe_asset_to_refound
	}
}

impl<T: crate::Config> Drop for Trader<T> {
	fn drop(&mut self) {
		if let Some(asset) = self.1.take() {
			let res = T::AssetTransactor::deposit_asset(
				&asset,
				&T::AccountIdToLocation::convert(T::XcmFeesAccount::get()),
				None,
			);
			debug_assert!(res.is_ok());
		}
	}
}

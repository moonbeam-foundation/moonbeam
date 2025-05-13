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
use frame_support::traits::fungible::NativeOrWithId;
use frame_support::traits::tokens::ConversionFromAssetBalance;
use frame_support::traits::Contains;
use frame_support::weights::WeightToFee;
use frame_system::pallet_prelude::*;
use moonbeam_core_primitives::{AssetId, Balance};
use sp_runtime::traits::MaybeEquivalence;
use sp_runtime::traits::{Convert, Zero};
use sp_std::vec::Vec;
use xcm::v5::{Asset, AssetId as XcmAssetId, Error as XcmError, Fungibility, Location, XcmContext};
use xcm::{IntoVersion, VersionedAssetId};
use xcm_executor::traits::{TransactAsset, WeightTrader};
use xcm_runtime_apis::fees::Error as XcmPaymentApiError;

pub const RELATIVE_PRICE_DECIMALS: u32 = 18;

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

		/// How to get an asset location from an asset id.
		///
		/// For XcmWeightTrader to convert between native
		/// and other assets' balances, it needs a way to map
		/// between locations (which are stored in this pallet)
		/// and asset ids (which are not). This type is responsible
		/// for providing the required mapping.
		type AssetIdentifier: MaybeEquivalence<Location, AssetId>;

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
		type WeightToFee: WeightToFee<Balance = <Self as pallet::Config>::Balance>;

		/// Account that will receive xcm fees
		type XcmFeesAccount: Get<Self::AccountId>;

		/// The benchmarks need a location that pass the filter AssetLocationFilter
		#[cfg(feature = "runtime-benchmarks")]
		type NotFilteredLocation: Get<Location>;

		/// The benchmarks need a way to create and add an asset
		/// to the supported assets list in order to ensure
		/// a conversion will be successful.
		#[cfg(feature = "runtime-benchmarks")]
		type AssetCreator: pallet_moonbeam_foreign_assets::AssetCreate;
	}

	/// Stores all supported assets per XCM Location.
	/// The u128 is the asset price relative to native asset with 18 decimals
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
		/// The relative price cannot be zero
		PriceCannotBeZero,
		/// The relative price calculation overflowed
		PriceOverflow,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New supported asset is registered
		SupportedAssetAdded {
			location: Location,
			relative_price: u128,
		},
		/// Changed the amount of units we are charging per execution second for a given asset
		SupportedAssetEdited {
			location: Location,
			relative_price: u128,
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
		#[pallet::weight(<T as pallet::Config>::WeightInfo::add_asset())]
		pub fn add_asset(
			origin: OriginFor<T>,
			location: Location,
			relative_price: u128,
		) -> DispatchResult {
			T::AddSupportedAssetOrigin::ensure_origin(origin)?;

			ensure!(relative_price != 0, Error::<T>::PriceCannotBeZero);
			ensure!(
				!SupportedAssets::<T>::contains_key(&location),
				Error::<T>::AssetAlreadyAdded
			);
			ensure!(
				T::AssetLocationFilter::contains(&location),
				Error::<T>::XcmLocationFiltered
			);

			SupportedAssets::<T>::insert(&location, (true, relative_price));

			Self::deposit_event(Event::SupportedAssetAdded {
				location,
				relative_price,
			});

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::edit_asset())]
		pub fn edit_asset(
			origin: OriginFor<T>,
			location: Location,
			relative_price: u128,
		) -> DispatchResult {
			T::EditSupportedAssetOrigin::ensure_origin(origin)?;

			ensure!(relative_price != 0, Error::<T>::PriceCannotBeZero);

			let enabled = SupportedAssets::<T>::get(&location)
				.ok_or(Error::<T>::AssetNotFound)?
				.0;

			SupportedAssets::<T>::insert(&location, (enabled, relative_price));

			Self::deposit_event(Event::SupportedAssetEdited {
				location,
				relative_price,
			});

			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::pause_asset_support())]
		pub fn pause_asset_support(origin: OriginFor<T>, location: Location) -> DispatchResult {
			T::PauseSupportedAssetOrigin::ensure_origin(origin)?;

			match SupportedAssets::<T>::get(&location) {
				Some((true, relative_price)) => {
					SupportedAssets::<T>::insert(&location, (false, relative_price));
					Self::deposit_event(Event::PauseAssetSupport { location });
					Ok(())
				}
				Some((false, _)) => Err(Error::<T>::AssetAlreadyPaused.into()),
				None => Err(Error::<T>::AssetNotFound.into()),
			}
		}

		#[pallet::call_index(3)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::resume_asset_support())]
		pub fn resume_asset_support(origin: OriginFor<T>, location: Location) -> DispatchResult {
			T::ResumeSupportedAssetOrigin::ensure_origin(origin)?;

			match SupportedAssets::<T>::get(&location) {
				Some((false, relative_price)) => {
					SupportedAssets::<T>::insert(&location, (true, relative_price));
					Self::deposit_event(Event::ResumeAssetSupport { location });
					Ok(())
				}
				Some((true, _)) => Err(Error::<T>::AssetNotPaused.into()),
				None => Err(Error::<T>::AssetNotFound.into()),
			}
		}

		#[pallet::call_index(4)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::remove_asset())]
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
		pub fn get_asset_relative_price(location: &Location) -> Option<u128> {
			if let Some((true, ratio)) = SupportedAssets::<T>::get(location) {
				Some(ratio)
			} else {
				None
			}
		}
		pub fn query_acceptable_payment_assets(
			xcm_version: xcm::Version,
		) -> Result<Vec<VersionedAssetId>, XcmPaymentApiError> {
			let v5_assets = [VersionedAssetId::from(XcmAssetId::from(
				T::NativeLocation::get(),
			))]
			.into_iter()
			.chain(
				SupportedAssets::<T>::iter().filter_map(|(asset_location, (enabled, _))| {
					enabled.then(|| VersionedAssetId::from(XcmAssetId(asset_location)))
				}),
			)
			.collect::<Vec<_>>();

			match xcm_version {
				xcm::v3::VERSION => v5_assets
					.into_iter()
					.map(|v5_asset| v5_asset.into_version(xcm::v3::VERSION))
					.collect::<Result<_, _>>()
					.map_err(|_| XcmPaymentApiError::VersionedConversionFailed),
				xcm::v4::VERSION => v5_assets
					.into_iter()
					.map(|v5_asset| v5_asset.into_version(xcm::v4::VERSION))
					.collect::<Result<_, _>>()
					.map_err(|_| XcmPaymentApiError::VersionedConversionFailed),
				xcm::v5::VERSION => Ok(v5_assets),
				_ => Err(XcmPaymentApiError::UnhandledXcmVersion),
			}
		}
		pub fn query_weight_to_asset_fee(
			weight: Weight,
			asset: VersionedAssetId,
		) -> Result<u128, XcmPaymentApiError> {
			if let VersionedAssetId::V5(XcmAssetId(asset_location)) = asset
				.into_version(xcm::latest::VERSION)
				.map_err(|_| XcmPaymentApiError::VersionedConversionFailed)?
			{
				Trader::<T>::compute_amount_to_charge(&weight, &asset_location).map_err(|e| match e
				{
					XcmError::AssetNotFound => XcmPaymentApiError::AssetNotFound,
					_ => XcmPaymentApiError::WeightNotComputable,
				})
			} else {
				Err(XcmPaymentApiError::UnhandledXcmVersion)
			}
		}
		#[cfg(any(feature = "std", feature = "runtime-benchmarks"))]
		pub fn set_asset_price(asset_location: Location, relative_price: u128) {
			SupportedAssets::<T>::insert(&asset_location, (true, relative_price));
		}
	}
}

pub struct Trader<T: crate::Config>(Weight, Option<Asset>, core::marker::PhantomData<T>);

impl<T: crate::Config> Trader<T> {
	fn compute_amount_to_charge(
		weight: &Weight,
		asset_location: &Location,
	) -> Result<u128, XcmError> {
		if *asset_location == <T as crate::Config>::NativeLocation::get() {
			<T as crate::Config>::WeightToFee::weight_to_fee(&weight)
				.try_into()
				.map_err(|_| XcmError::Overflow)
		} else if let Some(relative_price) = Pallet::<T>::get_asset_relative_price(asset_location) {
			if relative_price == 0u128 {
				Ok(0u128)
			} else {
				let native_amount: u128 = <T as crate::Config>::WeightToFee::weight_to_fee(&weight)
					.try_into()
					.map_err(|_| XcmError::Overflow)?;
				Ok(native_amount
					.checked_mul(10u128.pow(RELATIVE_PRICE_DECIMALS))
					.ok_or(XcmError::Overflow)?
					.checked_div(relative_price)
					.ok_or(XcmError::Overflow)?)
			}
		} else {
			Err(XcmError::AssetNotFound)
		}
	}
}

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
		log::trace!(
			target: "xcm::weight",
			"UsingComponents::buy_weight weight: {:?}, payment: {:?}, context: {:?}",
			weight,
			payment,
			context
		);

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
				let amount: u128 = Self::compute_amount_to_charge(&weight, &location)?;

				// We don't need to proceed if the amount is 0
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

				Ok(unused)
			}
			_ => Err(XcmError::AssetNotFound),
		}
	}

	fn refund_weight(&mut self, actual_weight: Weight, context: &XcmContext) -> Option<Asset> {
		log::trace!(
			target: "xcm-weight-trader",
			"refund_weight weight: {:?}, context: {:?}, available weight: {:?}, asset: {:?}",
			actual_weight,
			context,
			self.0,
			self.1
		);
		if let Some(Asset {
			fun: Fungibility::Fungible(initial_amount),
			id: XcmAssetId(location),
		}) = self.1.take()
		{
			if actual_weight == self.0 {
				self.1 = Some(Asset {
					fun: Fungibility::Fungible(initial_amount),
					id: XcmAssetId(location),
				});
				None
			} else {
				let weight = actual_weight.min(self.0);
				let amount: u128 =
					Self::compute_amount_to_charge(&weight, &location).unwrap_or(u128::MAX);
				let final_amount = amount.min(initial_amount);
				let amount_to_refund = initial_amount.saturating_sub(final_amount);
				self.0 -= weight;
				self.1 = Some(Asset {
					fun: Fungibility::Fungible(final_amount),
					id: XcmAssetId(location.clone()),
				});
				log::trace!(
					target: "xcm-weight-trader",
					"refund_weight amount to refund: {:?}",
					amount_to_refund
				);
				Some(Asset {
					fun: Fungibility::Fungible(amount_to_refund),
					id: XcmAssetId(location),
				})
			}
		} else {
			None
		}
	}
}

impl<T: crate::Config> Drop for Trader<T> {
	fn drop(&mut self) {
		log::trace!(
			target: "xcm-weight-trader",
			"Dropping `Trader` instance: (weight: {:?}, asset: {:?})",
			&self.0,
			&self.1
		);
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

impl<T: Config> ConversionFromAssetBalance<Balance, NativeOrWithId<AssetId>, Balance>
	for Pallet<T>
{
	type Error = Error<T>;
	fn from_asset_balance(
		asset_balance: Balance,
		asset_kind: NativeOrWithId<AssetId>,
	) -> Result<Balance, Error<T>> {
		match asset_kind {
			NativeOrWithId::Native => Ok(asset_balance),
			NativeOrWithId::WithId(asset_id) => {
				let location =
					T::AssetIdentifier::convert_back(&asset_id).ok_or(Error::<T>::AssetNotFound)?;
				let relative_price = Pallet::<T>::get_asset_relative_price(&location)
					.ok_or(Error::<T>::AssetNotFound)?;
				Ok(asset_balance
					.checked_mul(relative_price)
					.ok_or(Error::<T>::PriceOverflow)?
					.checked_div(10u128.pow(RELATIVE_PRICE_DECIMALS))
					.ok_or(Error::<T>::PriceOverflow)?)
			}
		}
	}

	/// Set a conversion rate to `1` for the `asset_id`.
	#[cfg(feature = "runtime-benchmarks")]
	fn ensure_successful(asset_id: NativeOrWithId<AssetId>) {
		use frame_support::{assert_ok, traits::OriginTrait};
		use pallet_moonbeam_foreign_assets::AssetCreate;
		use xcm::opaque::v5::Junction::Parachain;
		match asset_id {
			NativeOrWithId::Native => (),
			NativeOrWithId::WithId(asset_id) => {
				if let None = T::AssetIdentifier::convert_back(&asset_id) {
					let location = Location::new(1, [Parachain(1000)]);
					let root = <T as frame_system::Config>::RuntimeOrigin::root();

					assert_ok!(T::AssetCreator::create_asset(asset_id, location.clone()));

					assert_ok!(Self::add_asset(root, location.clone(), 1u128,));
				}
			}
		}
	}
}

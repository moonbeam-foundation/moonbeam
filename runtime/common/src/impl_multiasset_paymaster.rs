// Copyright 2025 Moonbeam foundation
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

use frame_support::traits::OriginTrait;
use frame_support::traits::{fungible::NativeOrWithId, tokens::Pay};
use moonbeam_core_primitives::{AssetId, Balance};
use pallet_moonbeam_foreign_assets::AssetsById;
use sp_core::U256;
use sp_runtime::{traits::StaticLookup, DispatchError};

pub struct MultiAssetPaymaster<R>(sp_std::marker::PhantomData<R>);
impl<R> Pay for MultiAssetPaymaster<R>
where
	R: frame_system::Config
		+ pallet_treasury::Config
		+ pallet_balances::Config
		+ pallet_asset_manager::Config
		+ pallet_assets::Config
		+ pallet_moonbeam_foreign_assets::Config
		+ pallet_xcm_weight_trader::Config,
{
	type Balance = Balance;
	type Beneficiary = R::AccountId;
	type AssetKind = NativeOrWithId<AssetId>;
	type Id = ();
	type Error = DispatchError;
	fn pay(
		who: &Self::Beneficiary,
		asset_kind: Self::AssetKind,
		amount: Self::Balance,
	) -> Result<Self::Id, Self::Error> {
		match asset_kind {
			Self::AssetKind::Native => {
				let dest = R::Lookup::unlookup(who.clone());
				let value = <R as pallet_balances::Config>::Balance::try_from(amount)
					.map_err(|_| pallet_treasury::Error::<R>::PayoutError)?;
				// Pay account with native balance
				pallet_balances::Pallet::<R>::transfer_keep_alive(
					R::RuntimeOrigin::signed(pallet_treasury::Pallet::<R>::account_id()),
					dest,
					value,
				)?;
				Ok(())
			}
			Self::AssetKind::WithId(id) => {
				// Check in the foreign assets first
				if let Some(_asset_loc) = AssetsById::<R>::get(id) {
					// Pay if asset found
					pallet_moonbeam_foreign_assets::Pallet::<R>::transfer(
						id,
						pallet_treasury::Pallet::<R>::account_id(),
						who.clone(),
						U256::from(amount as u128),
					)
					.map_err(|_| pallet_treasury::Error::<R>::PayoutError)?;
					return Ok(());
				}
				Err(pallet_moonbeam_foreign_assets::Error::<R>::AssetDoesNotExist.into())
			}
		}
	}

	fn check_payment(_id: Self::Id) -> frame_support::traits::tokens::PaymentStatus {
		frame_support::traits::tokens::PaymentStatus::Success
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn ensure_successful(_: &Self::Beneficiary, asset: Self::AssetKind, amount: Self::Balance) {
		use frame_support::traits::fungible::Mutate;
		use xcm::opaque::v4::Junction::Parachain;
		use xcm::v4::Location;
		let treasury = pallet_treasury::Pallet::<R>::account_id();
		match asset {
			Self::AssetKind::Native => {
				pallet_balances::Pallet::<R>::mint_into(
					&treasury,
					<R as pallet_balances::Config>::Balance::try_from(amount)
						.map_err(|_| pallet_treasury::Error::<R>::PayoutError)
						.unwrap(),
				)
				.expect("failed to mint the native asset into the treasury account");
			}
			Self::AssetKind::WithId(id) => {
				// Check if asset exists & create if required
				if let None = AssetsById::<R>::get(id) {
					let location = Location::new(1, [Parachain(1000)]);
					pallet_moonbeam_foreign_assets::Pallet::<R>::do_create_asset(
						id,
						location,
						18,
						"DEV".as_bytes().to_vec().try_into().expect("too long"),
						"DEV".as_bytes().to_vec().try_into().expect("too long"),
						None,
					)
					.expect("failed to create asset");
				}
				// Fund treasury account
				pallet_moonbeam_foreign_assets::Pallet::<R>::mint_into(
					id,
					treasury,
					U256::from(amount as u128),
				)
				.expect("failed to mint asset into treasury account");

				pallet_xcm_weight_trader::Pallet::<R>::add_asset(
					R::RuntimeOrigin::root(),
					Location::new(1, [Parachain(1000)]),
					1,
				)
				.expect("failed to register asset relative price in xcm weight trader");
			}
		}
	}
	#[cfg(feature = "runtime-benchmarks")]
	fn ensure_concluded(_: Self::Id) {}
}

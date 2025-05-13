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

use frame_support::traits::fungible;
use frame_support::traits::{
	fungible::NativeOrWithId,
	tokens::{Pay, Preservation::Expendable},
};
use moonbeam_core_primitives::{AssetId, Balance};
use sp_core::{Get, U256};
use sp_runtime::DispatchError;

pub struct MultiAssetPaymaster<R, TreasuryAccount, NativeAsset, ForeignAssets>(
	sp_std::marker::PhantomData<(R, TreasuryAccount, NativeAsset, ForeignAssets)>,
);
impl<R, TreasuryAccount, NativeAsset, ForeignAssets> Pay
	for MultiAssetPaymaster<R, TreasuryAccount, NativeAsset, ForeignAssets>
where
	R: frame_system::Config,
	TreasuryAccount: Get<R::AccountId>,
	NativeAsset: fungible::Mutate<R::AccountId> + fungible::Inspect<R::AccountId>,
	ForeignAssets: pallet_moonbeam_foreign_assets::SimpleMutate<R>
		+ pallet_moonbeam_foreign_assets::SimpleAssetExists,
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
				<NativeAsset as fungible::Mutate<_>>::transfer(
					&TreasuryAccount::get(),
					who,
					amount
						.try_into()
						.map_err(|_| DispatchError::Other("failed to convert amount"))?,
					Expendable,
				)?;
				Ok(())
			}
			Self::AssetKind::WithId(id) => {
				// Check in the foreign assets first
				if ForeignAssets::asset_exists(id) {
					// Pay if asset found
					ForeignAssets::transfer_asset(
						id,
						TreasuryAccount::get(),
						who.clone(),
						U256::from(amount as u128),
					)?;
					return Ok(());
				}
				Err(DispatchError::Other("asset not found"))
			}
		}
	}

	fn check_payment(_id: Self::Id) -> frame_support::traits::tokens::PaymentStatus {
		frame_support::traits::tokens::PaymentStatus::Success
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn ensure_successful(
		beneficiary: &Self::Beneficiary,
		asset: Self::AssetKind,
		amount: Self::Balance,
	) {
		use pallet_xcm_weight_trader::RELATIVE_PRICE_DECIMALS;
		use xcm::opaque::v4::Junction::Parachain;
		use xcm::v4::Location;
		let treasury = TreasuryAccount::get();
		match asset {
			Self::AssetKind::Native => {
				<NativeAsset as fungible::Mutate<_>>::mint_into(&treasury, (amount as u32).into());
			}
			Self::AssetKind::WithId(id) => {
				// Fund treasury account
				ForeignAssets::mint_asset(id, treasury, U256::from(amount as u128))
					.expect("failed to mint asset into treasury account");
			}
		}
	}
	#[cfg(feature = "runtime-benchmarks")]
	fn ensure_concluded(_: Self::Id) {}
}

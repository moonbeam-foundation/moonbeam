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

use super::*;
use frame_support::traits::tokens::{DepositConsequence, Provenance, WithdrawConsequence};
use moonbeam_core_primitives::{AssetId, Balance};
use sp_runtime::{traits::Convert, SaturatedConversion};

impl<T: Config> Inspect<T::AccountId> for Pallet<T> {
	type AssetId = AssetId;
	type Balance = Balance;

	fn total_issuance(asset: Self::AssetId) -> Self::Balance {
		let total_supply = EvmCaller::<T>::erc20_total_supply(asset).unwrap_or(U256::zero());
		let as_u128 = total_supply.saturated_into::<u128>();
		Self::Balance::from(as_u128)
	}

	fn minimum_balance(_asset: Self::AssetId) -> Self::Balance {
		Self::Balance::from(0u128)
	}

	fn total_balance(asset: Self::AssetId, who: &T::AccountId) -> Self::Balance {
		let balance =
			EvmCaller::<T>::erc20_balance_of(asset, T::AccountIdToH160::convert(who.clone()))
				.unwrap_or(U256::zero());
		let as_u128 = balance.saturated_into::<u128>();
		Self::Balance::from(as_u128)
	}

	fn balance(asset: Self::AssetId, who: &T::AccountId) -> Self::Balance {
		let balance =
			EvmCaller::<T>::erc20_balance_of(asset, T::AccountIdToH160::convert(who.clone()))
				.unwrap_or(U256::zero());
		let as_u128 = balance.saturated_into::<u128>();
		Self::Balance::from(as_u128)
	}

	fn reducible_balance(
		asset: Self::AssetId,
		who: &T::AccountId,
		_preservation: frame_support::traits::tokens::Preservation,
		_force: frame_support::traits::tokens::Fortitude,
	) -> Self::Balance {
		let balance =
			EvmCaller::<T>::erc20_balance_of(asset, T::AccountIdToH160::convert(who.clone()))
				.unwrap_or(U256::zero());
		let as_u128 = balance.saturated_into::<u128>();
		Self::Balance::from(as_u128)
	}

	fn can_deposit(
		asset: Self::AssetId,
		_who: &T::AccountId,
		amount: Self::Balance,
		provenance: Provenance,
	) -> DepositConsequence {
		let Some(location) = AssetsById::<T>::get(&asset) else {
			return DepositConsequence::UnknownAsset;
		};
		let Some(asset_info) = AssetsByLocation::<T>::get(&location) else {
			return DepositConsequence::UnknownAsset;
		};
		let status = asset_info.1;
		// Check for total supply overflow
		if provenance == Provenance::Minted {
			let total_supply = EvmCaller::<T>::erc20_total_supply(asset).unwrap_or(U256::zero());
			let minted_amount = U256::from(amount);
			let Some(_new_total_supply) = total_supply.checked_add(minted_amount) else {
				return DepositConsequence::Overflow;
			};
		};
		match (status, provenance) {
			(AssetStatus::FrozenXcmDepositForbidden, _) => DepositConsequence::Blocked,
			(AssetStatus::FrozenXcmDepositAllowed, Provenance::Minted) => {
				DepositConsequence::Success
			}
			(AssetStatus::Active, _) => DepositConsequence::Success,
			(_, _) => DepositConsequence::Blocked,
		}
	}

	fn can_withdraw(
		asset: Self::AssetId,
		who: &T::AccountId,
		amount: Self::Balance,
	) -> WithdrawConsequence<Self::Balance> {
		if Self::asset_exists(asset) {
			let balance = Self::balance(asset, who);
			if balance >= Self::Balance::from(amount) {
				WithdrawConsequence::Success
			} else {
				WithdrawConsequence::BalanceLow
			}
		} else {
			WithdrawConsequence::UnknownAsset
		}
	}

	fn asset_exists(asset: Self::AssetId) -> bool {
		AssetsById::<T>::contains_key(&asset)
	}
}

impl<T: Config> Create<T::AccountId> for Pallet<T> {
	fn create(
		_id: Self::AssetId,
		_admin: T::AccountId,
		_is_sufficient: bool,
		_min_balance: Self::Balance,
	) -> sp_runtime::DispatchResult {
		sp_runtime::DispatchResult::Err(DispatchError::Other(
			"Not implemented, must create through create_foreign_asset",
		))
	}
}

impl<T: Config> Unbalanced<T::AccountId> for Pallet<T> {
	fn handle_dust(_dust: frame_support::traits::fungibles::Dust<T::AccountId, Self>) {}
	fn set_total_issuance(_asset: Self::AssetId, _amount: Self::Balance) {}
	fn write_balance(
		asset: Self::AssetId,
		who: &T::AccountId,
		amount: Self::Balance,
	) -> Result<Option<Self::Balance>, DispatchError> {
		let balance =
			EvmCaller::<T>::erc20_balance_of(asset, T::AccountIdToH160::convert(who.clone()))
				.unwrap_or(U256::zero());
		let contract_address = Pallet::<T>::contract_address_from_asset_id(asset);
		match (U256::from(amount), balance) {
			(amount, balance) if amount == balance => {
				let as_u128 = balance.saturated_into::<u128>();
				Ok(Some(Self::Balance::from(as_u128)))
			}
			(amount, balance) if amount > balance => {
				EvmCaller::<T>::erc20_mint_into(
					contract_address,
					T::AccountIdToH160::convert(who.clone()),
					U256::from(amount).saturating_sub(balance),
				)
				.map_err(|_| DispatchError::Other("Failed to mint into account"))?;
				let balance = EvmCaller::<T>::erc20_balance_of(
					asset,
					T::AccountIdToH160::convert(who.clone()),
				)
				.unwrap_or(U256::zero());
				let as_u128 = balance.saturated_into::<u128>();
				Ok(Some(Self::Balance::from(as_u128)))
			} // Add balance
			(amount, balance) if amount < balance => {
				EvmCaller::<T>::erc20_burn_from(
					contract_address,
					T::AccountIdToH160::convert(who.clone()),
					U256::from(balance).saturating_sub(amount),
				)
				.map_err(|_| DispatchError::Other("Failed to burn from account"))?;
				let balance = EvmCaller::<T>::erc20_balance_of(
					asset,
					T::AccountIdToH160::convert(who.clone()),
				)
				.unwrap_or(U256::zero());
				let as_u128 = balance.saturated_into::<u128>();
				Ok(Some(Self::Balance::from(as_u128)))
			}
			(_, _) => Err(DispatchError::Other("Invalid amount")),
		}
	}
}

impl<T: Config> Mutate<T::AccountId> for Pallet<T> {}

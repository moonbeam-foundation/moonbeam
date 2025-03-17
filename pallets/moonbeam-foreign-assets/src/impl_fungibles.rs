use super::*;
use frame_support::traits::tokens::{DepositConsequence, Provenance, WithdrawConsequence};
use moonbeam_core_primitives::{Balance, AssetId};
use sp_runtime::traits::Convert;

impl <T: Config> Inspect<T::AccountId>  for Pallet<T> {
    type AssetId = AssetId;
    type Balance = Balance;

    fn total_issuance(asset: Self::AssetId) ->  Self::Balance {
        let total_supply = EvmCaller::<T>::erc20_total_supply(asset).unwrap_or(U256::zero());
        let as_u128 = u128::try_from(total_supply).unwrap_or(u128::MAX);
        Self::Balance::from(as_u128)
    }

    fn minimum_balance(_asset: Self::AssetId) -> Self::Balance {
        Self::Balance::from(0u128)
    }	

    fn total_balance(asset: Self::AssetId, who: &T::AccountId) -> Self::Balance {
        let balance = EvmCaller::<T>::erc20_balance_of(asset, T::AccountIdToH160::convert(who.clone())).unwrap_or(U256::zero());
        let as_u128 = u128::try_from(balance).unwrap_or(u128::MAX);
        Self::Balance::from(as_u128)
    }

    fn balance(asset: Self::AssetId, who: &T::AccountId) -> Self::Balance {
        let balance = EvmCaller::<T>::erc20_balance_of(asset, T::AccountIdToH160::convert(who.clone())).unwrap_or(U256::zero());
        let as_u128 = u128::try_from(balance).unwrap_or(u128::MAX);
        Self::Balance::from(as_u128)
    }
    
    fn reducible_balance(
            asset: Self::AssetId,
            who: &T::AccountId,
            _preservation: frame_support::traits::tokens::Preservation,
            _force: frame_support::traits::tokens::Fortitude,
        ) -> Self::Balance {
            let balance = EvmCaller::<T>::erc20_balance_of(asset, T::AccountIdToH160::convert(who.clone())).unwrap_or(U256::zero());
            let as_u128 = u128::try_from(balance).unwrap_or(u128::MAX);
            Self::Balance::from(as_u128)
    }

    fn can_deposit(
            asset: Self::AssetId,
            _who: &T::AccountId,
            amount: Self::Balance,
            provenance: Provenance,
        ) -> DepositConsequence {
        let Some(location) = AssetsById::<T>::get(&asset) else { return DepositConsequence::UnknownAsset; };
        let Some(asset_info) = AssetsByLocation::<T>::get(&location) else { return DepositConsequence::UnknownAsset; };
        let status  = asset_info.1;
        // Check for total supply overflow
        if provenance == Provenance::Minted {
            let total_supply = EvmCaller::<T>::erc20_total_supply(asset).unwrap_or(U256::zero());
            let minted_amount = U256::from(amount);
            let Some(_new_total_supply) = total_supply.checked_add(minted_amount) else { return DepositConsequence::Overflow; };
        };
        match (status, provenance) {
            (AssetStatus::FrozenXcmDepositForbidden, _) => DepositConsequence::Blocked,
            (AssetStatus::FrozenXcmDepositAllowed, Provenance::Minted) => DepositConsequence::Success,
            (AssetStatus::Active, _) => DepositConsequence::Success,
            (_,_) => DepositConsequence::Blocked
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
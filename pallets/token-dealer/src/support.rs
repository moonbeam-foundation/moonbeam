use crate::CurrencyId;
use parity_scale_codec::FullCodec;
use sp_runtime::traits::{
	CheckedConversion, Convert, MaybeSerializeDeserialize, SaturatedConversion,
};
use sp_std::{
	cmp::{Eq, PartialEq},
	collections::btree_set::BTreeSet,
	convert::{TryFrom, TryInto},
	fmt::Debug,
	marker::PhantomData,
	prelude::*,
	result,
};
use xcm::v0::{Error, Junction, MultiAsset, MultiLocation, Result};
use xcm_executor::traits::{
	FilterAssetLocation, LocationConversion, MatchesFungible, NativeAsset, TransactAsset,
};

use frame_support::{
	debug,
	traits::{Currency, ExistenceRequirement, Get, WithdrawReasons},
};

pub trait AssetToCurrency {
	fn asset_to_currency(asset: &MultiAsset) -> Option<CurrencyId>;
}

pub struct CurrencyAdapter<
	NativeCurrency,
	TokenFactory,
	Matcher,
	AccountIdConverter,
	AccountId,
	CurrencyIdConverter,
	CurrencyId,
>(
	PhantomData<(
		NativeCurrency,
		TokenFactory,
		Matcher,
		AccountIdConverter,
		AccountId,
		CurrencyIdConverter,
		CurrencyId,
	)>,
);

impl<
		NativeCurrency: Currency<AccountId>,
		TokenFactory: token_factory::TokenFactory<Vec<u8>, AccountId, NativeCurrency::Balance>,
		Matcher: MatchesFungible<NativeCurrency::Balance>,
		AccountIdConverter: LocationConversion<AccountId>,
		AccountId: sp_std::fmt::Debug,
		CurrencyIdConverter: AssetToCurrency,
	> TransactAsset
	for CurrencyAdapter<
		NativeCurrency,
		TokenFactory,
		Matcher,
		AccountIdConverter,
		AccountId,
		CurrencyIdConverter,
		CurrencyId,
	>
{
	fn deposit_asset(asset: &MultiAsset, location: &MultiLocation) -> Result {
		debug::info!("------------------------------------------------");
		debug::info!(
			">>> trying deposit. asset: {:?}, location: {:?}",
			asset,
			location
		);
		let who = AccountIdConverter::from_location(location).ok_or(())?;
		debug::info!("who: {:?}", who);
		let currency = CurrencyIdConverter::asset_to_currency(asset).ok_or(())?;
		debug::info!("currency_id: {:?}", currency);
		let amount: NativeCurrency::Balance = Matcher::matches_fungible(&asset).ok_or(())?;
		debug::info!("amount: {:?}", amount);
		// match on currency variant
		if let CurrencyId::Token(token_id) = currency {
			// mint erc20 token to `who`
			TokenFactory::mint(token_id, who, amount).map_err(|_| ())?;
		} else {
			// native currency transfer via `frame/pallet_balances` is only other variant
			// TODO: does deposit_creating make sense? is there a cost for this function, who pays for it?
			NativeCurrency::deposit_creating(&who, amount);
		}
		debug::info!(">>> success deposit.");
		debug::info!("------------------------------------------------");
		Ok(())
	}

	fn withdraw_asset(
		asset: &MultiAsset,
		location: &MultiLocation,
	) -> result::Result<MultiAsset, Error> {
		debug::info!("------------------------------------------------");
		debug::info!(
			">>> trying withdraw. asset: {:?}, location: {:?}",
			asset,
			location
		);
		let who = AccountIdConverter::from_location(location).ok_or(())?;
		debug::info!("who: {:?}", who);
		let currency = CurrencyIdConverter::asset_to_currency(asset).ok_or(())?;
		debug::info!("currency_id: {:?}", currency);
		let amount: NativeCurrency::Balance = Matcher::matches_fungible(&asset).ok_or(())?;
		debug::info!("amount: {:?}", amount);
		// match on currency variant
		if let CurrencyId::Token(token_id) = currency {
			// burn erc20 token from `who`
			TokenFactory::burn(token_id, who, amount).map_err(|_| ())?;
		} else {
			// native currency transfer via `frame/pallet_balances` is only other variant
			// TODO: check if WithdrawReasons and ExistenceRequirement make sense
			NativeCurrency::withdraw(
				&who,
				amount,
				WithdrawReasons::TRANSFER,
				ExistenceRequirement::AllowDeath,
			)
			.map_err(|_| ())?;
		}
		debug::info!(">>> success withdraw.");
		debug::info!("------------------------------------------------");
		Ok(asset.clone())
	}
}

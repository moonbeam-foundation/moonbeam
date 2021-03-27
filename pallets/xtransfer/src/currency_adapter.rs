use crate::support::CurrencyIdConversion;
use frame_support::{
	debug,
	traits::{Currency, ExistenceRequirement, WithdrawReasons},
};
use sp_std::{marker::PhantomData, prelude::*, result};
use token_factory::{CurrencyId, Ticker};
use xcm::v0::{Error, MultiAsset, MultiLocation, Result as XcmResult};
use xcm_executor::traits::{LocationConversion, MatchesFungible, TransactAsset};

/// The handler for processing cross-chain messages
pub struct MultiCurrencyAdapter<
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
		TokenFactory: token_factory::TokenMinter<Ticker, AccountId, NativeCurrency::Balance>,
		Matcher: MatchesFungible<NativeCurrency::Balance>,
		AccountIdConverter: LocationConversion<AccountId>,
		AccountId: sp_std::fmt::Debug + Clone,
		CurrencyIdConverter: CurrencyIdConversion<CurrencyId>,
	> TransactAsset
	for MultiCurrencyAdapter<
		NativeCurrency,
		TokenFactory,
		Matcher,
		AccountIdConverter,
		AccountId,
		CurrencyIdConverter,
		CurrencyId,
	>
{
	fn deposit_asset(asset: &MultiAsset, location: &MultiLocation) -> XcmResult {
		debug::info!("------------------------------------------------");
		debug::info!(
			">>> trying deposit. asset: {:?}, location: {:?}",
			asset,
			location
		);
		let who = AccountIdConverter::from_location(location).ok_or(())?;
		debug::info!("who: {:?}", who);
		let currency = CurrencyIdConverter::from_asset(asset).ok_or(())?;
		debug::info!("currency_id: {:?}", currency);
		let amount: NativeCurrency::Balance = Matcher::matches_fungible(&asset).ok_or(())?;
		debug::info!("amount: {:?}", amount);
		// match on currency variant
		if let CurrencyId::Token(token_id) = currency {
			// mint erc20 token to `who`
			TokenFactory::mint(token_id, who.clone(), amount).map_err(|error| {
				debug::info!(
					"Token factory `mint` failed
					\n token_id: {:?}\n who: {:?}\n amount: {:?}\n error: {:?}",
					token_id,
					who,
					amount,
					error
				);
				()
			})?;
		} else {
			// native currency transfer via `frame/pallet_balances` is only other variant
			NativeCurrency::deposit_creating(&who, amount);
		}
		debug::info!(">>> successful deposit.");
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
		let currency = CurrencyIdConverter::from_asset(asset).ok_or(())?;
		debug::info!("currency_id: {:?}", currency);
		let amount: NativeCurrency::Balance = Matcher::matches_fungible(&asset).ok_or(())?;
		debug::info!("amount: {:?}", amount);
		// match on currency variant
		if let CurrencyId::Token(token_id) = currency {
			// burn erc20 token from `who`
			TokenFactory::burn(token_id, who.clone(), amount).map_err(|error| {
				debug::info!(
					"Token factory `burn` failed
					\n token_id: {:?}\n who: {:?}\n amount: {:?}\n error: {:?}",
					token_id,
					who,
					amount,
					error
				);
				()
			})?;
		} else {
			// native currency transfer via `frame/pallet_balances` is only other variant
			NativeCurrency::withdraw(
				&who,
				amount,
				WithdrawReasons::TRANSFER,
				ExistenceRequirement::AllowDeath,
			)
			.map_err(|error| {
				debug::info!(
					"Native currency `withdraw` failed\n who: {:?}\n amount: {:?}\n error: {:?}",
					who,
					amount,
					error
				);
				()
			})?;
		}
		debug::info!(">>> successful withdraw.");
		debug::info!("------------------------------------------------");
		Ok(asset.clone())
	}
}

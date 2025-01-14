use frame_support::__private::Get;
use frame_support::pallet_prelude::TypedGet;
use frame_support::traits::fungible::Credit;
use frame_support::traits::tokens::imbalance::ResolveTo;
use frame_support::traits::Imbalance;
use frame_support::traits::OnUnbalanced;
use pallet_treasury::TreasuryAccountId;
use sp_runtime::Perbill;

/// Deal with substrate based fees and tip. This should be used with pallet_transaction_payment.
pub struct DealWithSubstrateFeesAndTip<R, FeesTreasuryProportion>(
	sp_std::marker::PhantomData<(R, FeesTreasuryProportion)>,
);
impl<R, FeesTreasuryProportion> DealWithSubstrateFeesAndTip<R, FeesTreasuryProportion>
where
	R: pallet_balances::Config + pallet_treasury::Config + pallet_author_inherent::Config,
	pallet_author_inherent::Pallet<R>: Get<R::AccountId>,
	FeesTreasuryProportion: Get<Perbill>,
{
	fn deal_with_fees(amount: Credit<R::AccountId, pallet_balances::Pallet<R>>) {
		// Balances pallet automatically burns dropped Credits by decreasing
		// total_supply accordingly
		let treasury_proportion = FeesTreasuryProportion::get();
		let treasury_part = treasury_proportion.deconstruct();
		let burn_part = Perbill::one().deconstruct() - treasury_part;
		let (_, to_treasury) = amount.ration(burn_part, treasury_part);
		ResolveTo::<TreasuryAccountId<R>, pallet_balances::Pallet<R>>::on_unbalanced(to_treasury);
	}

	fn deal_with_tip(amount: Credit<R::AccountId, pallet_balances::Pallet<R>>) {
		ResolveTo::<BlockAuthorAccountId<R>, pallet_balances::Pallet<R>>::on_unbalanced(amount);
	}
}

impl<R, FeesTreasuryProportion> OnUnbalanced<Credit<R::AccountId, pallet_balances::Pallet<R>>>
	for DealWithSubstrateFeesAndTip<R, FeesTreasuryProportion>
where
	R: pallet_balances::Config + pallet_treasury::Config + pallet_author_inherent::Config,
	pallet_author_inherent::Pallet<R>: Get<R::AccountId>,
	FeesTreasuryProportion: Get<Perbill>,
{
	fn on_unbalanceds(
		mut fees_then_tips: impl Iterator<Item = Credit<R::AccountId, pallet_balances::Pallet<R>>>,
	) {
		if let Some(fees) = fees_then_tips.next() {
			Self::deal_with_fees(fees);
			if let Some(tip) = fees_then_tips.next() {
				Self::deal_with_tip(tip);
			}
		}
	}
}

/// Deal with ethereum based fees. To handle tips/priority fees, use DealWithEthereumPriorityFees.
pub struct DealWithEthereumBaseFees<R, FeesTreasuryProportion>(
	sp_std::marker::PhantomData<(R, FeesTreasuryProportion)>,
);
impl<R, FeesTreasuryProportion> OnUnbalanced<Credit<R::AccountId, pallet_balances::Pallet<R>>>
	for DealWithEthereumBaseFees<R, FeesTreasuryProportion>
where
	R: pallet_balances::Config + pallet_treasury::Config,
	FeesTreasuryProportion: Get<Perbill>,
{
	fn on_nonzero_unbalanced(amount: Credit<R::AccountId, pallet_balances::Pallet<R>>) {
		// Balances pallet automatically burns dropped Credits by decreasing
		// total_supply accordingly
		let treasury_proportion = FeesTreasuryProportion::get();
		let treasury_part = treasury_proportion.deconstruct();
		let burn_part = Perbill::one().deconstruct() - treasury_part;
		let (_, to_treasury) = amount.ration(burn_part, treasury_part);
		ResolveTo::<TreasuryAccountId<R>, pallet_balances::Pallet<R>>::on_unbalanced(to_treasury);
	}
}

pub struct BlockAuthorAccountId<R>(sp_std::marker::PhantomData<R>);
impl<R> TypedGet for BlockAuthorAccountId<R>
where
	R: frame_system::Config + pallet_author_inherent::Config,
	pallet_author_inherent::Pallet<R>: Get<R::AccountId>,
{
	type Type = R::AccountId;
	fn get() -> Self::Type {
		<pallet_author_inherent::Pallet<R> as Get<R::AccountId>>::get()
	}
}

/// Deal with ethereum based priority fees/tips. See DealWithEthereumBaseFees for base fees.
pub struct DealWithEthereumPriorityFees<R>(sp_std::marker::PhantomData<R>);
impl<R> OnUnbalanced<Credit<R::AccountId, pallet_balances::Pallet<R>>>
	for DealWithEthereumPriorityFees<R>
where
	R: pallet_balances::Config + pallet_author_inherent::Config,
	pallet_author_inherent::Pallet<R>: Get<R::AccountId>,
{
	fn on_nonzero_unbalanced(amount: Credit<R::AccountId, pallet_balances::Pallet<R>>) {
		ResolveTo::<BlockAuthorAccountId<R>, pallet_balances::Pallet<R>>::on_unbalanced(amount);
	}
}

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

use core::marker::PhantomData;
use cumulus_primitives_core::Weight;
use frame_support::pallet_prelude::TypedGet;
use frame_support::traits::fungible::Credit;
use frame_support::traits::tokens::imbalance::ResolveTo;
use frame_support::traits::OnUnbalanced;
use frame_support::traits::{Get, Imbalance};
use frame_support::weights::ConstantMultiplier;
use moonbeam_core_primitives::Balance;
use pallet_treasury::TreasuryAccountId;
use sp_runtime::{Perbill, SaturatedConversion};

/// Type alias for converting reference time weight to fee using a constant multiplier.
///
/// This maps computational weight (ref_time) to a fee amount by multiplying
/// the weight by a constant factor `M`.
pub type RefTimeToFee<M> = ConstantMultiplier<Balance, M>;

/// Type alias for converting proof size weight to fee using a constant multiplier.
///
/// This maps the proof size (PoV size) component of weight to a fee amount
/// by multiplying by a constant factor `M`.
pub struct ProofSizeToFee<M>(PhantomData<M>);

impl<M> frame_support::weights::WeightToFee for ProofSizeToFee<M>
where
	M: Get<Balance>,
{
	type Balance = Balance;

	fn weight_to_fee(weight: &Weight) -> Self::Balance {
		Self::Balance::saturated_from(weight.proof_size()).saturating_mul(M::get())
	}
}

/// Combines reference time and proof size fees, charging by the more scarce resource.
///
/// This struct implements `WeightToFee` by computing fees for both the ref_time and
/// proof_size components of a `Weight`, then returning the maximum of the two.
/// This ensures transactions are charged based on whichever resource they consume
/// more of relative to block limits.
pub struct WeightToFee<RefTimeToFee, ProofSizeToFee>(PhantomData<(RefTimeToFee, ProofSizeToFee)>);
impl<
		RefTimeToFee: frame_support::weights::WeightToFee<Balance = Balance>,
		ProofSizeToFee: frame_support::weights::WeightToFee<Balance = Balance>,
	> frame_support::weights::WeightToFee for WeightToFee<RefTimeToFee, ProofSizeToFee>
{
	type Balance = Balance;

	fn weight_to_fee(weight: &Weight) -> Self::Balance {
		// Take the maximum instead of the sum to charge by the more scarce resource.
		RefTimeToFee::weight_to_fee(weight).max(ProofSizeToFee::weight_to_fee(weight))
	}
}

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

// Copyright 2019-2025 PureStake Inc.
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
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>

//! Notional fungible imbalance for representing erc20 amounts inside the XCM holding.
//!
//! As of polkadot-sdk stable2603, `xcm_executor::AssetsInHolding` tracks real
//! `fungible::Credit` imbalances rather than plain `Asset` descriptors. erc20 tokens are
//! EVM-side balances, not a Substrate `fungible`, so they have no real `Credit` to hold. This
//! type carries only the amount (mirroring the executor's own `MockCredit`); the actual token
//! movement is performed by EVM calls in `withdraw_asset`/`deposit_asset`, and the notional
//! credit exists purely so the XCM executor can thread the asset through holding.

use frame_support::traits::tokens::imbalance::{
	ImbalanceAccounting, UnsafeConstructorDestructor, UnsafeManualAccounting,
};
use sp_std::boxed::Box;

/// A notional (amount-only) fungible imbalance with no real backing.
pub struct NotionalImbalance(pub u128);

impl UnsafeConstructorDestructor<u128> for NotionalImbalance {
	fn unsafe_clone(&self) -> Box<dyn ImbalanceAccounting<u128>> {
		Box::new(NotionalImbalance(self.0))
	}
	fn forget_imbalance(&mut self) -> u128 {
		let amount = self.0;
		self.0 = 0;
		amount
	}
}

impl UnsafeManualAccounting<u128> for NotionalImbalance {
	fn saturating_subsume(&mut self, mut other: Box<dyn ImbalanceAccounting<u128>>) {
		self.0 = self.0.saturating_add(other.forget_imbalance());
	}
}

impl ImbalanceAccounting<u128> for NotionalImbalance {
	fn amount(&self) -> u128 {
		self.0
	}
	fn saturating_take(&mut self, amount: u128) -> Box<dyn ImbalanceAccounting<u128>> {
		let taken = self.0.min(amount);
		self.0 -= taken;
		Box::new(NotionalImbalance(taken))
	}
}

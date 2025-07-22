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
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

//! Core test environment and location helpers for XCM tests

use xcm::latest::prelude::{AccountKey20, Location, Parachain};
use xcm_simulator::TestExt;

// Location creation helpers

pub fn account_key20_location(account: [u8; 20]) -> Location {
	AccountKey20 {
		network: None,
		key: account,
	}
	.into()
}

pub fn parachain_location(para_id: u32) -> Location {
	Location {
		parents: 1,
		interior: [Parachain(para_id)].into(),
	}
}

pub fn para_to_para_location(dest_para: u32, account: [u8; 20]) -> Location {
	Location {
		parents: 1,
		interior: [
			Parachain(dest_para),
			AccountKey20 {
				network: None,
				key: account,
			},
		]
		.into(),
	}
}

// Account funding helper
pub fn fund_account_native(account: &[u8; 20], amount: u128) {
	use crate::xcm_mock::{parachain, ParaA};

	ParaA::execute_with(|| {
		let account_id = parachain::AccountId::from(*account);
		let _ = parachain::Balances::force_set_balance(
			parachain::RuntimeOrigin::root(),
			account_id.into(),
			amount,
		);
	});
}

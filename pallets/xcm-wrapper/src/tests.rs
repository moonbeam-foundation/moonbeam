// Copyright 2019-2021 PureStake Inc.
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

//! Unit testing
use crate::*;
use frame_support::{assert_noop, assert_ok, PalletId};
use mock::*;
use sp_runtime::traits::AccountIdConversion;
use sp_runtime::AccountId32;
const main_account: PalletId = PalletId(*b"pc/lqstk");
use cumulus_primitives_core::ParaId;
use hex_literal::hex;
use substrate_fixed::types::U64F64;
use xcm::v0::prelude::*;

#[test]
fn encode_proxy_works() {
	ExtBuilder::default()
		.with_balances(vec![(1, 1000)])
		.build()
		.execute_with(|| {
			// Insert contributors
			roll_to(4);

			assert_ok!(LiquidStaking::create_proxy(
				Origin::signed(1),
				RelayProxyType::Any,
				100u32.into(),
				0u16,
				200u32.into(),
			));

			let events = events();
			let myevent: &crate::Event<Test> = events.first().unwrap();
			let expectedEvent = crate::Event::XcmSent::<Test>(
				MultiLocation::Null,
				Xcm::WithdrawAsset {
					assets: vec![MultiAsset::ConcreteFungible {
						id: MultiLocation::Null,
						amount: 100,
					}],
					effects: vec![BuyExecution {
						fees: All,
						weight: 200,
						debt: 200,
						halt_on_error: false,
						xcm: vec![Transact {
							origin_type: OriginKind::SovereignAccount,
							require_weight_at_most: 200,
							call: hex!("1e0400000000000000").to_vec().into(),
						}],
					}],
				},
			);
			assert_eq!(expectedEvent, *myevent);
		});
}

#[test]
fn encode_staking_works() {
	ExtBuilder::default()
		.with_balances(vec![(1, 1000)])
		.build()
		.execute_with(|| {
			// Insert contributors
			roll_to(4);
			let account: AccountId32 = [1u8; 32].into();
			assert_ok!(LiquidStaking::bond(
				Origin::signed(1),
				100u32.into(),
				account,
				200u32.into(),
			));

			let events = events();
			let myevent: &crate::Event<Test> = events.first().unwrap();
			let expectedEvent = crate::Event::XcmSent::<Test>(
				MultiLocation::Null,
				Xcm::WithdrawAsset {
					assets: vec![MultiAsset::ConcreteFungible {
						id: MultiLocation::Null,
						amount: 100,
					}],
					effects: vec![BuyExecution {
						fees: All,
						weight: 200,
						debt: 200,
						halt_on_error: false,
						xcm: vec![Transact {
							origin_type: OriginKind::SovereignAccount,
							require_weight_at_most: 200,
							call: hex!("1e000101010101010101010101010101010101010101010101010101010101010101000600000101010101010101010101010101010101010101010101010101010101010101910102").to_vec().into(),
						}],
					}],
				},
			);
			assert_eq!(expectedEvent, *myevent);
		});
}

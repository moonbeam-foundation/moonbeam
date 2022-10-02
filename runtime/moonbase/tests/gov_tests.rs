// Copyright 2019-2022 PureStake Inc.
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

//! Governance tests

mod common;
use common::*;

use frame_support::{
	assert_noop, assert_ok,
	traits::{schedule::DispatchTime, PreimageRecipient},
};
use moonbase_runtime::{governance::*, Preimage, Referenda};
//use nimbus_primitives::NimbusId;
use pallet_referenda::ReferendumInfo;
use sp_core::{ByteArray, Encode, H256};
use sp_runtime::traits::{BlakeTwo256, Hash};

pub fn set_balance_proposal(value: u128) -> Vec<u8> {
	Call::Balances(pallet_balances::Call::set_balance {
		who: AccountId::from(ALICE),
		new_free: value,
		new_reserved: 0,
	})
	.encode()
}

fn set_balance_proposal_hash(value: u128) -> H256 {
	let c = Call::Balances(pallet_balances::Call::set_balance {
		who: AccountId::from(ALICE),
		new_free: value,
		new_reserved: 0,
	});
	<Preimage as PreimageRecipient<_>>::note_preimage(c.encode().try_into().unwrap());
	BlakeTwo256::hash_of(&c)
}

#[test]
fn referenda_is_immediately_ongoing() {
	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 2_000 * UNIT)])
		.build()
		.execute_with(|| {
			assert_ok!(Referenda::submit(
				origin_of(AccountId::from(ALICE)),
				Box::new(frame_system::RawOrigin::Root.into()),
				set_balance_proposal_hash(1),
				DispatchTime::At(10),
			));
			let is_ongoing = match pallet_referenda::ReferendumInfoFor::<Runtime>::get(0) {
				Some(ReferendumInfo::Ongoing(_)) => true,
				_ => false,
			};
			assert!(is_ongoing);
		});
}

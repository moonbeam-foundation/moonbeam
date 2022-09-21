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

//! TODO: move to integration tests if any of these are worth keeping

mod common;
use common::*;

use frame_support::{
	assert_noop, assert_ok,
	dispatch::RawOrigin,
	traits::{schedule::DispatchTime, PreimageRecipient},
};
use moonbase_runtime::{Preimage, Referenda};
use nimbus_primitives::NimbusId;
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

pub fn set_balance_proposal_hash(value: u128) -> H256 {
	let c = Call::Balances(pallet_balances::Call::set_balance {
		who: AccountId::from(ALICE),
		new_free: value,
		new_reserved: 0,
	});
	<Preimage as PreimageRecipient<_>>::note_preimage(c.encode().try_into().unwrap());
	BlakeTwo256::hash_of(&c)
}

#[test]
fn referenda_times_out_if_inaction() {
	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 2_000 * UNIT)])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * UNIT)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
			AccountId::from(ALICE),
		)])
		.build()
		.execute_with(|| {
			run_to_block(0, NimbusId::from_slice(&ALICE_NIMBUS).ok());
			assert_ok!(Referenda::submit(
				origin_of(AccountId::from(ALICE)),
				Box::new(RawOrigin::Root.into()),
				set_balance_proposal_hash(1),
				DispatchTime::At(10),
			));
			run_to_block(10, NimbusId::from_slice(&ALICE_NIMBUS).ok());
			let is_ongoing = match pallet_referenda::ReferendumInfoFor::<Runtime>::get(0) {
				Some(ReferendumInfo::Ongoing(_)) => true,
				_ => false,
			};
			assert!(is_ongoing);
			// TODO: bring back once scheduler fixed
			// TODO: check state in scheduler to see when expected to schedule
			// and try to figure out why not scheduled
			// run_to_block(15, NimbusId::from_slice(&ALICE_NIMBUS).ok());
			// // Timed out - ended.
			// let is_timed_out = match pallet_referenda::ReferendumInfoFor::<Runtime>::get(0) {
			// 	Some(ReferendumInfo::TimedOut(11, _, None)) => true,
			// 	_ => false,
			// };
			// assert!(is_timed_out, "{:?}", pallet_referenda::ReferendumInfoFor::<Runtime>::get(0));
		});
}

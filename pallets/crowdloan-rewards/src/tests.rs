// Copyright 2019-2020 PureStake Inc.
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
use mock::*;
use sp_runtime::MultiSignature;
use frame_support::{assert_ok, assert_noop};
use sp_core::Pair;
use parity_scale_codec::Encode;
#[test]
fn geneses() {
	let pairs = get_ed25519_pairs(3);
	two_assigned_three_unassigned().execute_with(|| {
		assert!(Sys::events().is_empty());
		// accounts_payable
		assert!(Crowdloan::accounts_payable(&1).is_some());
		assert!(Crowdloan::accounts_payable(&2).is_some());
		assert!(Crowdloan::accounts_payable(&3).is_none());
		assert!(Crowdloan::accounts_payable(&4).is_none());
		assert!(Crowdloan::accounts_payable(&5).is_none());


		// accounts_mapping
		assert!(Crowdloan::accounts_mapping(&[1u8; 32]).is_some());
		assert!(Crowdloan::accounts_mapping(&[2u8; 32]).is_some());
		assert!(Crowdloan::accounts_mapping(pairs[0].public().as_array_ref()).is_none());
		assert!(Crowdloan::accounts_mapping(pairs[1].public().as_array_ref()).is_none());
		assert!(Crowdloan::accounts_mapping(pairs[2].public().as_array_ref()).is_none());

		// unassociated_contributions
		assert!(Crowdloan::unassociated_contributions(&[1u8; 32]).is_none());
		assert!(Crowdloan::unassociated_contributions(&[2u8; 32]).is_none());
		assert!(Crowdloan::unassociated_contributions(pairs[0].public().as_array_ref()).is_some());
		assert!(Crowdloan::unassociated_contributions(pairs[1].public().as_array_ref()).is_some());
		assert!(Crowdloan::unassociated_contributions(pairs[2].public().as_array_ref()).is_some());
	});
}
#[test]
fn proving_assignation_works() {
	let pairs = get_ed25519_pairs(3);
	let signature: MultiSignature = pairs[0].sign(&3u64.encode()).into();
	two_assigned_three_unassigned().execute_with(|| {
		// 4 is not payable first
		assert!(Crowdloan::accounts_payable(&3).is_none());
		roll_to(4);
		// Signature is wrong, prove fails
		assert_noop!(
			Crowdloan::associate_native_identity(Origin::none(),
			4,
			pairs[0].public().into(),
			signature.clone()
			),
			Error::<Test>::InvalidClaimSignature
		);
		// Signature is right, prove passes
		assert_ok!(
			Crowdloan::associate_native_identity(Origin::none(),
			3,
			pairs[0].public().into(),
			signature.clone()
			)
		);
		// Signature is right, but address already claimed
		assert_noop!(
			Crowdloan::associate_native_identity(Origin::none(),
			3,
			pairs[0].public().into(),
			signature
			),
			Error::<Test>::AlreadyAssociated
		);
		// now three is payable
		assert!(Crowdloan::accounts_payable(&3).is_some());
		assert!(Crowdloan::unassociated_contributions(pairs[0].public().as_array_ref()).is_none());
		assert!(Crowdloan::accounts_mapping(pairs[0].public().as_array_ref()).is_some());
	});
}